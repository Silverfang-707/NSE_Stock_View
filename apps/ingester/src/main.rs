use anyhow::Result;
use dotenvy::dotenv;

use std::env;
use std::io::Cursor;

use clap::Parser;
use csv::Reader;
use zip::ZipArchive;

use db::create_pool;

use sqlx::{query, Row};

use chrono::Datelike;
use chrono::{Duration, NaiveDate, Utc};

use serde::Deserialize;

// =====================================
// CLI ARGS
// =====================================

#[derive(Parser, Debug)]
struct Args {
    // cargo run --bin ingester -- --bootstrap 5
    #[arg(long)]
    bootstrap: Option<i64>,
}

// =====================================
// NEW FORMAT (July 8 2024+)
// =====================================

#[derive(Debug, Deserialize)]
struct NewCandle {
    #[serde(rename = "TradDt")]
    trade_date: NaiveDate,

    #[serde(rename = "TckrSymb")]
    symbol: String,

    #[serde(rename = "SctySrs")]
    series: String,

    #[serde(rename = "OpnPric")]
    open_price: f64,

    #[serde(rename = "HghPric")]
    high_price: f64,

    #[serde(rename = "LwPric")]
    low_price: f64,

    #[serde(rename = "ClsPric")]
    close_price: f64,

    #[serde(rename = "TtlTradgVol")]
    volume: i64,
}

// =====================================
// OLD FORMAT (before July 8 2024)
// =====================================

#[derive(Debug, Deserialize)]
struct OldCandle {
    #[serde(rename = "SYMBOL")]
    symbol: String,

    #[serde(rename = "SERIES")]
    series: String,

    #[serde(rename = "OPEN")]
    open_price: f64,

    #[serde(rename = "HIGH")]
    high_price: f64,

    #[serde(rename = "LOW")]
    low_price: f64,

    #[serde(rename = "CLOSE")]
    close_price: f64,

    #[serde(rename = "TOTTRDQTY")]
    volume: i64,

    #[serde(rename = "TIMESTAMP")]
    trade_date: String, // "25-JUN-2021" — needs manual parsing
}

// =====================================
// INSERT ROW
// =====================================

async fn insert_candle(
    pool: &sqlx::Pool<sqlx::Postgres>,
    symbol: &str,
    series: &str,
    trade_date: NaiveDate,
    open_price: f64,
    high_price: f64,
    low_price: f64,
    close_price: f64,
    volume: i64,
) -> Result<()> {
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
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT DO NOTHING
        "#
    )
    .bind(symbol)
    .bind(series)
    .bind(trade_date)
    .bind(open_price)
    .bind(high_price)
    .bind(low_price)
    .bind(close_price)
    .bind(volume)
    .execute(pool)
    .await?;

    Ok(())
}

// =====================================
// INGEST SINGLE DAY
// =====================================

