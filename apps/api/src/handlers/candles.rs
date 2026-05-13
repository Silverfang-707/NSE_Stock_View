use axum::{
    extract::{Path, State},
    Json,
};

use sqlx::Row;

use crate::models::candle::CandleResponse;

pub async fn get_candles(

    Path(symbol): Path<String>,

    State(pool): State<
        sqlx::Pool<sqlx::Postgres>
    >,
) -> Json<Vec<CandleResponse>> {

    let rows =
        sqlx::query(
            r#"
            SELECT
                trade_date,
                series,
                open_price,
                high_price,
                low_price,
                close_price,
                volume
            FROM daily_prices
            WHERE symbol = $1
            ORDER BY trade_date
            "#
        )
        .bind(symbol)
        .fetch_all(&pool)
        .await
        .unwrap();

    let candles =
        rows
            .into_iter()
            .map(|row| {

                CandleResponse {

                    trade_date:
                        row.get("trade_date"),

                    series:
                        row.get("series"),

                    open_price:
                        row.get("open_price"),

                    high_price:
                        row.get("high_price"),

                    low_price:
                        row.get("low_price"),

                    close_price:
                        row.get("close_price"),

                    volume:
                        row.get("volume"),
                }
            })
            .collect();

    Json(candles)
}