use axum::{
    extract::{
        Path,
        State,
    },
    Json,
};

use sqlx::Row;

use patterns::{
    daily::generate_daily_plan,
    timeframe::Timeframe,
};

// =====================================
// FETCH TIMEFRAME DATA
// =====================================

async fn fetch_timeframe_data(

    pool: &sqlx::Pool<sqlx::Postgres>,

    symbol: &str,

    timeframe: Timeframe,

)
-> (f64, f64, f64)
{

    let query = format!(
        r#"
        SELECT

            MAX(high_price) AS high,

            MIN(low_price) AS low,

            (

                ARRAY_AGG(
                    close_price

                    ORDER BY
                        trade_date DESC
                )

            )[1] AS close

        FROM daily_prices

        WHERE symbol = $1

        AND trade_date >= (

            SELECT
                MAX(trade_date)

            FROM daily_prices

            WHERE symbol = $1

        ) - INTERVAL '{}'
        "#,

        timeframe.interval()
    );

    let row =
        sqlx::query(&query)

            .bind(symbol)

            .fetch_one(pool)

            .await

            .unwrap();

    let high: Option<f64> =
        row.get("high");

    let low: Option<f64> =
        row.get("low");

    let close: Option<f64> =
        row.get("close");

    (
        high.unwrap_or(0.0),

        low.unwrap_or(0.0),

        close.unwrap_or(0.0),
    )
}

// =====================================
// ANALYSIS ENDPOINT
// =====================================

pub async fn get_analysis(

    Path(symbol): Path<String>,

    State(pool): State<
        sqlx::Pool<sqlx::Postgres>
    >,
)
-> Json<serde_json::Value>
{

    println!(
        "📊 Requested analysis for {}",
        symbol
    );

    // =====================================
    // DAILY
    // =====================================

    let (
        d_high,
        d_low,
        d_close
    ) = fetch_timeframe_data(
        &pool,
        &symbol,
        Timeframe::Daily
    )
    .await;

    // =====================================
    // WEEKLY
    // =====================================

    let (
        w_high,
        w_low,
        w_close
    ) = fetch_timeframe_data(
        &pool,
        &symbol,
        Timeframe::Weekly
    )
    .await;

    // =====================================
    // MONTHLY
    // =====================================

    let (
        m_high,
        m_low,
        m_close
    ) = fetch_timeframe_data(
        &pool,
        &symbol,
        Timeframe::Monthly
    )
    .await;

    // =====================================
    // QUARTERLY
    // =====================================

    let (
        q_high,
        q_low,
        q_close
    ) = fetch_timeframe_data(
        &pool,
        &symbol,
        Timeframe::Quarterly
    )
    .await;

    // =====================================
    // HALF YEARLY
    // =====================================

    let (
        h_high,
        h_low,
        h_close
    ) = fetch_timeframe_data(
        &pool,
        &symbol,
        Timeframe::HalfYearly
    )
    .await;

    // =====================================
    // YEARLY
    // =====================================

    let (
        y_high,
        y_low,
        y_close
    ) = fetch_timeframe_data(
        &pool,
        &symbol,
        Timeframe::Yearly
    )
    .await;

    // =====================================
    // GENERATE ANALYSIS
    // =====================================

    let daily =
        generate_daily_plan(
            symbol.clone(),
            d_high,
            d_low,
            d_close,
        );

    let weekly =
        generate_daily_plan(
            symbol.clone(),
            w_high,
            w_low,
            w_close,
        );

    let monthly =
        generate_daily_plan(
            symbol.clone(),
            m_high,
            m_low,
            m_close,
        );

    let quarterly =
        generate_daily_plan(
            symbol.clone(),
            q_high,
            q_low,
            q_close,
        );

    let half_yearly =
        generate_daily_plan(
            symbol.clone(),
            h_high,
            h_low,
            h_close,
        );

    let yearly =
        generate_daily_plan(
            symbol.clone(),
            y_high,
            y_low,
            y_close,
        );

    println!(
        "✅ Analysis generated for {}",
        symbol
    );

    Json(
        serde_json::json!({

            "symbol": symbol,

            "daily": daily,

            "weekly": weekly,

            "monthly": monthly,

            "quarterly": quarterly,

            "half_yearly": half_yearly,

            "yearly": yearly

        })
    )
}