use axum::{
    extract::{
        Path,
        State,
    },

    Json,
};

use sqlx::Row;

use crate::models::market_levels::
    MarketLevelResponse;

pub async fn get_levels(

    Path(symbol): Path<String>,

    State(pool): State<
        sqlx::Pool<sqlx::Postgres>
    >,
)
-> Json<Vec<MarketLevelResponse>>
{

    let rows =
        sqlx::query(
            r#"
            SELECT *

            FROM market_levels

            WHERE symbol = $1

            ORDER BY

                timeframe,

                trade_date DESC
            "#
        )

        .bind(&symbol)

        .fetch_all(&pool)

        .await

        .unwrap();

    let levels =
        rows
            .into_iter()

            .map(|row| {

                MarketLevelResponse {

                    timeframe:
                        row.get("timeframe"),

                    trade_date:
                        row.get::<
                            chrono::NaiveDate,
                            _
                        >("trade_date")
                        .to_string(),

                    open_price:
                        row.get("open_price"),

                    high_price:
                        row.get("high_price"),

                    low_price:
                        row.get("low_price"),

                    close_price:
                        row.get("close_price"),

                    range_value:
                        row.get("range_value"),

                    buffer_value:
                        row.get("buffer_value"),

                    jgd:
                        row.get("jgd"),

                    jwd:
                        row.get("jwd"),

                    bdp:
                        row.get("bdp"),

                    wdp:
                        row.get("wdp"),

                    pattern:
                        row.get("pattern"),
                }
            })

            .collect();

    Json(levels)
}