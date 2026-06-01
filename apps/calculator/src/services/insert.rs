use crate::models::level::Level;
use sqlx::{Postgres, QueryBuilder};
use chrono::Utc;

pub async fn insert_levels_bulk(
    pool: &sqlx::Pool<Postgres>,
    levels: &[Level],
) {
    if levels.is_empty() {
        return;
    }

    // Postgres parameter limit prevents enormous single transactions.
    // 3500 items * 16 fields stays safely under the limit.
    for chunk in levels.chunks(3500) {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO market_levels (
                symbol, series, timeframe, trade_date, open_price, 
                high_price, low_price, close_price, range_value, 
                buffer_value, jgd, jwd, bdp, wdp, pattern, updated_at
            ) "
        );

        let now = Utc::now().naive_utc();

        query_builder.push_values(chunk, |mut b, level| {
            b.push_bind(&level.symbol)
             .push_bind(&level.series)
             .push_bind(&level.timeframe)
             .push_bind(level.trade_date)
             .push_bind(level.open_price)
             .push_bind(level.high_price)
             .push_bind(level.low_price)
             .push_bind(level.close_price)
             .push_bind(level.range_value)
             .push_bind(level.buffer_value)
             .push_bind(level.jgd)
             .push_bind(level.jwd)
             .push_bind(level.bdp)
             .push_bind(level.wdp)
             .push_bind(&level.pattern)
             .push_bind(now);
        });

        query_builder.push(
            r#"
            ON CONFLICT (symbol, series, timeframe, trade_date) 
            DO UPDATE SET 
                open_price = EXCLUDED.open_price,
                high_price = EXCLUDED.high_price,
                low_price = EXCLUDED.low_price,
                close_price = EXCLUDED.close_price,
                range_value = EXCLUDED.range_value,
                buffer_value = EXCLUDED.buffer_value,
                jgd = EXCLUDED.jgd,
                jwd = EXCLUDED.jwd,
                bdp = EXCLUDED.bdp,
                wdp = EXCLUDED.wdp,
                pattern = EXCLUDED.pattern,
                updated_at = EXCLUDED.updated_at
            "#
        );

        let query = query_builder.build();
        query.execute(pool).await.expect("Failed to execute bulk insert");
    }
}