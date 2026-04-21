pub mod client;

use client::{ApiClient, AuthResponse, EntryPayload, EntryResponse, ProjectResponse};
use serde::Serialize;
use std::collections::HashSet;
use std::sync::Mutex;
use tauri::State;

pub struct AppSession {
    pub token: Mutex<Option<String>>,
    pub client: ApiClient,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo { pub user_id: String, pub username: String }

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SaveResult {
    pub project_id: String, pub project_name: String,
    pub new_count: usize, pub skipped_count: usize,
}

fn token(s: &State<'_, AppSession>) -> Result<String, String> {
    s.token.lock().map_err(|e| e.to_string())?
        .clone().ok_or_else(|| "Not authenticated".into())
}

fn store_token(s: &State<'_, AppSession>, t: &str) -> Result<(), String> {
    *s.token.lock().map_err(|e| e.to_string())? = Some(t.to_string());
    Ok(())
}

#[tauri::command]
pub async fn register(session: State<'_, AppSession>, username: &str, password: &str) -> Result<SessionInfo, String> {
    let res: AuthResponse = session.client.register(username, password).await?;
    store_token(&session, &res.token)?;
    Ok(SessionInfo { user_id: res.user_id, username: res.username })
}

#[tauri::command]
pub async fn login(session: State<'_, AppSession>, username: &str, password: &str) -> Result<SessionInfo, String> {
    let res: AuthResponse = session.client.login(username, password).await?;
    store_token(&session, &res.token)?;
    Ok(SessionInfo { user_id: res.user_id, username: res.username })
}

#[tauri::command]
pub async fn logout(session: State<'_, AppSession>) -> Result<(), String> {
    *session.token.lock().map_err(|e| e.to_string())? = None;
    Ok(())
}

#[tauri::command]
pub async fn get_session(_: State<'_, AppSession>) -> Result<Option<SessionInfo>, String> {
    Ok(None)
}

#[tauri::command]
pub async fn list_projects(session: State<'_, AppSession>) -> Result<Vec<ProjectResponse>, String> {
    session.client.list_projects(&token(&session)?).await
}

#[tauri::command]
pub async fn save_to_project(
    session: State<'_, AppSession>,
    project_id: Option<&str>,
    project_name: &str,
    entries: Vec<EntryPayload>,
) -> Result<SaveResult, String> {
    let tok = token(&session)?;

    let (pid, name) = match project_id {
        Some(id) => (id.to_string(), project_name.to_string()),
        None => {
            let p = session.client.create_project(&tok, project_name).await?;
            (p.id, p.name)
        }
    };

    let existing: Vec<EntryResponse> = session.client.list_entries(&tok, &pid).await.unwrap_or_default();
    let existing_keys: HashSet<String> = existing.iter()
        .map(|e| format!("{}::{}", e.env_file, e.key)).collect();

    let new_entries: Vec<&EntryPayload> = entries.iter()
        .filter(|e| !existing_keys.contains(&format!("{}::{}", e.env_file, e.key))).collect();

    let new_count = new_entries.len();
    let skipped_count = entries.len() - new_count;

    if new_count > 0 {
        let mut merged: Vec<EntryPayload> = existing.into_iter().map(|e| EntryPayload {
            env_file: e.env_file, key: e.key,
            encrypted_value: e.encrypted_value, is_sensitive: e.is_sensitive,
        }).collect();
        merged.extend(new_entries.into_iter().cloned());
        session.client.upsert_entries(&tok, &pid, &merged).await?;
    }

    Ok(SaveResult { project_id: pid, project_name: name, new_count, skipped_count })
}

#[tauri::command]
pub async fn get_project_entries(session: State<'_, AppSession>, project_id: &str) -> Result<Vec<EntryResponse>, String> {
    session.client.list_entries(&token(&session)?, project_id).await
}
