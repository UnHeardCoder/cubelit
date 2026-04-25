//! Tauri shims for DB-only metadata operations on a server.

use tauri::State;

use cubelit_core::db::models::Cubelit;
use cubelit_core::server::ServerLifecycle;

use crate::error::CoreError;
use crate::state::AppState;

#[tauri::command]
pub async fn list_cubelits(state: State<'_, AppState>) -> Result<Vec<Cubelit>, CoreError> {
    state.host.list_servers().await
}

#[tauri::command]
pub async fn get_cubelit(state: State<'_, AppState>, id: String) -> Result<Cubelit, CoreError> {
    state.host.get_server(&id).await
}

#[tauri::command]
pub async fn rename_server(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> Result<Cubelit, CoreError> {
    state.host.rename_server(&id, &name).await
}
