use anyhow::Result;
use dotenvy::dotenv;
use std::env;

use db::create_pool;
use sqlx::Row;

mod calculations;
mod models;
mod services;

use calculations::{
    range::calculate_range,
    buffer::calculate_buffer,
    jgd::calculate_jgd,
    jwd::calculate_jwd,
    patterns::{detect_pattern, update_bdp_wdp},
};

use models::{
    level::Level,
    timeframe::Timeframe,
};

use services::fetch::{
    fetch_symbols,
    fetch_latest_level_date,
    fetch_last_level,
    fetch_timeframe_ohlc,
};

use services::insert::insert_level;

// =====================================
// MAIN
// =====================================

#[tokio::main]
async fn main() -> Result<()> {

    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL")?;

    let pool =
        create_pool(&database_url).await;

    println!("✅ Calculator DB Connected");

    // =====================================
    // FETCH SYMBOLS
    // =====================================

    println!("\n📥 Fetching symbols...");

    let symbols =
        fetch_symbols(&pool).await;

    println!(
        "✅ Found {} symbols",
        symbols.len()
    );

    // =====================================
    // TIMEFRAMES
    // =====================================

    let timeframes = vec![
        Timeframe::Daily,
        Timeframe::Weekly,
        Timeframe::Monthly,
        Timeframe::Quarterly,
        Timeframe::Yearly,
    ];

    let mut total_processed = 0;

    // =====================================
    // PROCESS EACH SYMBOL
    // =====================================

    for row in &symbols {

        let symbol: String = row.get("symbol");
        let series: String = row.get("series");

        println!(
            "\n📊 Processing {} [{}]",
            symbol,
            series
        );

        // =================================
        // PROCESS EACH TIMEFRAME
        // =================================

        for timeframe in &timeframes {

            let tf_str    = timeframe.as_str();
            let trunc_str = timeframe.trunc_str();

            // =============================
            // INCREMENTAL: latest calculated
            // date per symbol/series/timeframe
            // =============================

            let latest_date =
                fetch_latest_level_date(
                    &pool,
                    &symbol,
                    &series,
                    tf_str,
                )
                .await;

            // =============================
            // CARRY FORWARD: prev jwd/bdp/wdp
            // for correct pattern on
            // incremental runs
            // =============================

            let last_level =
                fetch_last_level(
                    &pool,
                    &symbol,
                    &series,
                    tf_str,
                )
                .await;

            // =============================
            // FETCH ALL NEW OHLC PERIODS
            // =============================

            let ohlc_rows =
                fetch_timeframe_ohlc(
                    &pool,
                    &symbol,
                    &series,
                    trunc_str,
                    latest_date,
                )
                .await;

            if ohlc_rows.is_empty() {
                println!(
                    "   ✅ {} up to date",
                    tf_str
                );
                continue;
            }

            println!(
                "   ⏳ {} — {} new periods",
                tf_str,
                ohlc_rows.len()
            );

            // =============================
            // CARRY FORWARD PREV STATE
            // =============================

            let mut prev_jwd =
                last_level.map(|(jwd, _, _)| jwd);

            let mut prev_bdp =
                last_level.map(|(_, bdp, _)| bdp);

            // =============================
            // COMPUTE + INSERT EACH PERIOD
            // =============================

            for ohlc in &ohlc_rows {

                let range_value =
                    calculate_range(
                        ohlc.high_price,
                        ohlc.low_price,
                    );

                let buffer_value =
                    calculate_buffer(
                        ohlc.close_price,
                        range_value,
                    );

                let jgd =
                    calculate_jgd(
                        ohlc.high_price,
                        range_value,
                    );

                let jwd =
                    calculate_jwd(
                        ohlc.low_price,
                        range_value,
                    );

                // ===========================
                // PATTERN + BDP/WDP
                // first period has no prev
                // ===========================

                let (pattern, bdp, wdp) =
                    match prev_jwd {

                        Some(pjwd) => {

                            let pat =
                                detect_pattern(
                                    jgd,
                                    jwd,
                                    pjwd,
                                );

                            let (new_bdp, new_wdp) =
                                update_bdp_wdp(
                                    &pat,
                                    jgd,
                                    jwd,
                                    prev_bdp.unwrap_or(jgd),
                                );

                            (pat, new_bdp, new_wdp)
                        }

                        None => (
                            String::new(),
                            jgd,
                            jwd,
                        ),
                    };

                // ===========================
                // BUILD LEVEL
                // ===========================

                let level = Level {
                    symbol:       symbol.clone(),
                    series:       series.clone(),
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
                    pattern,
                };

                // ===========================
                // INSERT
                // ===========================

                insert_level(&pool, &level).await;

                // ===========================
                // ROLL FORWARD FOR NEXT PERIOD
                // ===========================

                prev_jwd = Some(jwd);
                prev_bdp = Some(bdp);

                total_processed += 1;
            }
        }
    }

    // =====================================
    // COMPLETE
    // =====================================

    println!("\n🎉 Calculation Complete");

    println!(
        "📦 Total Levels Generated: {}",
        total_processed
    );

    Ok(())
}