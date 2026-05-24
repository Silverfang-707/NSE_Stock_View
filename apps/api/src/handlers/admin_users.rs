use axum::{
    extract::{
        Path,
        State,
    },

    Json,
};

use serde::Serialize;

use sqlx::Row;

// =====================================
// USER RESPONSE
// =====================================

#[derive(Serialize)]

pub struct UserResponse {

    id: i32,

    username: String,

    role: String,

    is_root: bool,
}

// =====================================
// LIST USERS
// =====================================

pub async fn list_users(

    State(pool): State<
        sqlx::Pool<sqlx::Postgres>
    >,
)
-> Json<Vec<UserResponse>>
{

    let rows =
        sqlx::query(
            r#"
            SELECT

                id,

                username,

                role,

                is_root

            FROM users

            ORDER BY id
            "#
        )

        .fetch_all(&pool)

        .await

        .unwrap();

    let users =
        rows
            .into_iter()

            .map(|row| {

                UserResponse {

                    id:
                        row.get("id"),

                    username:
                        row.get("username"),

                    role:
                        row.get("role"),

                    is_root:
                        row.get("is_root"),
                }
            })

            .collect();

    Json(users)
}

// =====================================
// DELETE USER
// =====================================

pub async fn delete_user(

    Path(id): Path<i32>,

    State(pool): State<
        sqlx::Pool<sqlx::Postgres>
    >,
)
-> Json<serde_json::Value>
{

    let row =
        sqlx::query(
            r#"
            SELECT

                is_root

            FROM users

            WHERE id = $1
            "#
        )

        .bind(id)

        .fetch_one(&pool)

        .await

        .unwrap();

    let is_root: bool =
        row.get("is_root");

    if is_root {

        return Json(
            serde_json::json!({

                "success": false,

                "error":
                    "Root admin cannot be deleted"
            })
        );
    }

    sqlx::query(
        r#"
        DELETE FROM users

        WHERE id = $1
        "#
    )

    .bind(id)

    .execute(&pool)

    .await

    .unwrap();

    Json(
        serde_json::json!({

            "success": true
        })
    )
}