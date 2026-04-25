use tauri::State;

use crate::db::models::Cubelit;
use crate::error::CoreError;
use crate::state::AppState;

#[tauri::command]
pub async fn list_cubelits(state: State<'_, AppState>) -> Result<Vec<Cubelit>, CoreError> {
    crate::db::queries::list_cubelits(&state.db).await
}

#[tauri::command]
pub async fn get_cubelit(state: State<'_, AppState>, id: String) -> Result<Cubelit, CoreError> {
    crate::db::queries::get_cubelit(&state.db, &id).await
}

#[tauri::command]
pub async fn rename_server(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> Result<Cubelit, CoreError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(CoreError::Validation("Server name cannot be empty".into()));
    }
    crate::db::queries::update_cubelit_name(&state.db, &id, &name).await?;
    crate::db::queries::get_cubelit(&state.db, &id).await
}
