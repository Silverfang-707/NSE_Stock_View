use anyhow::Result;

use dotenvy::dotenv;

use std::env;

use chrono::NaiveDate;

use db::create_pool;

use sqlx::Row;

mod calculations;
mod models;
mod services;

use calculations::range::calculate_range;

use calculations::buffer::calculate_buffer;

use calculations::jgd::calculate_jgd;

use calculations::jwd::calculate_jwd;

use calculations::bdp::calculate_bdp;

use calculations::wdp::calculate_wdp;

use calculations::patterns::detect_pattern;

use models::level::Level;

use services::{
    fetch::fetch_daily_prices,
    insert::insert_level,
};

// =====================================
// GET LATEST CALCULATED DATE
// =====================================

async fn get_latest_level_date(

    pool: &sqlx::Pool<sqlx::Postgres>

)
-> Option<NaiveDate>
{

    let row =
        sqlx::query(
            r#"
            SELECT

                MAX(trade_date)
                AS latest_date

            FROM daily_levels
            "#
        )

        .fetch_one(pool)

        .await

        .ok()?;

    row.try_get("latest_date")
        .ok()
}

// =====================================
// MAIN
// =====================================

#[tokio::main]

async fn main() -> Result<()>
{

    dotenv().ok();

    // =====================================
    // DATABASE
    // =====================================

    let database_url =
        env::var("DATABASE_URL")?;

    let pool =
        create_pool(&database_url)
            .await;

    println!(
        "✅ Calculator DB Connected"
    );

    // =====================================
    // LATEST CALCULATED DATE
    // =====================================

    let latest_level_date =
        get_latest_level_date(&pool)
            .await;

    match latest_level_date {

        Some(date) => {

            println!(
                "📅 Latest calculated date: {}",
                date
            );
        }

        None => {

            println!(
                "⚠️ No existing calculated levels"
            );
        }
    }

    // =====================================
    // FETCH RAW DAILY PRICES
    // =====================================

    println!(
        "\n📥 Fetching daily prices..."
    );

    let rows =
        fetch_daily_prices(&pool)
            .await;

    println!(
        "✅ Fetched {} rows",
        rows.len()
    );

    // =====================================
    // PROCESS ROWS
    // =====================================

    let mut processed = 0;

    for row in rows {

        let trade_date: NaiveDate =
            row.get("trade_date");

        // =================================
        // INCREMENTAL CALCULATION
        // =================================

        if let Some(latest_date) =
            latest_level_date
        {

            if trade_date <= latest_date {

                continue;
            }
        }

        // =================================
        // FETCH RAW VALUES
        // =================================

        let symbol: String =
            row.get("symbol");

        let series: String =
            row.get("series");

        let open_price: f64 =
            row.get("open_price");

        let high_price: f64 =
            row.get("high_price");

        let low_price: f64 =
            row.get("low_price");

        let close_price: f64 =
            row.get("close_price");

        // =================================
        // CALCULATIONS
        // =================================

        let range_value =
            calculate_range(
                high_price,
                low_price
            );

        let buffer_value =
            calculate_buffer(
                range_value
            );

        let jgd =
            calculate_jgd(
                high_price,
                buffer_value
            );

        let jwd =
            calculate_jwd(
                low_price,
                buffer_value
            );

        let bdp =
            calculate_bdp(
                close_price,
                buffer_value
            );

        let wdp =
            calculate_wdp(
                close_price,
                buffer_value
            );

        let pattern =
            detect_pattern(
                open_price,
                close_price
            );

        // =================================
        // BUILD LEVEL MODEL
        // =================================

        let level = Level {

            symbol,

            series,

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
        };

        // =================================
        // INSERT LEVEL
        // =================================

        insert_level(
            &pool,
            &level
        )
        .await;

        processed += 1;

        // =================================
        // LOGGING
        // =================================

        if processed % 1000 == 0 {

            println!(
                "⚡ Processed {} rows",
                processed
            );
        }
    }

    // =====================================
    // COMPLETE
    // =====================================

    println!(
        "\n🎉 Calculation Complete"
    );

    println!(
        "📦 Total Levels Generated: {}",
        processed
    );

    Ok(())
}