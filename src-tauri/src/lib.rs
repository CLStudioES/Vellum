mod api;
mod scanner;

use api::{AppSession, client::ApiClient};
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppSession {
            token: Mutex::new(None),
            client: ApiClient::new(),
        })
        .invoke_handler(tauri::generate_handler![
            api::register,
            api::login,
            api::logout,
            api::get_session,
            api::list_projects,
            api::save_to_project,
            api::get_project_entries,
            scanner::scan_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
