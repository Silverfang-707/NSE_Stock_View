use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use jsonwebtoken::{
    decode,
    DecodingKey,
    Validation,
};

use crate::auth::jwt::Claims;

pub async fn admin_guard(

    request: Request,

    next: Next,
) -> Result<Response, StatusCode>
{

    let auth_header =

        request
            .headers()
            .get("Authorization")

            .and_then(|h|
                h.to_str().ok()
            );

    let Some(auth_header) = auth_header
    else {

        return Err(
            StatusCode::UNAUTHORIZED
        );
    };

    if !auth_header.starts_with("Bearer ")
    {

        return Err(
            StatusCode::UNAUTHORIZED
        );
    }

    let token =
        auth_header
            .trim_start_matches(
                "Bearer "
            );

    let secret =
        std::env::var(
            "JWT_SECRET"
        )
        .unwrap();

    let decoded =

        decode::<Claims>(

            token,

            &DecodingKey::from_secret(
                secret.as_bytes()
            ),

            &Validation::default(),
        );

    let Ok(token_data) = decoded
    else {

        return Err(
            StatusCode::UNAUTHORIZED
        );
    };

    if token_data.claims.role != "admin"
    {

        return Err(
            StatusCode::FORBIDDEN
        );
    }

    Ok(
        next.run(request).await
    )
}