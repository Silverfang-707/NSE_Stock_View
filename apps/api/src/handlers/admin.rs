use axum::Json;

use chrono::Utc;

use market_core::ingest::ingest_day;

pub async fn ingest_latest(
    axum::extract::State(
        pool
    ): axum::extract::State<
        sqlx::Pool<sqlx::Postgres>
    >
) -> Json<serde_json::Value> {

    let today =
        Utc::now().date_naive();

    match ingest_day(&pool, today).await {

        Ok(rows) => {

            Json(
                serde_json::json!({
                    "success": true,
                    "rows": rows
                })
            )
        }

        Err(err) => {

            Json(
                serde_json::json!({
                    "success": false,
                    "error": err.to_string()
                })
            )
        }
    }
}