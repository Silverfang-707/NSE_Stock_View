use crate::models::level::Level;

pub async fn insert_level(

    pool: &sqlx::Pool<sqlx::Postgres>,

    level: &Level,

)
{

    sqlx::query(
        r#"
        INSERT INTO daily_levels (

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

            pattern

        )

        VALUES (

            $1, $2, $3,

            $4, $5, $6, $7,

            $8, $9,

            $10, $11,

            $12, $13,

            $14
        )

        ON CONFLICT DO NOTHING
        "#
    )

    .bind(&level.symbol)

    .bind(&level.series)

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