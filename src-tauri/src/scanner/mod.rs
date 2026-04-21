pub mod parser;

use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

pub use parser::EnvEntry;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EnvFile {
    pub filename: String,
    pub entries: Vec<EnvEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub directory: String,
    pub folder_name: String,
    pub files: Vec<EnvFile>,
}

const SKIP_DIRS: &[&str] = &[
    "node_modules", "target", "dist", ".git", "build", "out",
    "__pycache__", ".next", ".nuxt", "vendor",
];

#[tauri::command]
pub fn scan_directory(directory: &str) -> Result<ScanResult, String> {
    let root = Path::new(directory);
    if !root.is_dir() {
        return Err(format!("Not a valid directory: {}", directory));
    }

    let folder_name = root.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| directory.to_string());

    let mut found: Vec<PathBuf> = Vec::new();
    walk(root, &mut found);
    found.sort();

    let files = found.into_iter().filter_map(|path| {
        let rel = path.strip_prefix(root).ok()?;
        let filename = rel.to_string_lossy().replace('\\', "/");
        let entries = parser::parse(path.to_str()?).ok()?;
        Some(EnvFile { filename, entries })
    }).collect();

    Ok(ScanResult { directory: directory.to_string(), folder_name, files })
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if path.is_dir() {
            if !SKIP_DIRS.contains(&name.as_ref()) {
                walk(&path, out);
            }
        } else if is_env_file(&name) {
            out.push(path);
        }
    }
}

fn is_env_file(name: &str) -> bool {
    name == ".env" || name.starts_with(".env.") || name.ends_with(".env")
}
