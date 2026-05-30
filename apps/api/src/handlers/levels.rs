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
            WITH levels_with_prev AS (

                SELECT

                    *,

                    LAG(jgd)
                    OVER (

                        PARTITION BY timeframe

                        ORDER BY trade_date

                    ) AS prev_jgd,

                    LAG(jwd)
                    OVER (

                        PARTITION BY timeframe

                        ORDER BY trade_date

                    ) AS prev_jwd,

                    LAG(bdp)
                    OVER (

                        PARTITION BY timeframe

                        ORDER BY trade_date

                    ) AS prev_bdp,

                    LAG(wdp)
                    OVER (

                        PARTITION BY timeframe

                        ORDER BY trade_date

                    ) AS prev_wdp

                FROM market_levels

                WHERE symbol = $1
            )

            SELECT *

            FROM levels_with_prev

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

                    prev_jgd:
                        row.try_get("prev_jgd").ok(),

                    prev_jwd:
                        row.try_get("prev_jwd").ok(),

                    prev_bdp:
                        row.try_get("prev_bdp").ok(),

                    prev_wdp:
                        row.try_get("prev_wdp").ok(),
                }
            })

            .collect();

    Json(levels)
}