pub mod crypto;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Manager, State};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserProfile {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    #[serde(with = "base64_bytes")]
    pub vault_salt: Vec<u8>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct UserStore {
    pub users: Vec<UserProfile>,
}

/// Active session — held in memory, never persisted
pub struct Session {
    pub user_id: String,
    pub username: String,
    pub vault_key: [u8; 32],
}

pub struct SessionState(pub Mutex<Option<Session>>);

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub user_id: String,
    pub username: String,
}

fn users_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("users.json"))
}

fn load_users(app: &tauri::AppHandle) -> Result<UserStore, String> {
    let path = users_path(app)?;
    if !path.exists() {
        return Ok(UserStore::default());
    }
    let data = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&data).map_err(|e| e.to_string())
}

fn persist_users(app: &tauri::AppHandle, store: &UserStore) -> Result<(), String> {
    let path = users_path(app)?;
    let data = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    fs::write(&path, data).map_err(|e| e.to_string())
}

fn now_ts() -> String {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", d.as_secs())
}

#[tauri::command]
pub fn register(
    app: tauri::AppHandle,
    session_state: State<'_, SessionState>,
    username: &str,
    password: &str,
) -> Result<SessionInfo, String> {
    if username.trim().is_empty() || password.len() < 8 {
        return Err("Username required, password must be at least 8 characters".into());
    }

    let mut store = load_users(&app)?;

    if store.users.iter().any(|u| u.username == username) {
        return Err("Username already taken".into());
    }

    let password_hash = crypto::hash_password(password)?;
    let vault_salt = crypto::generate_salt();
    let vault_key = crypto::derive_vault_key(password, &vault_salt)?;

    let user = UserProfile {
        id: Uuid::new_v4().to_string(),
        username: username.to_string(),
        password_hash,
        vault_salt: vault_salt.to_vec(),
        created_at: now_ts(),
    };

    let info = SessionInfo {
        user_id: user.id.clone(),
        username: user.username.clone(),
    };

    store.users.push(user.clone());
    persist_users(&app, &store)?;

    let mut session = session_state.0.lock().map_err(|e| e.to_string())?;
    *session = Some(Session {
        user_id: user.id,
        username: user.username,
        vault_key,
    });

    Ok(info)
}

#[tauri::command]
pub fn login(
    app: tauri::AppHandle,
    session_state: State<'_, SessionState>,
    username: &str,
    password: &str,
) -> Result<SessionInfo, String> {
    let store = load_users(&app)?;

    let user = store
        .users
        .iter()
        .find(|u| u.username == username)
        .ok_or("Invalid credentials")?;

    if !crypto::verify_password(password, &user.password_hash)? {
        return Err("Invalid credentials".into());
    }

    let vault_key = crypto::derive_vault_key(password, &user.vault_salt)?;

    let info = SessionInfo {
        user_id: user.id.clone(),
        username: user.username.clone(),
    };

    let mut session = session_state.0.lock().map_err(|e| e.to_string())?;
    *session = Some(Session {
        user_id: user.id.clone(),
        username: user.username.clone(),
        vault_key,
    });

    Ok(info)
}

#[tauri::command]
pub fn logout(session_state: State<'_, SessionState>) -> Result<(), String> {
    let mut session = session_state.0.lock().map_err(|e| e.to_string())?;
    *session = None;
    Ok(())
}

#[tauri::command]
pub fn get_session(session_state: State<'_, SessionState>) -> Result<Option<SessionInfo>, String> {
    let session = session_state.0.lock().map_err(|e| e.to_string())?;
    Ok(session.as_ref().map(|s| SessionInfo {
        user_id: s.user_id.clone(),
        username: s.username.clone(),
    }))
}

// serde helper for Vec<u8> as base64
mod base64_bytes {
    use serde::{Deserialize, Deserializer, Serializer};
    use serde::de;

    pub fn serialize<S: Serializer>(bytes: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::Serialize;
        let encoded = bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        encoded.serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(d)?;
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(de::Error::custom))
            .collect()
    }
}
