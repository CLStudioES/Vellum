use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::middleware::issue_token;
use crate::AppState;

const MAX_USERNAME_LEN: usize = 32;
const MAX_PASSWORD_LEN: usize = 128;
const MIN_PASSWORD_LEN: usize = 8;

#[derive(Deserialize)]
pub struct AuthPayload {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
    pub username: String,
}

fn validate_auth_input(payload: &AuthPayload) -> Result<(), (StatusCode, String)> {
    let username = payload.username.trim();
    if username.is_empty() || username.len() > MAX_USERNAME_LEN {
        return Err((StatusCode::BAD_REQUEST, format!("Username must be 1-{} characters", MAX_USERNAME_LEN)));
    }
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err((StatusCode::BAD_REQUEST, "Username can only contain letters, numbers, hyphens, and underscores".into()));
    }
    if payload.password.len() < MIN_PASSWORD_LEN || payload.password.len() > MAX_PASSWORD_LEN {
        return Err((StatusCode::BAD_REQUEST, format!("Password must be {}-{} characters", MIN_PASSWORD_LEN, MAX_PASSWORD_LEN)));
    }
    Ok(())
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    validate_auth_input(&payload)?;

    let username = payload.username.trim().to_lowercase();
    let password_hash = hash_password(&payload.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let user = sqlx::query_as::<_, crate::models::User>(
        "INSERT INTO users (username, password_hash) VALUES ($1, $2) RETURNING *"
    )
    .bind(&username)
    .bind(&password_hash)
    .fetch_one(&state.db)
    .await
    .map_err(|_| (StatusCode::CONFLICT, "Username unavailable".into()))?;

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
    validate_auth_input(&payload)?;

    let username = payload.username.trim().to_lowercase();

    let user = sqlx::query_as::<_, crate::models::User>(
        "SELECT * FROM users WHERE username = $1"
    )
    .bind(&username)
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
