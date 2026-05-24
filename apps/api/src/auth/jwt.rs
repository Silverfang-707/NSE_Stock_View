use chrono::{
    Duration,
    Utc,
};

use jsonwebtoken::{

    decode,
    encode,

    DecodingKey,
    EncodingKey,

    Header,
    Validation,
};

use serde::{

    Deserialize,
    Serialize,
};

// =====================================
// CLAIMS
// =====================================

#[derive(Debug, Serialize, Deserialize)]

pub struct Claims {

    pub sub: String,

    pub role: String,

    pub is_root: bool,

    pub exp: usize,
}

// =====================================
// GENERATE JWT
// =====================================

pub fn generate_jwt(

    user_id: String,

    role: String,

    is_root: bool,
)
-> String
{

    let expiration =

        Utc::now()

        + Duration::days(7);

    let claims = Claims {

        sub: user_id,

        role,

        is_root,

        exp:
            expiration.timestamp()
            as usize,
    };

    let secret =

        std::env::var(
            "JWT_SECRET"
        )

        .expect(
            "JWT_SECRET missing"
        );

    encode(

        &Header::default(),

        &claims,

        &EncodingKey::from_secret(
            secret.as_bytes()
        ),
    )

    .unwrap()
}

// =====================================
// DECODE JWT
// =====================================

pub fn decode_jwt(
    token: &str
)
-> Result<
    Claims,

    jsonwebtoken::errors::Error
>
{

    let secret =

        std::env::var(
            "JWT_SECRET"
        )

        .expect(
            "JWT_SECRET missing"
        );

    let decoded =

        decode::<Claims>(

            token,

            &DecodingKey::from_secret(
                secret.as_bytes()
            ),

            &Validation::default(),
        )?;

    Ok(decoded.claims)
}