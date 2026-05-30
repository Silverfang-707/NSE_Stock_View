use chrono::NaiveDate;

use sqlx::Row;

use crate::models::timeframe::Timeframe;

pub async fn fetch_ohlc(

    pool: &sqlx::Pool<
        sqlx::Postgres
    >,

    symbol: &str,

    series: &str,

    timeframe: &Timeframe,
)
-> Option<(

    NaiveDate,

    f64,
    f64,
    f64,
    f64
)>
{

    let query = match timeframe {

        Timeframe::Daily => {

            r#"
            SELECT

                trade_date,

                open_price,

                high_price,

                low_price,

                close_price

            FROM daily_prices

            WHERE symbol = $1
            AND series = $2

            ORDER BY trade_date DESC

            LIMIT 1
            "#
        }

        _ => {

            r#"
            SELECT

                MAX(trade_date)
                AS trade_date,

                (
                    ARRAY_AGG(
                        open_price
                        ORDER BY trade_date ASC
                    )
                )[1]
                AS open_price,

                MAX(high_price)
                AS high_price,

                MIN(low_price)
                AS low_price,

                (
                    ARRAY_AGG(
                        close_price
                        ORDER BY trade_date DESC
                    )
                )[1]
                AS close_price

            FROM daily_prices

            WHERE symbol = $1
            AND series = $2
            "#
        }
    };

    let row =
        sqlx::query(query)

        .bind(symbol)

        .bind(series)

        .fetch_optional(pool)

        .await

        .unwrap()?;

    Some((

        row.get("trade_date"),

        row.get("open_price"),

        row.get("high_price"),

        row.get("low_price"),

        row.get("close_price"),
    ))
}