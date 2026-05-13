use axum::{
    extract::State,
    Json,
};

use sqlx::Row;

use crate::{
    auth::{
        jwt::generate_jwt,

        password::{
            hash_password,
            verify_password,
        },
    },

    models::user::LoginRequest,
};

pub async fn create_admin()
-> Json<serde_json::Value>
{

    let password =
        "admin123";

    let hash =
        hash_password(password);

    Json(
        serde_json::json!({

            "username": "admin",

            "password": password,

            "password_hash": hash
        })
    )
}

pub async fn login(

    State(pool): State<
        sqlx::Pool<sqlx::Postgres>
    >,

    Json(payload): Json<LoginRequest>,
)
-> Json<serde_json::Value>
{

    let row =
        sqlx::query(
            r#"
            SELECT
                id,
                password_hash,
                role
            FROM users
            WHERE username = $1
            "#
        )
        .bind(&payload.username)

        .fetch_optional(&pool)

        .await

        .unwrap();

    let Some(user) = row else {

        return Json(
            serde_json::json!({

                "success": false,

                "error":
                    "Invalid credentials"
            })
        );
    };

    let hash: String =
        user.get("password_hash");

    let valid =
        verify_password(
            &hash,
            &payload.password
        );

    if !valid {

        return Json(
            serde_json::json!({

                "success": false,

                "error":
                    "Invalid credentials"
            })
        );
    }

    let user_id: uuid::Uuid =
        user.get("id");

    let role: String =
        user.get("role");

    let token =
        generate_jwt(
            user_id.to_string(),
            role.clone()
        );

    Json(
        serde_json::json!({

            "success": true,

            "token": token,

            "role": role
        })
    )
}