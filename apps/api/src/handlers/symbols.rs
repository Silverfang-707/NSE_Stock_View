use axum::{
    extract::Path,
    Json,
};

pub async fn get_symbols(

    Path(series): Path<String>,

    axum::extract::State(
        pool
    ): axum::extract::State<
        sqlx::Pool<sqlx::Postgres>
    >
) -> Json<Vec<String>> {

    let rows: Vec<(String,)> =
        sqlx::query_as(
            r#"
            SELECT DISTINCT symbol
            FROM daily_prices
            WHERE series = $1
            ORDER BY symbol ASC
            "#
        )
        .bind(series)
        .fetch_all(&pool)
        .await
        .unwrap();

    Json(
        rows
            .into_iter()
            .map(|r| r.0)
            .collect()
    )
}