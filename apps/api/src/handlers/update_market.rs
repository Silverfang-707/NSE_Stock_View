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

use market_core::ingest::ingest_day;

pub async fn update_market(

    State(pool): State<
        sqlx::Pool<sqlx::Postgres>
    >

) -> Json<serde_json::Value> {

    println!("🚀 Starting update...");

    let row = sqlx::query(
        r#"
        SELECT MAX(trade_date)
        AS latest_date

        FROM daily_prices
        "#
    )
    .fetch_one(&pool)
    .await;

    let latest_date: NaiveDate = match row {

        Ok(row) => row.get("latest_date"),

        Err(err) => {

            return Json(json!({

                "success": false,

                "error":
                    format!(
                        "Failed to read latest date: {}",
                        err
                    )
            }));
        }
    };

    let today =
        Utc::now().date_naive();

    println!(
        "📅 Latest DB date: {}",
        latest_date
    );

    println!(
        "📅 Today: {}",
        today
    );

    let mut current =
        latest_date + Duration::days(1);

    let mut total_rows = 0;

    let mut days_processed = 0;

    while current <= today {

        if current
            .weekday()
            .number_from_monday()
            < 6
        {

            match ingest_day(
                &pool,
                current
            )
            .await
            {

                Ok(rows) => {

                    total_rows += rows;

                    days_processed += 1;

                    println!(
                        "✅ {} -> {} rows",
                        current,
                        rows
                    );
                }

                Err(err) => {

                    println!(
                        "❌ {} -> {}",
                        current,
                        err
                    );
                }
            }
        }

        current += Duration::days(1);
    }

    println!(
        "✅ Ingestion complete"
    );

    println!(
        "🧮 Starting calculator..."
    );

    let output =

        Command::new("cargo")

            .args([
                "run",
                "-p",
                "calculator"
            ])

            .output()

            .await;

    match output {

        Ok(result) => {

            if !result.status.success() {

                return Json(json!({

                    "success": false,

                    "stage": "calculator",

                    "error":
                        String::from_utf8_lossy(
                            &result.stderr
                        )
                }));
            }
        }

        Err(err) => {

            return Json(json!({

                "success": false,

                "stage": "calculator",

                "error":
                    err.to_string()
            }));
        }
    }

    println!(
        "✅ Calculator complete"
    );

    Json(json!({

        "success": true,

        "message":
            "Database updated successfully",

        "days_processed":
            days_processed,

        "rows":
            total_rows,

        "latest_date":
            latest_date.to_string(),

        "updated_to":
            today.to_string()
    }))
}