use sqlx::{
    postgres::PgRow,
    Row,
};

pub async fn fetch_daily_prices(

    pool: &sqlx::Pool<sqlx::Postgres>

)
-> Vec<PgRow>
{

    sqlx::query(
        r#"
        SELECT

            symbol,
            series,
            trade_date,

            open_price,
            high_price,
            low_price,
            close_price

        FROM daily_prices

        ORDER BY trade_date ASC
        "#
    )

    .fetch_all(pool)

    .await

    .unwrap()
}