async fn ingest_day(
    pool: &sqlx::Pool<sqlx::Postgres>,
    client: &reqwest::Client,
    date: NaiveDate,
) -> Result<usize> {

    let date_str = date.format("%Y%m%d");

    let cutoff = NaiveDate::from_ymd_opt(2024, 7, 8).unwrap();

    let is_new_format = date >= cutoff;

    let url = if is_new_format {
        format!(
            "https://nsearchives.nseindia.com/content/cm/BhavCopy_NSE_CM_0_0_0_{}_F_0000.csv.zip",
            date_str
        )
    } else {
        let day = date.format("%d");
        let month = date.format("%b").to_string().to_uppercase();
        let year = date.format("%Y");
        format!(
            "https://nsearchives.nseindia.com/content/historical/EQUITIES/{}/{}/cm{}{}{}bhav.csv.zip",
            year, month, day, month, year
        )
    };

    println!("\n📡 Downloading {}", date_str);
    println!("🌐 URL: {}", url);

    let response = client
        .get(&url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .header("Referer", "https://www.nseindia.com")
        .header("Accept", "*/*")
        .send()
        .await?;

    println!("🌐 Status: {}", response.status());

    if !response.status().is_success() {
        println!("⚠️ Skipping {} (holiday/no data)", date_str);
        return Ok(0);
    }

    let bytes = response.bytes().await?;
    let reader = Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader)?;
    let file = archive.by_index(0)?;
    let mut csv_reader = Reader::from_reader(file);

    let mut inserted = 0;

    if is_new_format {

        for result in csv_reader.deserialize::<NewCandle>() {
            let c = result?;

            insert_candle(
                pool,
                &c.symbol,
                &c.series,
                c.trade_date,
                c.open_price,
                c.high_price,
                c.low_price,
                c.close_price,
                c.volume,
            ).await?;

            inserted += 1;
        }

    } else {

        for result in csv_reader.deserialize::<OldCandle>() {
            let c = result?;

            // old format date: "25-JUN-2021"
            let trade_date = NaiveDate::parse_from_str(
                c.trade_date.trim(),
                "%d-%b-%Y"
            )?;

            insert_candle(
                pool,
                &c.symbol,
                &c.series,
                trade_date,
                c.open_price,
                c.high_price,
                c.low_price,
                c.close_price,
                c.volume,
            ).await?;

            inserted += 1;
        }
    }

    println!("✅ Inserted {} rows for {}", inserted, date_str);

    Ok(inserted)
}

// =====================================
// GET LATEST TRADE DATE
// =====================================

async fn get_latest_trade_date(
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> Result<Option<NaiveDate>> {

    let row = sqlx::query(
        r#"
        SELECT MAX(trade_date) AS latest_date
        FROM daily_prices
        "#
    )
    .fetch_one(pool)
    .await?;

    let latest: Option<NaiveDate> =
        row.try_get("latest_date")?;

    Ok(latest)
}

// =====================================
// SYNC INSTRUMENTS
// =====================================

async fn sync_instruments(
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> Result<()> {

    println!("\n🔄 Syncing instruments...");

    query(
        r#"
        INSERT INTO instruments (symbol, series)
        SELECT DISTINCT symbol, series
        FROM daily_prices
        ON CONFLICT DO NOTHING
        "#
    )
    .execute(pool)
    .await?;

    println!("✅ Instruments synced");

    Ok(())
}

// =====================================
// MAIN
// =====================================

#[tokio::main]
async fn main() -> Result<()> {

    let args = Args::parse();

    dotenv().ok();

    let database_url = env::var("DATABASE_URL")?;

    let pool = create_pool(&database_url).await;

    println!("✅ Database Connected");

    let client = reqwest::Client::builder().build()?;

    // avoid partial current-day archives
    let today = Utc::now().date_naive() - Duration::days(1);

    let latest_date = get_latest_trade_date(&pool).await?;

    // =====================================
    // START DATE LOGIC
    // =====================================

    let start_date = if let Some(years) = args.bootstrap {
        println!("🚀 Bootstrap mode: {} years", years);
        today - Duration::days(years * 365)
    } else {
        match latest_date {
            Some(date) => {
                println!("📅 Latest DB date: {}", date);
                date + Duration::days(1)
            }
            None => {
                println!("⚠️ Empty DB detected, defaulting to last 30 days");
                today - Duration::days(30)
            }
        }
    };

    println!("🚀 Starting sync from {}", start_date);

    let mut current_date = start_date;
    let mut total_inserted = 0;

    while current_date <= today {

        if current_date.weekday().number_from_monday() >= 6 {
            println!("⏭️ Skipping weekend {}", current_date);
            current_date += Duration::days(1);
            continue;
        }

        match ingest_day(&pool, &client, current_date).await {
            Ok(inserted) => total_inserted += inserted,
            Err(err) => println!("❌ Failed {}: {}", current_date, err),
        }

        current_date += Duration::days(1);
    }

    // =====================================
    // SYNC INSTRUMENTS
    // =====================================

    sync_instruments(&pool).await?;

    println!("\n🎉 Sync Complete");
    println!("📦 Total Rows Inserted: {}", total_inserted);

    Ok(())
}