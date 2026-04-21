use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::middleware::Claims;
use crate::models::Role;
use crate::AppState;

#[derive(Deserialize)]
pub struct InvitePayload {
    pub username: String,
    pub role: Role,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberResponse {
    pub user_id: Uuid,
    pub username: String,
    pub role: Role,
}

pub async fn invite_member(
    State(state): State<AppState>,
    claims: Claims,
    Path(project_id): Path<Uuid>,
    Json(payload): Json<InvitePayload>,
) -> Result<(StatusCode, Json<MemberResponse>), (StatusCode, String)> {
    require_owner(&state, project_id, claims.sub).await?;

    if payload.role == Role::Owner {
        return Err((StatusCode::BAD_REQUEST, "Cannot invite as owner".into()));
    }

    let username = payload.username.trim().to_lowercase();
    if username.is_empty() || username.len() > 32 {
        return Err((StatusCode::BAD_REQUEST, "Invalid username".into()));
    }

    let target_user = sqlx::query_as::<_, crate::models::User>(
        "SELECT * FROM users WHERE username = $1"
    )
    .bind(&username)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "User not found".into()))?;

    sqlx::query(
        "INSERT INTO project_members (project_id, user_id, role) VALUES ($1, $2, $3) \
         ON CONFLICT (project_id, user_id) DO UPDATE SET role = $3"
    )
    .bind(project_id)
    .bind(target_user.id)
    .bind(&payload.role)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(MemberResponse {
        user_id: target_user.id,
        username: target_user.username,
        role: payload.role,
    })))
}

pub async fn list_members(
    State(state): State<AppState>,
    claims: Claims,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<MemberResponse>>, (StatusCode, String)> {
    require_membership(&state, project_id, claims.sub).await?;

    let rows = sqlx::query_as::<_, crate::models::MemberRow>(
        "SELECT pm.user_id, u.username, pm.role FROM project_members pm \
         JOIN users u ON u.id = pm.user_id \
         WHERE pm.project_id = $1 \
         ORDER BY pm.created_at"
    )
    .bind(project_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(rows.into_iter().map(|r| MemberResponse {
        user_id: r.user_id,
        username: r.username,
        role: r.role,
    }).collect()))
}

pub async fn remove_member(
    State(state): State<AppState>,
    claims: Claims,
    Path((project_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, String)> {
    require_owner(&state, project_id, claims.sub).await?;

    if user_id == claims.sub {
        return Err((StatusCode::BAD_REQUEST, "Cannot remove yourself as owner".into()));
    }

    sqlx::query("DELETE FROM project_members WHERE project_id = $1 AND user_id = $2")
        .bind(project_id)
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

async fn require_owner(state: &AppState, project_id: Uuid, user_id: Uuid) -> Result<(), (StatusCode, String)> {
    let member = sqlx::query_as::<_, crate::models::ProjectMember>(
        "SELECT * FROM project_members WHERE project_id = $1 AND user_id = $2"
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::FORBIDDEN, "Access denied".into()))?;

    if member.role != Role::Owner {
        return Err((StatusCode::FORBIDDEN, "Owner access required".into()));
    }
    Ok(())
}

async fn require_membership(state: &AppState, project_id: Uuid, user_id: Uuid) -> Result<(), (StatusCode, String)> {
    sqlx::query("SELECT 1 FROM project_members WHERE project_id = $1 AND user_id = $2")
        .bind(project_id)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::FORBIDDEN, "Access denied".into()))?;
    Ok(())
}
