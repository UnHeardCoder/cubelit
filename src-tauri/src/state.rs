use std::path::PathBuf;

use cubelit_core::server::LocalServerHost;

use crate::error::CoreError;

/// Tauri-managed application state.
///
/// All persistent resources (Docker handle, SQLite pool, data dir, recipes
/// dir) live on `host`. Earlier versions stored these as separate fields
/// on `AppState` itself; v0.1.8 collapses them into the same
/// `LocalServerHost` that implements the `ServerLifecycle` trait, so the
/// Tauri command shims call `state.host.<method>(...).await` instead of
/// reaching for individual fields.
pub struct AppState {
    pub host: LocalServerHost,
}

impl AppState {
    pub async fn new(data_dir: PathBuf, recipes_dir: PathBuf) -> Result<Self, CoreError> {
        let host = LocalServerHost::new(data_dir, recipes_dir).await?;
        Ok(Self { host })
    }
}
