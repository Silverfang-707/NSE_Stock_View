use axum::{
    extract::State,
    Json,
};

use chrono::{
    Datelike,
    Duration,
    Utc,
    NaiveDate,
};

use serde_json::json;

use sqlx::Row;

use tokio::process::Command;

use std::{
    env,
    sync::atomic::{AtomicBool, Ordering},
};

use market_core::ingest::ingest_day;

// ── Global calculator lock ────────────────────────────────────────────────────

static CALCULATOR_RUNNING: AtomicBool = AtomicBool::new(false);

pub async fn update_market(
    State(pool): State<sqlx::Pool<sqlx::Postgres>>,
) -> Json<serde_json::Value> {

    println!("🚀 Starting update...");

    // ── 1. Fetch latest prices date ─────────────────────────────────────────

    let row = sqlx::query(
        r#"SELECT MAX(trade_date) AS latest_date FROM daily_prices"#,
    )
    .fetch_one(&pool)
    .await;

    let latest_prices_date: NaiveDate = match row {
        Ok(row) => match row.try_get("latest_date") {
            Ok(date) => date,
            Err(err) => {
                return Json(json!({
                    "success": false,
                    "stage":   "db_read",
                    "error":   format!("latest_date column missing or wrong type: {}", err)
                }));
            }
        },
        Err(sqlx::Error::RowNotFound) => {
            return Json(json!({
                "success": false,
                "stage":   "db_read",
                "error":   "daily_prices table is empty — nothing to update from"
            }));
        }
        Err(err) => {
            return Json(json!({
                "success": false,
                "stage":   "db_read",
                "error":   format!("Failed to read latest prices date: {}", err)
            }));
        }
    };

    // ── 2. Fetch latest levels date across ALL timeframes ───────────────────
    //
    // We take MIN(MAX(trade_date) per timeframe) so that if any single
    // timeframe is behind — daily complete, weekly missing, etc. — we
    // correctly detect the gap and relaunch the calculator.
    //
    // Using MAX(daily) alone would mask missing weekly/monthly data.

    let latest_levels_date: Option<NaiveDate> = sqlx::query(
        r#"
        SELECT MIN(last_date) AS latest_date
        FROM (
            SELECT timeframe,
                   MAX(trade_date) AS last_date
            FROM market_levels
            GROUP BY timeframe
        ) t
        "#,
    )
    .fetch_one(&pool)
    .await
    .ok()
    .and_then(|row| row.try_get("latest_date").ok());

    let today = Utc::now().date_naive();

    println!("📅 Latest prices date: {}", latest_prices_date);
    println!(
        "📅 Latest levels date: {}",
        latest_levels_date
            .map(|d| d.to_string())
            .unwrap_or_else(|| "none".to_string())
    );
    println!("📅 Today:              {}", today);

    // ── 3. Guard: truly nothing to do ──────────────────────────────────────
    //
    // Only return early if prices AND every timeframe in market_levels
    // are fully current. A single lagging timeframe keeps us going.

    let levels_are_current = latest_levels_date
        .map(|d| d >= latest_prices_date)
        .unwrap_or(false);

    if latest_prices_date >= today && levels_are_current {
        println!("✅ Already up to date");
        return Json(json!({
            "success":        true,
            "message":        "Already up to date",
            "days_processed": 0,
            "rows":           0,
            "latest_date":    latest_prices_date.to_string(),
            "updated_to":     today.to_string(),
            "calculator":     "skipped"
        }));
    }

    // ── 4. Ingest missing days (only if prices are behind) ──────────────────

    let mut total_rows: i64 = 0;
    let mut days_processed: u32 = 0;
    let mut ingest_errors: Vec<String> = Vec::new();

    if latest_prices_date < today {

        let mut current = latest_prices_date + Duration::days(1);

        while current <= today {
            let weekday = current.weekday().number_from_monday();

            if weekday < 6 {
                match ingest_day(&pool, current).await {
                    Ok(rows) => {
                        total_rows += rows as i64;
                        days_processed += 1;
                        println!("✅ {} → {} rows", current, rows);
                    }
                    Err(err) => {
                        let msg = format!("{}: {}", current, err);
                        println!("❌ {}", msg);
                        ingest_errors.push(msg);
                    }
                }
            }

            current += Duration::days(1);
        }

        println!(
            "✅ Ingestion complete ({} days, {} rows)",
            days_processed,
            total_rows
        );

    } else {
        println!("⚠️  Prices current but levels are behind — launching calculator catch-up");
    }

    // ── 5. Guard: refuse concurrent calculator launches ─────────────────────

    if CALCULATOR_RUNNING
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        println!("⚠️  Calculator already running — skipping launch");
        return Json(json!({
            "success":        true,
            "message":        "Market data ingested. Calculator already running in background.",
            "days_processed": days_processed,
            "rows":           total_rows,
            "latest_date":    latest_prices_date.to_string(),
            "updated_to":     today.to_string(),
            "ingest_errors":  ingest_errors,
            "calculator":     "already_running"
        }));
    }

    // ── 6. Fire-and-forget calculator ───────────────────────────────────────

    let calculator_path = resolve_calculator_path();

    println!(
        "🧮 Launching calculator in background: {}",
        calculator_path.display()
    );

    tokio::spawn(async move {

        // Drop guard — resets the flag however the task exits:
        // success, failure, or panic.
        struct CalculatorGuard;
        impl Drop for CalculatorGuard {
            fn drop(&mut self) {
                CALCULATOR_RUNNING.store(false, Ordering::SeqCst);
                println!("🔓 Calculator lock released");
            }
        }
        let _guard = CalculatorGuard;

        match Command::new(&calculator_path).status().await {
            Ok(status) if status.success() => {
                println!("✅ Calculator complete");
            }
            Ok(status) => {
                println!(
                    "❌ Calculator exited with status {:?} — \
                     levels may be incomplete; next update will retry",
                    status.code()
                );
            }
            Err(err) => {
                println!(
                    "❌ Calculator launch failed (path: {}): {} — \
                     next update will retry",
                    calculator_path.display(),
                    err
                );
            }
        }

        // _guard drops here → CALCULATOR_RUNNING = false
    });

    // ── 7. Return immediately ───────────────────────────────────────────────

    let message = if days_processed > 0 {
        "Market data ingested. Calculator running in background."
    } else {
        "Prices already current. Calculator catching up on missing levels."
    };

    Json(json!({
        "success":        true,
        "message":        message,
        "days_processed": days_processed,
        "rows":           total_rows,
        "latest_date":    latest_prices_date.to_string(),
        "updated_to":     today.to_string(),
        "ingest_errors":  ingest_errors,
        "calculator":     "running"
    }))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn resolve_calculator_path() -> std::path::PathBuf {
    if let Ok(val) = env::var("CALCULATOR_BIN") {
        return std::path::PathBuf::from(val);
    }

    let exe_name = if cfg!(windows) {
        "calculator.exe"
    } else {
        "calculator"
    };

    if let Ok(mut path) = env::current_exe() {
        path.pop();
        path.push(exe_name);
        if path.exists() {
            return path;
        }
    }

    std::path::PathBuf::from("target/release").join(exe_name)
}