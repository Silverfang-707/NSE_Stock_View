use axum::{
    extract::{
        Path,
        State,
    },
    Json,
};

use sqlx::Row;

use serde::Serialize;

#[derive(Serialize)]

pub struct LevelResponse {

    trade_date: chrono::NaiveDate,

    jgd: Option<f64>,

    jwd: Option<f64>,

    bdp: Option<f64>,

    wdp: Option<f64>,

    range_value: Option<f64>,

    buffer_value: Option<f64>,

    pattern: Option<String>,
}

pub async fn get_levels(

    Path(symbol): Path<String>,

    State(pool): State<
        sqlx::Pool<sqlx::Postgres>
    >,
)
-> Json<Vec<LevelResponse>>
{

    println!(
        "📊 Fetching levels for {}",
        symbol
    );

    let rows =
        sqlx::query(
            r#"
            SELECT

                trade_date,

                jgd,
                jwd,

                bdp,
                wdp,

                range_value,
                buffer_value,

                pattern

            FROM daily_levels

            WHERE symbol = $1

            ORDER BY trade_date DESC

            LIMIT 30
            "#
        )

        .bind(&symbol)

        .fetch_all(&pool)

        .await

        .unwrap();

    let levels =
        rows
            .into_iter()

            .map(|row| {

                LevelResponse {

                    trade_date:
                        row.get("trade_date"),

                    jgd:
                        row.get("jgd"),

                    jwd:
                        row.get("jwd"),

                    bdp:
                        row.get("bdp"),

                    wdp:
                        row.get("wdp"),

                    range_value:
                        row.get("range_value"),

                    buffer_value:
                        row.get("buffer_value"),

                    pattern:
                        row.get("pattern"),
                }
            })

            .collect();

    Json(levels)
}