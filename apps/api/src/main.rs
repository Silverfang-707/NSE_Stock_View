mod handlers;
mod models;
mod auth;

use axum::{
    routing::get,
    routing::post,
    Router,
    middleware,
};

use auth::middleware::admin_guard;

use dotenvy::dotenv;

use std::env;

use db::create_pool;

use handlers::{
    admin::ingest_latest,

    backfill::backfill_data,

    candles::get_candles,

    symbols::get_symbols,

    analysis::get_analysis,

    auth::{
        create_admin,
        login,
    },
};

use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {

    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL")
            .expect("DATABASE_URL missing");

    let pool =
        create_pool(&database_url).await;

    println!("✅ Database Connected");

    let app =
        Router::new()

            .route(
                "/symbols/{series}",
                get(get_symbols)
            )

            .route(
                "/auth/create-admin",
                get(create_admin)
            )

            .route(
                "/auth/login",
                post(login)
            )

            .route(
                "/candles/{symbol}",
                get(get_candles)
            )

            .route(
                "/analysis/{symbol}",
                get(get_analysis)
            )

            .nest(

                "/admin",

                Router::new()

                    .route(
                        "/ingest",
                        get(ingest_latest)
                    )

                    .route(
                        "/backfill",
                        get(backfill_data)
                    )

                    .layer(
                        middleware::from_fn(
                            admin_guard
                        )
                    )
            )

            .layer(
                CorsLayer::permissive()
            )

            .with_state(pool);

    let listener =
        tokio::net::TcpListener::bind(
            "0.0.0.0:3000"
        )
        .await
        .unwrap();

    println!(
        "🚀 API running on http://localhost:3000"
    );

    axum::serve(
        listener,
        app.into_make_service()
    )
    .await
    .unwrap();
}