use reqwest::Client;
use serde::{Deserialize, Serialize};

const API_BASE: &str = "http://localhost:3000";

#[derive(Clone)]
pub struct ApiClient {
    http: Client,
    base: String,
}

#[derive(Serialize)]
struct AuthPayload { username: String, password: String }

#[derive(Serialize)]
struct CreateProjectPayload { name: String }

#[allow(dead_code)]
#[derive(Serialize)]
pub struct InvitePayload { pub username: String, pub role: String }

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse { pub token: String, pub user_id: String, pub username: String }

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProjectResponse {
    pub id: String, pub name: String, pub owner_id: String,
    pub role: String, pub created_at: String, pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EntryPayload {
    pub env_file: String, pub key: String,
    pub encrypted_value: String, pub is_sensitive: bool,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EntryResponse {
    pub id: String, pub env_file: String, pub key: String,
    pub encrypted_value: String, pub is_sensitive: bool,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberResponse { pub user_id: String, pub username: String, pub role: String }

#[allow(dead_code)]
impl ApiClient {
    pub fn new() -> Self {
        Self { http: Client::new(), base: API_BASE.to_string() }
    }

    pub async fn register(&self, username: &str, password: &str) -> Result<AuthResponse, String> {
        self.post("/auth/register", &AuthPayload { username: username.into(), password: password.into() }).await
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<AuthResponse, String> {
        self.post("/auth/login", &AuthPayload { username: username.into(), password: password.into() }).await
    }

    pub async fn list_projects(&self, token: &str) -> Result<Vec<ProjectResponse>, String> {
        self.get_auth("/projects", token).await
    }

    pub async fn create_project(&self, token: &str, name: &str) -> Result<ProjectResponse, String> {
        self.post_auth("/projects", token, &CreateProjectPayload { name: name.into() }).await
    }

    pub async fn upsert_entries(&self, token: &str, project_id: &str, entries: &[EntryPayload]) -> Result<(), String> {
        let res = self.http.put(format!("{}/projects/{}/entries", self.base, project_id))
            .bearer_auth(token).json(entries)
            .send().await.map_err(|e| e.to_string())?;
        if res.status().is_success() { Ok(()) } else { Err(err_body(res).await) }
    }

    pub async fn list_entries(&self, token: &str, project_id: &str) -> Result<Vec<EntryResponse>, String> {
        self.get_auth(&format!("/projects/{}/entries", project_id), token).await
    }

    pub async fn invite_member(&self, token: &str, pid: &str, username: &str, role: &str) -> Result<MemberResponse, String> {
        self.post_auth(&format!("/projects/{}/members", pid), token, &InvitePayload { username: username.into(), role: role.into() }).await
    }

    pub async fn list_members(&self, token: &str, pid: &str) -> Result<Vec<MemberResponse>, String> {
        self.get_auth(&format!("/projects/{}/members", pid), token).await
    }

    async fn post<T: Serialize, R: serde::de::DeserializeOwned>(&self, path: &str, body: &T) -> Result<R, String> {
        let res = self.http.post(format!("{}{}", self.base, path))
            .json(body).send().await.map_err(|e| e.to_string())?;
        parse(res).await
    }

    async fn post_auth<T: Serialize, R: serde::de::DeserializeOwned>(&self, path: &str, token: &str, body: &T) -> Result<R, String> {
        let res = self.http.post(format!("{}{}", self.base, path))
            .bearer_auth(token).json(body).send().await.map_err(|e| e.to_string())?;
        parse(res).await
    }

    async fn get_auth<R: serde::de::DeserializeOwned>(&self, path: &str, token: &str) -> Result<R, String> {
        let res = self.http.get(format!("{}{}", self.base, path))
            .bearer_auth(token).send().await.map_err(|e| e.to_string())?;
        parse(res).await
    }
}

async fn parse<T: serde::de::DeserializeOwned>(res: reqwest::Response) -> Result<T, String> {
    if res.status().is_success() {
        res.json::<T>().await.map_err(|e| e.to_string())
    } else {
        Err(err_body(res).await)
    }
}

async fn err_body(res: reqwest::Response) -> String {
    res.text().await.unwrap_or_else(|_| "Unknown API error".into())
}
