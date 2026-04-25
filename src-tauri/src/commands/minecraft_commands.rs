//! Tauri shims for game-specific server operations (RCON, backups).

use tauri::State;

use cubelit_core::server::ServerLifecycle;

use crate::error::CoreError;
use crate::state::AppState;

/// Send a command to a running Minecraft server via RCON and return the response.
#[tauri::command]
pub async fn send_minecraft_command(
    state: State<'_, AppState>,
    id: String,
    command: String,
) -> Result<String, CoreError> {
    state.host.send_minecraft_command(&id, &command).await
}

/// Copy the server's data directory to a timestamped backup folder.
/// Returns the path of the created backup.
#[tauri::command]
pub async fn backup_server(
    state: State<'_, AppState>,
    id: String,
) -> Result<String, CoreError> {
    state.host.backup_server(&id).await
}
