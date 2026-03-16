use serde::Serialize;
use std::path::PathBuf;
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

#[tauri::command]
pub async fn list_server_files(
    state: State<'_, AppState>,
    id: String,
    subpath: Option<String>,
) -> Result<Vec<FileEntry>, AppError> {
    let cubelit = crate::db::queries::get_cubelit(&state.db, &id).await?;
    let mut base = PathBuf::from(&cubelit.volume_path);
    if let Some(ref sub) = subpath {
        base = base.join(sub);
    }

    if !base.exists() {
        return Ok(vec![]);
    }

    let mut entries = Vec::new();
    for entry in std::fs::read_dir(&base)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        entries.push(FileEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            is_dir: metadata.is_dir(),
            size: metadata.len(),
        });
    }

    entries.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name))
    });

    Ok(entries)
}

#[tauri::command]
pub async fn copy_file_to_server(
    state: State<'_, AppState>,
    id: String,
    source_path: String,
    dest_subpath: String,
) -> Result<(), AppError> {
    let cubelit = crate::db::queries::get_cubelit(&state.db, &id).await?;
    let dest = PathBuf::from(&cubelit.volume_path).join(&dest_subpath);

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::copy(&source_path, &dest)?;
    Ok(())
}

#[tauri::command]
pub async fn delete_server_file(
    state: State<'_, AppState>,
    id: String,
    filepath: String,
) -> Result<(), AppError> {
    let cubelit = crate::db::queries::get_cubelit(&state.db, &id).await?;
    let full_path = PathBuf::from(&cubelit.volume_path).join(&filepath);

    // Safety: ensure the path is within the volume
    let canonical_base = PathBuf::from(&cubelit.volume_path).canonicalize().unwrap_or_else(|_| PathBuf::from(&cubelit.volume_path));
    let canonical_target = full_path.canonicalize().unwrap_or(full_path.clone());
    if !canonical_target.starts_with(&canonical_base) {
        return Err(AppError::Validation("Path traversal not allowed".into()));
    }

    if full_path.is_dir() {
        std::fs::remove_dir_all(&full_path)?;
    } else {
        std::fs::remove_file(&full_path)?;
    }
    Ok(())
}
