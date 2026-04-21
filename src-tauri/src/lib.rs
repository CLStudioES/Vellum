mod auth;
mod scanner;
mod vault;

use auth::SessionState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(SessionState(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            auth::register,
            auth::login,
            auth::logout,
            auth::get_session,
            scanner::scan_directory,
            vault::list_projects,
            vault::save_to_project,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
