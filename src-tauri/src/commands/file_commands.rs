//! Tauri shims for filesystem operations within a server's data volume.
//!
//! These commands deliberately stay in the desktop crate rather than core
//! because they're host-side concerns (drag-drop file transfer, "open in
//! file manager") that don't generalize to the future remote agent
//! transport. They lean on `state.host.get_server` to resolve the
//! `volume_path` they should operate on.
//!
//! All three commands accept a user-supplied relative path. Every entrypoint
//! runs the same two-step containment guard:
//!   1. Syntactic check (`validate_relative_subpath`) — reject absolute
//!      paths, drive prefixes, and any `..` segment up front.
//!   2. Canonical check — resolve the volume root and the target (or, for
//!      writes, the parent that's about to receive the file) with
//!      `std::fs::canonicalize`, and require the result to start with the
//!      canonical volume root. This catches symlink-based escapes that a
//!      pure component check would miss. If canonicalization fails we
//!      reject — never fall back to the raw joined path.
//!
//! See `tests::` at the bottom of this file for the exact escape vectors
//! these guards block.

use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::State;

use cubelit_core::server::ServerLifecycle;

use crate::error::CoreError;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

/// Reject any user-supplied subpath that's absolute, has a Windows drive
/// prefix, or contains a `..` segment. These are the cheap syntactic
/// containment checks; the canonical `starts_with` guard is layered on
/// top by each call site.
fn validate_relative_subpath(subpath: &str) -> Result<(), CoreError> {
    use std::path::Component;
    let p = Path::new(subpath);
    if p.is_absolute() {
        return Err(CoreError::Validation("Subpath must be relative".into()));
    }
    for component in p.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(CoreError::Validation("Path traversal not allowed".into()));
            }
        }
    }
    Ok(())
}

/// Canonicalize `path` and verify it starts with `canonical_base`. Used
/// after a target file/dir is known to exist on disk (delete + read paths).
fn ensure_canonical_within(
    canonical_base: &Path,
    path: &Path,
) -> Result<PathBuf, CoreError> {
    let canonical = path
        .canonicalize()
        .map_err(|_| CoreError::Validation("Path traversal not allowed".into()))?;
    if !canonical.starts_with(canonical_base) {
        return Err(CoreError::Validation("Path traversal not allowed".into()));
    }
    Ok(canonical)
}

#[tauri::command]
pub async fn list_server_files(
    state: State<'_, AppState>,
    id: String,
    subpath: Option<String>,
) -> Result<Vec<FileEntry>, CoreError> {
    let cubelit = state.host.get_server(&id).await?;
    let base = PathBuf::from(&cubelit.volume_path);

    if let Some(ref sub) = subpath {
        validate_relative_subpath(sub)?;
    }

    let target = match subpath {
        Some(ref sub) => base.join(sub),
        None => base.clone(),
    };

    if !target.exists() {
        return Ok(vec![]);
    }

    // Canonical containment check — guards against any symlink already
    // sitting inside the volume that points outside it.
    let canonical_base = base
        .canonicalize()
        .map_err(|_| CoreError::Validation("Server data directory not accessible".into()))?;
    let _ = ensure_canonical_within(&canonical_base, &target)?;

    let mut entries = Vec::new();
    for entry in std::fs::read_dir(&target)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        entries.push(FileEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            is_dir: metadata.is_dir(),
            size: metadata.len(),
        });
    }

    entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then(a.name.cmp(&b.name)));

    Ok(entries)
}

#[tauri::command]
pub async fn copy_file_to_server(
    state: State<'_, AppState>,
    id: String,
    source_path: String,
    dest_subpath: String,
) -> Result<(), CoreError> {
    let cubelit = state.host.get_server(&id).await?;
    let base = PathBuf::from(&cubelit.volume_path);

    validate_relative_subpath(&dest_subpath)?;

    let dest = base.join(&dest_subpath);

    // Create the destination's parent before canonicalizing — canonicalize
    // requires the directory to exist on disk.
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Canonical containment check on the parent. We canonicalize the parent
    // (which now exists) rather than `dest` itself because `dest` is the
    // about-to-be-created file. Any symlink that would escape the volume
    // is caught here.
    let canonical_base = base
        .canonicalize()
        .map_err(|_| CoreError::Validation("Server data directory not accessible".into()))?;
    let parent = dest
        .parent()
        .ok_or_else(|| CoreError::Validation("Invalid destination path".into()))?;
    let _ = ensure_canonical_within(&canonical_base, parent)?;

    std::fs::copy(&source_path, &dest)?;
    Ok(())
}

#[tauri::command]
pub async fn delete_server_file(
    state: State<'_, AppState>,
    id: String,
    filepath: String,
) -> Result<(), CoreError> {
    let cubelit = state.host.get_server(&id).await?;
    let base = PathBuf::from(&cubelit.volume_path);

    validate_relative_subpath(&filepath)?;

    let full_path = base.join(&filepath);

    // Canonical containment — refuse to delete anything that doesn't
    // resolve cleanly inside the volume. Failure to canonicalize means
    // the path doesn't exist or escapes; in either case we reject rather
    // than fall back to the raw joined path (which is the bug this fix
    // closes — a `..` segment in `filepath` would syntactically pass the
    // old prefix check but resolve outside the volume on the next call).
    let canonical_base = base
        .canonicalize()
        .map_err(|_| CoreError::Validation("Server data directory not accessible".into()))?;
    let canonical_target = ensure_canonical_within(&canonical_base, &full_path)?;

    if canonical_target.is_dir() {
        std::fs::remove_dir_all(&canonical_target)?;
    } else {
        std::fs::remove_file(&canonical_target)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_absolute_paths() {
        assert!(validate_relative_subpath("/etc/passwd").is_err());
    }

    #[test]
    fn rejects_parent_dir_segments() {
        assert!(validate_relative_subpath("../etc/passwd").is_err());
        assert!(validate_relative_subpath("mods/../../etc/passwd").is_err());
        assert!(validate_relative_subpath("..").is_err());
    }

    #[test]
    fn allows_normal_relative_paths() {
        assert!(validate_relative_subpath("mods/cool-mod.jar").is_ok());
        assert!(validate_relative_subpath("server.properties").is_ok());
        assert!(validate_relative_subpath("./logs/latest.log").is_ok());
    }

    #[cfg(windows)]
    #[test]
    fn rejects_windows_drive_prefixes() {
        assert!(validate_relative_subpath("C:\\Windows\\System32").is_err());
    }
}
