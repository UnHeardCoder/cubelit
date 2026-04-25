//! Tauri shims for the `ServerLifecycle` trait.
//!
//! Each `#[tauri::command]` here is a thin wrapper that:
//!   1. Constructs a `TauriEventSink` (cheap — just clones the `AppHandle`).
//!   2. Delegates to a method on `state.host` (the core `LocalServerHost`).
//!   3. Returns the result back to the frontend over IPC.
//!
//! The wire shape (command names, argument names, payload shapes, error
//! strings) matches v0.1.7 byte-for-byte. Don't rename arguments here —
//! the frontend passes them by name.

use std::collections::HashMap;
use std::sync::Arc;

use tauri::{AppHandle, State};

use cubelit_core::db::models::Cubelit;
use cubelit_core::docker;
use cubelit_core::events::EventSink;
use cubelit_core::server::ServerLifecycle;
use cubelit_core::CreateServerConfig;

use crate::error::CoreError;
use crate::event_sink::TauriEventSink;
use crate::state::AppState;

#[tauri::command]
pub async fn check_docker_status(
    state: State<'_, AppState>,
) -> Result<docker::health::DockerStatus, CoreError> {
    Ok(docker::health::check_docker_status(&state.host.docker).await)
}

#[tauri::command]
pub async fn create_server(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    config: CreateServerConfig,
) -> Result<Cubelit, CoreError> {
    let events: Arc<dyn EventSink> = TauriEventSink::shared(app_handle);
    state.host.create_server(config, events).await
}

#[tauri::command]
pub async fn start_server(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    id: String,
) -> Result<(), CoreError> {
    let events: Arc<dyn EventSink> = TauriEventSink::shared(app_handle);
    state.host.start_server(&id, events).await
}

#[tauri::command]
pub async fn stop_server(state: State<'_, AppState>, id: String) -> Result<(), CoreError> {
    state.host.stop_server(&id).await
}

#[tauri::command]
pub async fn restart_server(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    id: String,
) -> Result<(), CoreError> {
    let events: Arc<dyn EventSink> = TauriEventSink::shared(app_handle);
    state.host.restart_server(&id, events).await
}

#[tauri::command]
pub async fn delete_server(
    state: State<'_, AppState>,
    id: String,
    delete_data: bool,
) -> Result<(), CoreError> {
    state.host.delete_server(&id, delete_data).await
}

#[tauri::command]
pub async fn sync_server_status(
    state: State<'_, AppState>,
    id: String,
) -> Result<Cubelit, CoreError> {
    state.host.sync_single(&id).await
}

#[tauri::command]
pub async fn sync_all_statuses(state: State<'_, AppState>) -> Result<Vec<Cubelit>, CoreError> {
    state.host.sync_all().await
}

#[tauri::command]
pub async fn update_server_settings(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    id: String,
    environment: HashMap<String, String>,
) -> Result<Cubelit, CoreError> {
    let events: Arc<dyn EventSink> = TauriEventSink::shared(app_handle);
    state
        .host
        .update_server_settings(&id, environment, events)
        .await
}

#[tauri::command]
pub async fn get_server_stats(
    state: State<'_, AppState>,
    id: String,
) -> Result<docker::stats::ContainerStats, CoreError> {
    state.host.server_stats(&id).await
}

#[tauri::command]
pub async fn get_server_logs(
    id: String,
    lines: Option<u64>,
    state: State<'_, AppState>,
) -> Result<Vec<String>, CoreError> {
    state.host.server_logs(&id, lines).await
}
