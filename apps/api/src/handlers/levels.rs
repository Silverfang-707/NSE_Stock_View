use axum::{
    extract::{
        Path,
        Query,
        State,
    },
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use serde::Deserialize;
use sqlx::Row;
use tracing::error;

use crate::models::market_levels::
    MarketLevelResponse;

// =====================================
// QUERY PARAMS
// =====================================

#[derive(Deserialize)]
pub struct LevelQuery {
    pub series: Option<String>,
}

pub async fn get_levels(
    Path(symbol): Path<String>,
    Query(query): Query<LevelQuery>,
    State(pool): State<
        sqlx::Pool<sqlx::Postgres>
    >,
) -> Result<Json<Vec<MarketLevelResponse>>, (StatusCode, String)> {

    let rows = sqlx::query(
        r#"
        SELECT
            timeframe,
            trade_date,
            open_price,
            high_price,
            low_price,
            close_price,
            range_value,
            buffer_value,
            jgd,
            jwd,
            bdp,
            wdp,
            pattern,
            
            LAG(jgd) OVER (
                PARTITION BY timeframe, series
                ORDER BY trade_date
            ) AS prev_jgd,

            LAG(jwd) OVER (
                PARTITION BY timeframe, series
                ORDER BY trade_date
            ) AS prev_jwd,

            LAG(bdp) OVER (
                PARTITION BY timeframe, series
                ORDER BY trade_date
            ) AS prev_bdp,

            LAG(wdp) OVER (
                PARTITION BY timeframe, series
                ORDER BY trade_date
            ) AS prev_wdp

        FROM market_levels
        WHERE symbol = $1 
          AND ($2::text IS NULL OR LOWER(series) = LOWER($2))
        ORDER BY
            timeframe,
            trade_date DESC
        "#
    )
    .bind(&symbol)
    .bind(&query.series)
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        error!(%symbol, error = %e, "Database query failed in get_levels");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    })?;

    // THE FIX: We collect into a Result<Vec, sqlx::Error>. 
    // If ANY row.try_get() fails (due to a unexpected NULL or type mismatch), 
    // the ? operator short-circuits the entire loop and returns the Err.
    let levels_result: Result<Vec<MarketLevelResponse>, sqlx::Error> = rows
        .into_iter()
        .map(|row| {
            Ok(MarketLevelResponse {
                // try_get will safely error if the field is missing or unexpectedly NULL
                timeframe: row.try_get("timeframe")?,
                
                // Safely extracts the date, formats it, or triggers the error short-circuit
                trade_date: row.try_get::<chrono::NaiveDate, _>("trade_date")?.to_string(),
                
                open_price:  row.try_get("open_price")?,
                high_price:  row.try_get("high_price")?,
                low_price:   row.try_get("low_price")?,
                close_price: row.try_get("close_price")?,
                
                range_value:  row.try_get("range_value")?,
                buffer_value: row.try_get("buffer_value")?,
                
                jgd: row.try_get("jgd")?,
                jwd: row.try_get("jwd")?,
                bdp: row.try_get("bdp")?,
                wdp: row.try_get("wdp")?,
                
                pattern: row.try_get("pattern")?,

                // Because the struct expects Option<T> for these fields, 
                // try_get automatically maps DB NULLs to None, and real values to Some(val).
                // It will only throw an error if there is a severe type mismatch.
                prev_jgd: row.try_get("prev_jgd")?,
                prev_jwd: row.try_get("prev_jwd")?,
                prev_bdp: row.try_get("prev_bdp")?,
                prev_wdp: row.try_get("prev_wdp")?,
            })
        })
        .collect();

    // Map any row-parsing errors to our standard HTTP 500 response
    let levels = levels_result.map_err(|e| {
        error!(%symbol, error = %e, "Data mapping failed in get_levels - schema mismatch or unexpected NULL");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal Server Error".to_string(),
        )
    })?;

    Ok(Json(levels))
}