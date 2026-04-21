use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::middleware::Claims;
use crate::models::Role;
use crate::AppState;

const MAX_PROJECT_NAME_LEN: usize = 64;

#[derive(Deserialize)]
pub struct CreateProjectPayload {
    pub name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectResponse {
    pub id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    pub role: Role,
    pub created_at: String,
    pub updated_at: String,
}

pub async fn create_project(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateProjectPayload>,
) -> Result<(StatusCode, Json<ProjectResponse>), (StatusCode, String)> {
    let name = payload.name.trim().to_string();
    if name.is_empty() || name.len() > MAX_PROJECT_NAME_LEN {
        return Err((StatusCode::BAD_REQUEST, format!("Project name must be 1-{} characters", MAX_PROJECT_NAME_LEN)));
    }

    let project = sqlx::query_as::<_, crate::models::Project>(
        "INSERT INTO projects (name, owner_id) VALUES ($1, $2) RETURNING *"
    )
    .bind(&name)
    .bind(claims.sub)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query(
        "INSERT INTO project_members (project_id, user_id, role) VALUES ($1, $2, 'owner')"
    )
    .bind(project.id)
    .bind(claims.sub)
    .execute(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(ProjectResponse {
        id: project.id,
        name: project.name,
        owner_id: project.owner_id,
        role: Role::Owner,
        created_at: project.created_at.to_rfc3339(),
        updated_at: project.updated_at.to_rfc3339(),
    })))
}

pub async fn list_projects(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<Vec<ProjectResponse>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, crate::models::ProjectWithRole>(
        "SELECT p.id, p.name, p.owner_id, p.created_at, p.updated_at, pm.role \
         FROM projects p \
         JOIN project_members pm ON pm.project_id = p.id \
         WHERE pm.user_id = $1 \
         ORDER BY p.updated_at DESC"
    )
    .bind(claims.sub)
    .fetch_all(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let result = rows.into_iter().map(|r| ProjectResponse {
        id: r.id,
        name: r.name,
        owner_id: r.owner_id,
        role: r.role,
        created_at: r.created_at.to_rfc3339(),
        updated_at: r.updated_at.to_rfc3339(),
    }).collect();

    Ok(Json(result))
}

pub async fn delete_project(
    State(state): State<AppState>,
    claims: Claims,
    Path(project_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let project = sqlx::query_as::<_, crate::models::Project>(
        "SELECT * FROM projects WHERE id = $1"
    )
    .bind(project_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Project not found".into()))?;

    if project.owner_id != claims.sub {
        return Err((StatusCode::FORBIDDEN, "Only the owner can delete a project".into()));
    }

    sqlx::query("DELETE FROM projects WHERE id = $1")
        .bind(project_id)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
