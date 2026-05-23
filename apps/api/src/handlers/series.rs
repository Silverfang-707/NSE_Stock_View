use axum::{
    extract::{
        Path,
        State,
    },
    Json,
};

use sqlx::Row;

pub async fn get_series(

    Path(symbol): Path<String>,

    State(pool): State<
        sqlx::Pool<sqlx::Postgres>
    >,
)
-> Json<Vec<String>>
{

    println!(
        "📂 Requested symbol: {}",
        symbol
    );

    let rows =
        sqlx::query(
            r#"
            SELECT DISTINCT series

            FROM instruments

            WHERE TRIM(
                UPPER(symbol)
            ) = TRIM(
                UPPER($1)
            )

            AND series IS NOT NULL

            ORDER BY series
            "#
        )

        .bind(&symbol)

        .fetch_all(&pool)

        .await

        .unwrap();

    println!(
        "✅ Series found: {}",
        rows.len()
    );

    let series =
        rows
            .into_iter()
            .map(|row| {

                row.get::<String, _>(
                    "series"
                )

            })

            .collect();

    Json(series)
}