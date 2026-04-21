use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub username: String,
    pub exp: i64,
}

pub fn issue_token(secret: &str, user_id: Uuid, username: &str) -> Result<String, StatusCode> {
    let claims = Claims {
        sub: user_id,
        username: username.to_string(),
        exp: (Utc::now() + chrono::Duration::hours(24)).timestamp(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn verify_token(secret: &str, token: &str) -> Result<Claims, StatusCode> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| StatusCode::UNAUTHORIZED)
}

/// Extractor: pulls Claims from the Authorization header.
impl<S: Send + Sync> FromRequestParts<S> for Claims {
    type Rejection = StatusCode;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            let secret = parts
                .extensions
                .get::<JwtSecret>()
                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

            let header = parts
                .headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .ok_or(StatusCode::UNAUTHORIZED)?;

            let token = header
                .strip_prefix("Bearer ")
                .ok_or(StatusCode::UNAUTHORIZED)?;

            verify_token(&secret.0, token)
        }
    }
}

#[derive(Clone)]
pub struct JwtSecret(pub String);
