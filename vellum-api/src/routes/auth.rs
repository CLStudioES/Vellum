use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::middleware::issue_token;
use crate::AppState;

#[derive(Deserialize)]
pub struct AuthPayload {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
    pub username: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    if payload.username.trim().is_empty() || payload.password.len() < 8 {
        return Err((StatusCode::BAD_REQUEST, "Username required, password min 8 chars".into()));
    }

    let password_hash = hash_password(&payload.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let user = sqlx::query_as::<_, crate::models::User>(
        "INSERT INTO users (username, password_hash) VALUES ($1, $2) RETURNING *"
    )
    .bind(&payload.username)
    .bind(&password_hash)
    .fetch_one(&state.db)
    .await
    .map_err(|_| (StatusCode::CONFLICT, "Username already taken".into()))?;

    let token = issue_token(&state.jwt_secret.0, user.id, &user.username)
        .map_err(|s| (s, "Token generation failed".into()))?;

    Ok(Json(AuthResponse {
        token,
        user_id: user.id.to_string(),
        username: user.username,
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let user = sqlx::query_as::<_, crate::models::User>(
        "SELECT * FROM users WHERE username = $1"
    )
    .bind(&payload.username)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::UNAUTHORIZED, "Invalid credentials".into()))?;

    if !verify_password(&payload.password, &user.password_hash) {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".into()));
    }

    let token = issue_token(&state.jwt_secret.0, user.id, &user.username)
        .map_err(|s| (s, "Token generation failed".into()))?;

    Ok(Json(AuthResponse {
        token,
        user_id: user.id.to_string(),
        username: user.username,
    }))
}

fn hash_password(password: &str) -> Result<String, String> {
    use argon2::password_hash::{rand_core::OsRng, SaltString};
    use argon2::{Argon2, PasswordHasher};

    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

fn verify_password(password: &str, hash: &str) -> bool {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    PasswordHash::new(hash)
        .map(|parsed| Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok())
        .unwrap_or(false)
}
