pub mod parser;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

pub use parser::EnvEntry;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EnvFile {
    pub filename: String,
    pub entries: Vec<EnvEntry>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub directory: String,
    pub folder_name: String,
    pub files: Vec<EnvFile>,
}

#[tauri::command]
pub fn scan_directory(directory: &str) -> Result<ScanResult, String> {
    let path = Path::new(directory);
    if !path.is_dir() {
        return Err(format!("Not a valid directory: {}", directory));
    }

    let folder_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| directory.to_string());

    let dir_entries = fs::read_dir(path).map_err(|e| e.to_string())?;
    let mut filenames: Vec<String> = dir_entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name().to_string_lossy().to_string();
            if entry.file_type().ok()?.is_file() && is_env_file(&name) {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    filenames.sort();

    let files: Vec<EnvFile> = filenames
        .into_iter()
        .filter_map(|filename| {
            let filepath = path.join(&filename);
            let entries = parser::parse(filepath.to_str()?).ok()?;
            Some(EnvFile { filename, entries })
        })
        .collect();

    Ok(ScanResult {
        directory: directory.to_string(),
        folder_name,
        files,
    })
}

fn is_env_file(name: &str) -> bool {
    name == ".env"
        || name.starts_with(".env.")
        || name.ends_with(".env")
        || name == ".env.example"
}
