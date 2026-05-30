use chrono::NaiveDate;
use sqlx::postgres::PgRow;
use sqlx::Row;

use crate::models::ohlc::OhlcRow;

// =====================================
// FETCH UNIQUE SYMBOLS
// =====================================

pub async fn fetch_symbols(
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> Vec<PgRow> {

    sqlx::query(
        r#"
        SELECT DISTINCT symbol, series
        FROM daily_prices
        WHERE symbol IS NOT NULL
        AND series   IS NOT NULL
        ORDER BY symbol
        "#
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

// =====================================
// FETCH LATEST CALCULATED DATE
// PER SYMBOL + SERIES + TIMEFRAME
// =====================================

pub async fn fetch_latest_level_date(
    pool:      &sqlx::Pool<sqlx::Postgres>,
    symbol:    &str,
    series:    &str,
    timeframe: &str,
) -> Option<NaiveDate> {

    let row = sqlx::query(
        r#"
        SELECT MAX(trade_date) AS latest_date
        FROM market_levels
        WHERE symbol    = $1
        AND   series    = $2
        AND   timeframe = $3
        "#
    )
    .bind(symbol)
    .bind(series)
    .bind(timeframe)
    .fetch_one(pool)
    .await
    .ok()?;

    row.try_get("latest_date").ok()
}

// =====================================
// FETCH LAST KNOWN JWD/BDP/WDP
// for carry-forward on incremental runs
// =====================================

pub async fn fetch_last_level(
    pool:      &sqlx::Pool<sqlx::Postgres>,
    symbol:    &str,
    series:    &str,
    timeframe: &str,
) -> Option<(f64, f64, f64)> {
    // returns (prev_jwd, prev_bdp, prev_wdp)

    let row = sqlx::query(
        r#"
        SELECT jwd, bdp, wdp
        FROM market_levels
        WHERE symbol    = $1
        AND   series    = $2
        AND   timeframe = $3
        ORDER BY trade_date DESC
        LIMIT 1
        "#
    )
    .bind(symbol)
    .bind(series)
    .bind(timeframe)
    .fetch_optional(pool)
    .await
    .ok()??;

    Some((
        row.try_get("jwd").ok()?,
        row.try_get("bdp").ok()?,
        row.try_get("wdp").ok()?,
    ))
}

// =====================================
// FETCH ALL OHLC ROWS FOR A TIMEFRAME
// aggregated from daily_prices
// only rows after since_date
// =====================================

pub async fn fetch_timeframe_ohlc(
    pool:       &sqlx::Pool<sqlx::Postgres>,
    symbol:     &str,
    series:     &str,
    trunc:      &str,
    since_date: Option<NaiveDate>,
) -> Vec<OhlcRow> {

    let period_expr = match trunc {

        "half_yearly" => {
            r#"
            make_date(
                EXTRACT(YEAR FROM trade_date)::int,
                CASE
                    WHEN EXTRACT(MONTH FROM trade_date) <= 6
                    THEN 1
                    ELSE 7
                END,
                1
            )
            "#
        }

        _ => {
            Box::leak(
                format!(
                    "date_trunc('{}', trade_date)::date",
                    trunc
                )
                .into_boxed_str()
            )
        }
    };

    let since_filter =
        match since_date {

            Some(d) => format!(
                "AND ({}) > '{}'",
                period_expr,
                d
            ),

            None => String::new(),
        };

    let sql = format!(
        r#"
        SELECT

            ({period_expr}) AS trade_date,

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

        {since_filter}

        GROUP BY ({period_expr})

        ORDER BY ({period_expr}) ASC
        "#,
        period_expr = period_expr,
        since_filter = since_filter,
    );

    let rows =
        sqlx::query(&sql)

        .bind(symbol)

        .bind(series)

        .fetch_all(pool)

        .await

        .unwrap_or_default();

    rows.iter()
        .filter_map(|row| {

            Some(OhlcRow {

                trade_date:
                    row.try_get("trade_date").ok()?,

                open_price:
                    row.try_get("open_price").ok()?,

                high_price:
                    row.try_get("high_price").ok()?,

                low_price:
                    row.try_get("low_price").ok()?,

                close_price:
                    row.try_get("close_price").ok()?,
            })
        })
        .collect()
}