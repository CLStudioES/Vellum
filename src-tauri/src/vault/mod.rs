mod crypto;

use crate::scanner::EnvFile;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub directory: String,
    pub files: Vec<EnvFile>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Default)]
struct VaultStore {
    projects: Vec<Project>,
}

/// Lightweight version sent to the frontend — no raw values
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub directory: String,
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

fn now_iso() -> String {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", d.as_secs())
}

fn load_vault(app: &tauri::AppHandle, passphrase: &str) -> Result<VaultStore, String> {
    let path = vault_path(app)?;
    if !path.exists() {
        return Ok(VaultStore::default());
    }
    let ciphertext = fs::read(&path).map_err(|e| e.to_string())?;
    let plaintext = crypto::decrypt(passphrase, &ciphertext)?;
    serde_json::from_slice(&plaintext).map_err(|e| e.to_string())
}

fn persist_vault(app: &tauri::AppHandle, passphrase: &str, store: &VaultStore) -> Result<(), String> {
    let path = vault_path(app)?;
    let plaintext = serde_json::to_vec(store).map_err(|e| e.to_string())?;
    let ciphertext = crypto::encrypt(passphrase, &plaintext)?;
    fs::write(&path, ciphertext).map_err(|e| e.to_string())
}

fn to_summary(p: &Project) -> ProjectSummary {
    let entry_count: usize = p.files.iter().map(|f| f.entries.len()).sum();
    ProjectSummary {
        id: p.id.clone(),
        name: p.name.clone(),
        directory: p.directory.clone(),
        file_count: p.files.len(),
        entry_count,
        created_at: p.created_at.clone(),
        updated_at: p.updated_at.clone(),
    }
}

#[tauri::command]
pub fn list_projects(app: tauri::AppHandle, passphrase: &str) -> Result<Vec<ProjectSummary>, String> {
    let store = load_vault(&app, passphrase)?;
    Ok(store.projects.iter().map(to_summary).collect())
}

#[tauri::command]
pub fn save_to_project(
    app: tauri::AppHandle,
    passphrase: &str,
    project_id: Option<&str>,
    project_name: &str,
    directory: &str,
    files: Vec<EnvFile>,
) -> Result<ProjectSummary, String> {
    let mut store = load_vault(&app, passphrase)?;
    let ts = now_iso();

    let project = match project_id {
        Some(id) => {
            let existing = store
                .projects
                .iter_mut()
                .find(|p| p.id == id)
                .ok_or_else(|| format!("Project not found: {}", id))?;
            existing.files = files;
            existing.updated_at = ts;
            existing.clone()
        }
        None => {
            let new_project = Project {
                id: Uuid::new_v4().to_string(),
                name: project_name.to_string(),
                directory: directory.to_string(),
                files,
                created_at: ts.clone(),
                updated_at: ts,
            };
            store.projects.push(new_project.clone());
            new_project
        }
    };

    persist_vault(&app, passphrase, &store)?;
    Ok(to_summary(&project))
}
