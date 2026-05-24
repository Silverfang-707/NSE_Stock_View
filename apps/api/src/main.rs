mod handlers;
mod models;
mod auth;

use axum::{
    middleware,

    routing::{
        delete,
        get,
        post,
    },

    Router,
};

use auth::middleware::admin_guard;

use dotenvy::dotenv;

use std::env;

use db::create_pool;

use handlers::{

    // =====================================
    // MARKET
    // =====================================

    admin::ingest_latest,

    backfill::backfill_data,

    candles::get_candles,

    symbols::get_symbols,

    analysis::get_analysis,

    series::get_series,

    levels::get_levels,

    // =====================================
    // AUTH
    // =====================================

    auth::{

        login,

        create_user,
    },

    // =====================================
    // ADMIN USERS
    // =====================================

    admin_users::{

        list_users,

        delete_user,
    },
};

use tower_http::cors::CorsLayer;

#[tokio::main]

async fn main() {

    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL")
            .expect(
                "DATABASE_URL missing"
            );

    let pool =
        create_pool(&database_url)
            .await;

    println!(
        "✅ Database Connected"
    );

    let app =
        Router::new()

            // =====================================
            // PUBLIC MARKET ROUTES
            // =====================================

            .route(
                "/symbols",
                get(get_symbols)
            )

            .route(
                "/series/{symbol}",
                get(get_series)
            )

            .route(
                "/series",
                get(get_series)
            )

            .route(
                "/candles/{symbol}",
                get(get_candles)
            )

            .route(
                "/analysis/{symbol}",
                get(get_analysis)
            )

            .route(
                "/levels/{symbol}",
                get(get_levels)
            )

            // =====================================
            // AUTH
            // =====================================

            .route(
                "/auth/login",
                post(login)
            )

            // =====================================
            // ADMIN
            // =====================================

            .nest(

                "/admin",

                Router::new()

                    // =========================
                    // MARKET OPS
                    // =========================

                    .route(
                        "/ingest",
                        get(ingest_latest)
                    )

                    .route(
                        "/backfill",
                        get(backfill_data)
                    )

                    // =========================
                    // USER MANAGEMENT
                    // =========================

                    .route(
                        "/create-user",
                        post(create_user)
                    )

                    .route(
                        "/users",
                        get(list_users)
                    )

                    .route(
                        "/users/{id}",
                        delete(delete_user)
                    )

                    // =========================
                    // AUTH MIDDLEWARE
                    // =========================

                    .layer(
                        middleware::from_fn(
                            admin_guard
                        )
                    )
            )

            // =====================================
            // GLOBAL LAYERS
            // =====================================

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