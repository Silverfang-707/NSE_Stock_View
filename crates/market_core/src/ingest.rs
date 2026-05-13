use anyhow::Result;

use std::io::Cursor;

use chrono::NaiveDate;

use csv::Reader;

use sqlx::{
    query,
    Pool,
    Postgres,
};

use zip::ZipArchive;

use crate::Candle;

pub async fn ingest_day(
    pool: &Pool<Postgres>,
    date: NaiveDate,
) -> Result<usize> {

    let date_str =
        date.format("%Y%m%d");

    let url = format!(
        "https://nsearchives.nseindia.com/content/cm/BhavCopy_NSE_CM_0_0_0_{}_F_0000.csv.zip",
        date_str
    );

    println!("📡 Downloading {}", date_str);

    let response =
        reqwest::get(&url).await?;

    if !response.status().is_success() {

        println!("⚠️ No data for {}", date_str);

        return Ok(0);
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

    Ok(inserted)
}