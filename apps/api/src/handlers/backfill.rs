use axum::{
    extract::{
        Query,
        State,
    },
    Json,
};

use chrono::{
    Datelike,
    Duration,
    NaiveDate,
};

use serde::Deserialize;

use market_core::ingest::ingest_day;

#[derive(Deserialize)]

pub struct BackfillParams {

    pub year: Option<i32>,

    pub from: Option<String>,

    pub to: Option<String>,
}

pub async fn backfill_data(

    Query(params): Query<BackfillParams>,

    State(pool): State<
        sqlx::Pool<sqlx::Postgres>
    >,
)
-> Json<serde_json::Value>
{

    let mut inserted = 0;

    let (start, end) =

        if let Some(year) =
            params.year
        {

            (
                NaiveDate::from_ymd_opt(
                    year,
                    1,
                    1
                )
                .unwrap(),

                NaiveDate::from_ymd_opt(
                    year,
                    12,
                    31
                )
                .unwrap(),
            )

        } else {

            let start =
                params
                    .from
                    .as_ref()

                    .and_then(|d|

                        NaiveDate::parse_from_str(
                            d,
                            "%Y-%m-%d"
                        )
                        .ok()
                    )
                    .unwrap();

            let end =
                params
                    .to
                    .as_ref()

                    .and_then(|d|

                        NaiveDate::parse_from_str(
                            d,
                            "%Y-%m-%d"
                        )
                        .ok()
                    )
                    .unwrap();

            (start, end)
        };

    let mut current = start;

    while current <= end {

        // Skip weekends
        if current
            .weekday()
            .number_from_monday()
            < 6
        {

            match ingest_day(
                &pool,
                current
            )
            .await
            {

                Ok(rows) => {

                    inserted += rows;

                    println!(
                        "✅ {} -> {} rows",
                        current,
                        rows
                    );
                }

                Err(err) => {

                    println!(
                        "❌ {} -> {}",
                        current,
                        err
                    );
                }
            }
        }

        current += Duration::days(1);
    }

    Json(
        serde_json::json!({

            "success": true,

            "rows_inserted":
                inserted,

            "from":
                start.to_string(),

            "to":
                end.to_string(),
        })
    )
}