use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

#[derive(FromRow, Serialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
pub enum Role {
    #[serde(rename = "owner")]  #[sqlx(rename = "owner")]  Owner,
    #[serde(rename = "editor")] #[sqlx(rename = "editor")] Editor,
    #[serde(rename = "viewer")] #[sqlx(rename = "viewer")] Viewer,
}

#[derive(FromRow)]
#[allow(dead_code)]
pub struct ProjectMember {
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub role: Role,
    pub created_at: DateTime<Utc>,
}

#[derive(FromRow)]
#[allow(dead_code)]
pub struct EnvEntry {
    pub id: Uuid,
    pub project_id: Uuid,
    pub env_file: String,
    pub key: String,
    pub encrypted_value: String,
    pub is_sensitive: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(FromRow)]
pub struct ProjectWithRole {
    pub id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub role: Role,
}

#[derive(FromRow)]
pub struct MemberRow {
    pub user_id: Uuid,
    pub username: String,
    pub role: Role,
}
