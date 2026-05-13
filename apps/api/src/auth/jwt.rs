use chrono::{
    Duration,
    Utc,
};

use jsonwebtoken::{
    encode,
    EncodingKey,
    Header,
};

use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, Serialize, Deserialize)]

pub struct Claims {

    pub sub: String,

    pub role: String,

    pub exp: usize,
}

pub fn generate_jwt(

    user_id: String,

    role: String,
) -> String {

    let expiration =

        Utc::now()

        + Duration::days(7);

    let claims = Claims {

        sub: user_id,

        role,

        exp: expiration.timestamp()
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