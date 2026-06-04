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

use std::env;

use market_core::ingest::ingest_day;

pub async fn update_market(
    State(pool): State<sqlx::Pool<sqlx::Postgres>>,
) -> Json<serde_json::Value> {

    println!("🚀 Starting update...");

    // ── 1. Fetch latest date ────────────────────────────────────────────────

    let row = sqlx::query(
        r#"SELECT MAX(trade_date) AS latest_date FROM daily_prices"#,
    )
    .fetch_one(&pool)
    .await;

    let latest_date: NaiveDate = match row {
        Ok(row) => match row.try_get("latest_date") {
            Ok(date) => date,
            Err(err) => {
                return Json(json!({
                    "success": false,
                    "stage": "db_read",
                    "error": format!("latest_date column missing or wrong type: {}", err)
                }));
            }
        },
        Err(sqlx::Error::RowNotFound) => {
            return Json(json!({
                "success": false,
                "stage": "db_read",
                "error": "daily_prices table is empty — nothing to update from"
            }));
        }
        Err(err) => {
            return Json(json!({
                "success": false,
                "stage": "db_read",
                "error": format!("Failed to read latest date: {}", err)
            }));
        }
    };

    let today = Utc::now().date_naive();

    println!("📅 Latest DB date: {}", latest_date);
    println!("📅 Today:          {}", today);

    // ── 2. Guard: nothing to do ─────────────────────────────────────────────

    if latest_date >= today {
        println!("✅ Already up to date");
        return Json(json!({
            "success": true,
            "message": "Already up to date",
            "days_processed": 0,
            "rows": 0,
            "latest_date": latest_date.to_string(),
            "updated_to": today.to_string(),
            "calculator": "skipped"
        }));
    }

    // ── 3. Ingest missing days ──────────────────────────────────────────────

    let mut current = latest_date + Duration::days(1);
    let mut total_rows: i64 = 0;
    let mut days_processed: u32 = 0;
    let mut ingest_errors: Vec<String> = Vec::new();

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

    // ── 4. Fire-and-forget calculator ───────────────────────────────────────
    //
    // We spawn the calculator as a background task and return the HTTP
    // response immediately. This means:
    //
    //   • Cloudflare / proxies / mobile clients never time out waiting
    //     for a long-running process.
    //   • The ingestion result is visible in the UI right away.
    //   • Calculator progress streams to your server logs as normal.
    //
    // Trade-off: the response can't report whether the calculator
    // succeeded — the caller should check server logs or a status
    // endpoint if that matters.

    let calculator_path = resolve_calculator_path();

    println!(
        "🧮 Launching calculator in background: {}",
        calculator_path.display()
    );

    tokio::spawn(async move {
        match Command::new(&calculator_path).status().await {
            Ok(status) if status.success() => {
                println!("✅ Calculator complete");
            }
            Ok(status) => {
                println!(
                    "❌ Calculator exited with status {:?}",
                    status.code()
                );
            }
            Err(err) => {
                println!(
                    "❌ Calculator launch failed (path: {}): {}",
                    calculator_path.display(),
                    err
                );
            }
        }
    });

    // ── 5. Return immediately ───────────────────────────────────────────────

    Json(json!({
        "success":        true,
        "message":        "Update started — calculator running in background",
        "days_processed": days_processed,
        "rows":           total_rows,
        "latest_date":    latest_date.to_string(),
        "updated_to":     today.to_string(),
        "ingest_errors":  ingest_errors,
        "calculator":     "running"
    }))
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Resolves the path to the calculator binary.
///
/// Resolution order:
///   1. `CALCULATOR_BIN` env var      — easiest override in prod / Docker
///   2. Sibling of the current exe    — correct when both are in target/release/
///   3. `target/release/calculator`   — fallback for dev / `cargo run`
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