use anyhow::Result;
use dotenvy::dotenv;
use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use futures::stream::{self, StreamExt};

use db::create_pool;

mod calculations;
mod models;
mod services;

use calculations::{
    buffer::calculate_buffer, jgd::calculate_jgd, jwd::calculate_jwd, patterns::{detect_pattern, update_bdp_wdp}, range::calculate_range,
};

use models::{
    level::Level,
    timeframe::Timeframe,
};

use services::fetch::{
    fetch_symbols, fetch_previous_state, fetch_timeframe_ohlc, SymbolSeries,
};
use services::insert::insert_levels_bulk;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")?;
    let pool = create_pool(&database_url).await;
    
    let start_time = Instant::now();

    println!("✅ Calculator DB Connected");
    println!("⏱️ Starting benchmark clock...");
    println!("\n📥 Fetching symbols...");

    let symbols = fetch_symbols(&pool).await;
    println!("✅ Found {} symbols", symbols.len());

    let timeframes = vec![
        Timeframe::Daily,
        Timeframe::Weekly,
        Timeframe::Monthly,
        Timeframe::Quarterly,
        Timeframe::HalfYearly,
        Timeframe::Yearly,
    ];

    let total_processed = Arc::new(AtomicUsize::new(0));

    // =====================================
    // CONCURRENT PROCESSING
    // Starting safe at 4-6 concurrent streams
    // =====================================
    stream::iter(symbols)
        .for_each_concurrent(6, |sym_data| {
            let pool = pool.clone();
            let timeframes = timeframes.clone();
            let total_processed = Arc::clone(&total_processed);

            async move {
                process_symbol(&pool, sym_data, &timeframes, total_processed).await;
            }
        })
        .await;

    println!("\n🎉 Calculation Complete");
    println!("📦 Total Levels Generated: {}", total_processed.load(Ordering::Relaxed));
    println!("⏱️ Total Execution Time: {:.2?}", start_time.elapsed());

    Ok(())
}

// =====================================
// PROCESS SINGLE SYMBOL
// =====================================
async fn process_symbol(
    pool: &sqlx::Pool<sqlx::Postgres>,
    sym_data: SymbolSeries,
    timeframes: &[Timeframe],
    total_processed: Arc<AtomicUsize>,
) {
    for timeframe in timeframes {
        let tf_str = timeframe.as_str();
        let trunc_str = timeframe.trunc_str();

        let prev_state = fetch_previous_state(pool, &sym_data.symbol, &sym_data.series, tf_str).await;
        
        let latest_date = prev_state.map(|(date, _, _, _)| date);
        let mut prev_jwd = prev_state.map(|(_, jwd, _, _)| jwd);
        let mut prev_bdp = prev_state.map(|(_, _, bdp, _)| bdp);

        let ohlc_rows = fetch_timeframe_ohlc(
            pool,
            &sym_data.symbol,
            &sym_data.series,
            trunc_str,
            latest_date,
        )
        .await;

        if ohlc_rows.is_empty() {
            continue;
        }

        let mut levels_to_insert = Vec::with_capacity(ohlc_rows.len());

        for ohlc in &ohlc_rows {
            let range_value = calculate_range(ohlc.high_price, ohlc.low_price);
            let buffer_value = calculate_buffer(ohlc.close_price, range_value);
            let jgd = calculate_jgd(ohlc.high_price, range_value);
            let jwd = calculate_jwd(ohlc.low_price, range_value);

            let (pattern_enum, bdp, wdp) = match prev_jwd {
                Some(pjwd) => {
                    let pat = detect_pattern(jgd, jwd, pjwd);
                    let (new_bdp, new_wdp) = update_bdp_wdp(pat, jgd, jwd, prev_bdp.unwrap_or(jgd));
                    (pat, new_bdp, new_wdp)
                }
                None => (calculations::patterns::Pattern::None, jgd, jwd), 
            };

            levels_to_insert.push(Level {
                symbol:       sym_data.symbol.clone(),
                series:       sym_data.series.clone(),
                timeframe:    tf_str.to_string(),
                trade_date:   ohlc.trade_date,
                open_price:   ohlc.open_price,
                high_price:   ohlc.high_price,
                low_price:    ohlc.low_price,
                close_price:  ohlc.close_price,
                range_value,
                buffer_value,
                jgd,
                jwd,
                bdp,
                wdp,
                pattern:      pattern_enum.as_str().to_string(),
            });

            prev_jwd = Some(jwd);
            prev_bdp = Some(bdp);
        }

        if !levels_to_insert.is_empty() {
            let count = levels_to_insert.len();
            insert_levels_bulk(pool, &levels_to_insert).await;
            total_processed.fetch_add(count, Ordering::Relaxed);
            
            // Reduced terminal spam: only prints when actively inserting
            println!("✅ {} [{}] {} — {} periods inserted", sym_data.symbol, sym_data.series, tf_str, count);
        }
    }
}