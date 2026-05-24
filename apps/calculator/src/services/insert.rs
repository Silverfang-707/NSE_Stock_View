use crate::models::level::Level;

pub async fn insert_level(

    pool: &sqlx::Pool<sqlx::Postgres>,

    level: &Level,
)
{

    sqlx::query(
        r#"
        INSERT INTO market_levels (

            symbol,

            series,

            timeframe,

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

            updated_at
        )

        VALUES (

            $1,

            $2,

            $3,

            $4,

            $5,

            $6,

            $7,

            $8,

            $9,

            $10,

            $11,

            $12,

            $13,

            $14,

            $15,

            NOW()
        )

        ON CONFLICT (

            symbol,

            series,

            timeframe,

            trade_date
        )

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

            updated_at = NOW()
        "#
    )

    .bind(&level.symbol)

    .bind(&level.series)

    .bind(&level.timeframe)

    .bind(level.trade_date)

    .bind(level.open_price)

    .bind(level.high_price)

    .bind(level.low_price)

    .bind(level.close_price)

    .bind(level.range_value)

    .bind(level.buffer_value)

    .bind(level.jgd)

    .bind(level.jwd)

    .bind(level.bdp)

    .bind(level.wdp)

    .bind(&level.pattern)

    .execute(pool)

    .await

    .unwrap();
}