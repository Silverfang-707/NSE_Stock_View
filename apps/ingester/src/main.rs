use anyhow::Result;
use dotenvy::dotenv;

use std::env;
use std::io::Cursor;

use csv::Reader;
use zip::ZipArchive;

use db::create_pool;
use market_core::Candle;

use sqlx::query;

use chrono::Datelike;
use chrono::{
    Duration,
    NaiveDate,
    Utc,
};

async fn ingest_day(
    pool: &sqlx::Pool<sqlx::Postgres>,
    date: NaiveDate,
) -> Result<()> {

    let date_str =
        date.format("%Y%m%d");

    let url = format!(
        "https://nsearchives.nseindia.com/content/cm/BhavCopy_NSE_CM_0_0_0_{}_F_0000.csv.zip",
        date_str
    );

    println!("\n📡 Downloading {}", date_str);

    let response =
        reqwest::get(&url).await?;

    if !response.status().is_success() {

        println!(
            "⚠️ Skipping {} (likely holiday)",
            date_str
        );

        return Ok(());
    }

    let bytes =
        response.bytes().await?;

    let reader =
        Cursor::new(bytes);

    let mut archive =
        ZipArchive::new(reader)?;

    let file =
        archive.by_index(0)?;

    let mut csv_reader =
        Reader::from_reader(file);

    let mut inserted = 0;

    for result in
        csv_reader.deserialize::<Candle>()
    {
        let candle = result?;

        query(
            r#"
            INSERT INTO daily_prices (
                symbol,
                series,
                trade_date,
                open_price,
                high_price,
                low_price,
                close_price,
                volume
            )
            VALUES (
                $1, $2, $3, $4,
                $5, $6, $7, $8
            )
            ON CONFLICT DO NOTHING
            "#
        )
        .bind(&candle.symbol)
        .bind(&candle.series)
        .bind(candle.trade_date)
        .bind(candle.open_price)
        .bind(candle.high_price)
        .bind(candle.low_price)
        .bind(candle.close_price)
        .bind(candle.volume)
        .execute(pool)
        .await?;

        inserted += 1;
    }

    println!(
        "✅ Inserted {} rows for {}",
        inserted,
        date_str
    );

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {

    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL")?;

    let pool =
        create_pool(&database_url).await;

    println!("✅ Database Connected");

    let today =
        Utc::now().date_naive();

    let days_back = 7;

    for i in 1..=days_back {

        let date =
            today - Duration::days(i);

        // Skip weekends
        if date.weekday().number_from_monday() >= 6 {

            println!(
                "⏭️ Skipping weekend {}",
                date
            );

            continue;
        }

        if let Err(err) =
            ingest_day(&pool, date).await
        {
            println!(
                "❌ Failed {}: {}",
                date,
                err
            );
        }
    }

    println!("\n🚀 Historical ingestion complete");

    Ok(())
}