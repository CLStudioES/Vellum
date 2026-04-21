mod crypto;

use crate::auth::SessionState;
use crate::scanner::EnvFile;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{Manager, State};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Owner,
    Editor,
    Viewer,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMember {
    pub user_id: String,
    pub role: Role,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub directory: String,
    pub owner_id: String,
    pub members: Vec<ProjectMember>,
    pub files: Vec<EnvFile>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Default)]
struct VaultStore {
    projects: Vec<Project>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub directory: String,
    pub owner_id: String,
    pub role: Role,
    pub file_count: usize,
    pub entry_count: usize,
    pub created_at: String,
    pub updated_at: String,
}

fn vault_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("vault.enc"))
}

fn now_ts() -> String {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", d.as_secs())
}

fn get_vault_key(session_state: &State<'_, SessionState>) -> Result<[u8; 32], String> {
    let session = session_state.0.lock().map_err(|e| e.to_string())?;
    session
        .as_ref()
        .map(|s| s.vault_key)
        .ok_or_else(|| "Not authenticated".to_string())
}

fn get_user_id(session_state: &State<'_, SessionState>) -> Result<String, String> {
    let session = session_state.0.lock().map_err(|e| e.to_string())?;
    session
        .as_ref()
        .map(|s| s.user_id.clone())
        .ok_or_else(|| "Not authenticated".to_string())
}

fn load_vault(app: &tauri::AppHandle, key: &[u8; 32]) -> Result<VaultStore, String> {
    let path = vault_path(app)?;
    if !path.exists() {
        return Ok(VaultStore::default());
    }
    let ciphertext = fs::read(&path).map_err(|e| e.to_string())?;
    let plaintext = crypto::decrypt(key, &ciphertext)?;
    serde_json::from_slice(&plaintext).map_err(|e| e.to_string())
}

fn persist_vault(app: &tauri::AppHandle, key: &[u8; 32], store: &VaultStore) -> Result<(), String> {
    let path = vault_path(app)?;
    let plaintext = serde_json::to_vec(store).map_err(|e| e.to_string())?;
    let ciphertext = crypto::encrypt(key, &plaintext)?;
    fs::write(&path, ciphertext).map_err(|e| e.to_string())
}

fn to_summary(project: &Project, user_id: &str) -> ProjectSummary {
    let role = project
        .members
        .iter()
        .find(|m| m.user_id == user_id)
        .map(|m| m.role.clone())
        .unwrap_or(Role::Viewer);

    let entry_count: usize = project.files.iter().map(|f| f.entries.len()).sum();

    ProjectSummary {
        id: project.id.clone(),
        name: project.name.clone(),
        directory: project.directory.clone(),
        owner_id: project.owner_id.clone(),
        role,
        file_count: project.files.len(),
        entry_count,
        created_at: project.created_at.clone(),
        updated_at: project.updated_at.clone(),
    }
}

#[tauri::command]
pub fn list_projects(
    app: tauri::AppHandle,
    session_state: State<'_, SessionState>,
) -> Result<Vec<ProjectSummary>, String> {
    let key = get_vault_key(&session_state)?;
    let user_id = get_user_id(&session_state)?;
    let store = load_vault(&app, &key)?;

    let visible: Vec<ProjectSummary> = store
        .projects
        .iter()
        .filter(|p| p.members.iter().any(|m| m.user_id == user_id))
        .map(|p| to_summary(p, &user_id))
        .collect();

    Ok(visible)
}

#[tauri::command]
pub fn save_to_project(
    app: tauri::AppHandle,
    session_state: State<'_, SessionState>,
    project_id: Option<&str>,
    project_name: &str,
    directory: &str,
    files: Vec<EnvFile>,
) -> Result<ProjectSummary, String> {
    let key = get_vault_key(&session_state)?;
    let user_id = get_user_id(&session_state)?;
    let mut store = load_vault(&app, &key)?;
    let ts = now_ts();

    let project = match project_id {
        Some(id) => {
            let existing = store
                .projects
                .iter_mut()
                .find(|p| p.id == id)
                .ok_or_else(|| format!("Project not found: {}", id))?;

            let member = existing
                .members
                .iter()
                .find(|m| m.user_id == user_id)
                .ok_or("Access denied")?;

            if member.role == Role::Viewer {
                return Err("Viewers cannot modify projects".into());
            }

            existing.files = files;
            existing.updated_at = ts;
            existing.clone()
        }
        None => {
            let new_project = Project {
                id: Uuid::new_v4().to_string(),
                name: project_name.to_string(),
                directory: directory.to_string(),
                owner_id: user_id.clone(),
                members: vec![ProjectMember {
                    user_id: user_id.clone(),
                    role: Role::Owner,
                }],
                files,
                created_at: ts.clone(),
                updated_at: ts,
            };
            store.projects.push(new_project.clone());
            new_project
        }
    };

    persist_vault(&app, &key, &store)?;
    Ok(to_summary(&project, &user_id))
}
