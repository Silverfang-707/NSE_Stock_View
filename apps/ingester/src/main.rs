use anyhow::Result;
use dotenvy::dotenv;

use futures::stream::{self, StreamExt};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use std::env;
use std::io::Cursor;
use std::time::Duration;

use clap::Parser;
use csv::Reader;
use zip::ZipArchive;

use db::create_pool;

use sqlx::{query, Postgres, QueryBuilder, Row};

use chrono::Datelike;
use chrono::{NaiveDate, Utc};

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
// UNIFIED DATABASE STRUCT
// =====================================
#[derive(Debug)]
struct DbCandle {
    symbol: String,
    series: String,
    trade_date: NaiveDate,
    open_price: f64,
    high_price: f64,
    low_price: f64,
    close_price: f64,
    volume: i64,
}

// =====================================
// BULK INSERT
// =====================================

async fn insert_candles_bulk(
    pool: &sqlx::Pool<Postgres>,
    candles: &[DbCandle],
) -> Result<()> {
    if candles.is_empty() {
        return Ok(());
    }

    // Postgres parameter limit is 65535.
    // 3000 items * 8 fields = 24,000 parameters, which is well within safe limits.
    for chunk in candles.chunks(3000) {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO daily_prices (
                symbol, series, trade_date, open_price, 
                high_price, low_price, close_price, volume
            ) "
        );

        query_builder.push_values(chunk, |mut b, candle| {
            b.push_bind(&candle.symbol)
             .push_bind(&candle.series)
             .push_bind(candle.trade_date)
             .push_bind(candle.open_price)
             .push_bind(candle.high_price)
             .push_bind(candle.low_price)
             .push_bind(candle.close_price)
             .push_bind(candle.volume);
        });

        query_builder.push(
            r#"
            ON CONFLICT DO NOTHING
            "#
        );

        let query = query_builder.build();
        query.execute(pool).await?;
    }

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

    // 404 Not Found or 403 Forbidden usually indicates a genuine holiday on NSE archives
    if response.status() == reqwest::StatusCode::NOT_FOUND || response.status() == reqwest::StatusCode::FORBIDDEN {
        println!("⚠️ Skipping {} (holiday/missing file)", date_str);
        return Ok(0);
    } else if !response.status().is_success() {
        // Bail out on actual server failures or explicit rate limits (429/500)
        anyhow::bail!("NSE Server rejected request with status: {}", response.status());
    }

    let bytes = response.bytes().await?;
    let reader = Cursor::new(bytes);
    let mut archive = ZipArchive::new(reader)?;
    let file = archive.by_index(0)?;
    let mut csv_reader = Reader::from_reader(file);

    let mut candles_to_insert = Vec::new();

    if is_new_format {
        for result in csv_reader.deserialize::<NewCandle>() {
            let c = match result {
                Ok(candle) => candle,
                Err(e) => {
                    println!("⚠️ Skipping malformed row: {}", e);
                    continue;
                }
            };

            candles_to_insert.push(DbCandle {
                symbol: c.symbol,
                series: c.series,
                trade_date: c.trade_date,
                open_price: c.open_price,
                high_price: c.high_price,
                low_price: c.low_price,
                close_price: c.close_price,
                volume: c.volume,
            });
        }
    } else {
        for result in csv_reader.deserialize::<OldCandle>() {
            let c = match result {
                Ok(candle) => candle,
                Err(e) => {
                    println!("⚠️ Skipping malformed row: {}", e);
                    continue;
                }
            };

            // Parse the old format date safely
            let trade_date = match NaiveDate::parse_from_str(c.trade_date.trim(), "%d-%b-%Y") {
                Ok(d) => d,
                Err(e) => {
                    println!("⚠️ Skipping row due to bad date format {}: {}", c.trade_date, e);
                    continue;
                }
            };

            candles_to_insert.push(DbCandle {
                symbol: c.symbol,
                series: c.series,
                trade_date,
                open_price: c.open_price,
                high_price: c.high_price,
                low_price: c.low_price,
                close_price: c.close_price,
                volume: c.volume,
            });
        }
    }

    let inserted = candles_to_insert.len();
    if inserted > 0 {
        insert_candles_bulk(pool, &candles_to_insert).await?;
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

// =====================================
// MAIN (PARALLELIZED)
// =====================================

#[tokio::main]
async fn main() -> Result<()> {

    let args = Args::parse();
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")?;
    let pool = create_pool(&database_url).await;

    println!("✅ Database Connected");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let today = Utc::now().date_naive() - chrono::Duration::days(1);
    let latest_date = get_latest_trade_date(&pool).await?;

    let start_date = if let Some(years) = args.bootstrap {
        println!("🚀 Bootstrap mode: {} years", years);
        today - chrono::Duration::days(years * 365)
    } else {
        match latest_date {
            Some(date) => {
                println!("📅 Latest DB date: {}", date);
                date + chrono::Duration::days(1)
            }
            None => {
                println!("⚠️ Empty DB detected, defaulting to last 30 days");
                today - chrono::Duration::days(30)
            }
        }
    };

    println!("🚀 Starting sync from {}", start_date);

    // 1. Build a list of valid weekdays to fetch
    let mut dates_to_fetch = Vec::new();
    let mut current_date = start_date;

    while current_date <= today {
        if current_date.weekday().number_from_monday() < 6 {
            dates_to_fetch.push(current_date);
        }
        current_date += chrono::Duration::days(1);
    }

    println!("📅 Total trading days to evaluate: {}", dates_to_fetch.len());

    // 2. Set up thread-safe counter
    let total_inserted = Arc::new(AtomicUsize::new(0));

    // 3. Process the dates concurrently
    // ⚠️ CRITICAL: Do not set this concurrency limit higher than 3-5 for NSE!
    let concurrency_limit = 4; 

    stream::iter(dates_to_fetch)
        .for_each_concurrent(concurrency_limit, |date| {
            // Clone references for the async block
            let pool = pool.clone(); // sqlx pools are cheap to clone (they are Arcs under the hood)
            let client = client.clone();
            let total = Arc::clone(&total_inserted);

            async move {
                match ingest_day(&pool, &client, date).await {
                    Ok(inserted) => {
                        total.fetch_add(inserted, Ordering::Relaxed);
                    }
                    Err(err) => println!("❌ Failed {}: {}", date, err),
                }
            }
        })
        .await;

    // =====================================
    // SYNC INSTRUMENTS
    // =====================================

    sync_instruments(&pool).await?;

    println!("\n🎉 Sync Complete");
    println!("📦 Total Rows Inserted: {}", total_inserted.load(Ordering::Relaxed));

    Ok(())
}