use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::middleware::Claims;
use crate::models::Role;
use crate::AppState;

const MAX_ENTRIES_PER_REQUEST: usize = 500;
const MAX_KEY_LEN: usize = 256;
const MAX_VALUE_LEN: usize = 65_536;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryPayload {
    pub env_file: String,
    pub key: String,
    pub encrypted_value: String,
    pub is_sensitive: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryResponse {
    pub id: Uuid,
    pub env_file: String,
    pub key: String,
    pub encrypted_value: String,
    pub is_sensitive: bool,
}

pub async fn upsert_entries(
    State(state): State<AppState>,
    claims: Claims,
    Path(project_id): Path<Uuid>,
    Json(entries): Json<Vec<EntryPayload>>,
) -> Result<StatusCode, (StatusCode, String)> {
    if entries.len() > MAX_ENTRIES_PER_REQUEST {
        return Err((StatusCode::BAD_REQUEST, format!("Max {} entries per request", MAX_ENTRIES_PER_REQUEST)));
    }
    for entry in &entries {
        if entry.key.len() > MAX_KEY_LEN || entry.encrypted_value.len() > MAX_VALUE_LEN || entry.env_file.len() > MAX_KEY_LEN {
            return Err((StatusCode::BAD_REQUEST, "Entry field exceeds max length".into()));
        }
    }

    let role = get_role(&state, project_id, claims.sub).await?;
    if role == Role::Viewer {
        return Err((StatusCode::FORBIDDEN, "Viewers cannot modify entries".into()));
    }

    // Replace all entries for this project atomically
    let mut tx = state.db.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query("DELETE FROM env_entries WHERE project_id = $1")
        .bind(project_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    for entry in &entries {
        sqlx::query(
            "INSERT INTO env_entries (project_id, env_file, key, encrypted_value, is_sensitive) \
             VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(project_id)
        .bind(&entry.env_file)
        .bind(&entry.key)
        .bind(&entry.encrypted_value)
        .bind(entry.is_sensitive)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    sqlx::query("UPDATE projects SET updated_at = now() WHERE id = $1")
        .bind(project_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}

pub async fn list_entries(
    State(state): State<AppState>,
    claims: Claims,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<EntryResponse>>, (StatusCode, String)> {
    let _role = get_role(&state, project_id, claims.sub).await?;

    let entries = sqlx::query_as::<_, crate::models::EnvEntry>(
        "SELECT * FROM env_entries WHERE project_id = $1 ORDER BY env_file, key"
    )
    .bind(project_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(entries.into_iter().map(|e| EntryResponse {
        id: e.id,
        env_file: e.env_file,
        key: e.key,
        encrypted_value: e.encrypted_value,
        is_sensitive: e.is_sensitive,
    }).collect()))
}

async fn get_role(state: &AppState, project_id: Uuid, user_id: Uuid) -> Result<Role, (StatusCode, String)> {
    let member = sqlx::query_as::<_, crate::models::ProjectMember>(
        "SELECT * FROM project_members WHERE project_id = $1 AND user_id = $2"
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::FORBIDDEN, "Access denied".into()))?;

    Ok(member.role)
}
