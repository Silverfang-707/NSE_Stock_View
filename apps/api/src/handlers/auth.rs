use axum::{
    extract::State,
    Json,
};

use serde::Deserialize;

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

// =====================================
// CREATE USER REQUEST
// =====================================

#[derive(Deserialize)]

pub struct CreateUserRequest {

    pub username: String,

    pub password: String,

    pub role: String,
}

// =====================================
// LOGIN
// =====================================

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

                role,

                is_root

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

    let user_id: i32 =
        user.get("id");

    let role: String =
        user.get("role");

    let is_root: bool =
        user.get("is_root");

    let token =
        generate_jwt(

        user_id.to_string(),

        role.clone(),

        is_root
    );

    Json(
        serde_json::json!({

            "success": true,

            "token": token,

            "role": role,

            "is_root": is_root
        })
    )
}

// =====================================
// CREATE USER
// =====================================

pub async fn create_user(

    State(pool): State<
        sqlx::Pool<sqlx::Postgres>
    >,

    Json(payload): Json<CreateUserRequest>,
)
-> Json<serde_json::Value>
{

    let existing =
        sqlx::query(
            r#"
            SELECT id

            FROM users

            WHERE username = $1
            "#
        )

        .bind(&payload.username)

        .fetch_optional(&pool)

        .await

        .unwrap();

    if existing.is_some() {

        return Json(
            serde_json::json!({

                "success": false,

                "error":
                    "Username already exists"
            })
        );
    }

    let hashed_password =
        hash_password(
            &payload.password
        );

    let is_admin =
        payload.role == "admin";

    sqlx::query(
        r#"
        INSERT INTO users (

            username,

            password_hash,

            role,

            is_admin,

            is_root

        )

        VALUES (

            $1,

            $2,

            $3,

            $4,

            FALSE
        )
        "#
    )

    .bind(&payload.username)

    .bind(&hashed_password)

    .bind(&payload.role)

    .bind(is_admin)

    .execute(&pool)

    .await

    .unwrap();

    Json(
        serde_json::json!({

            "success": true
        })
    )
}