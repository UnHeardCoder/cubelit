use tauri::State;

use crate::error::AppError;
use crate::recipes::{Recipe, RecipeSummary};
use crate::state::AppState;

#[tauri::command]
pub async fn list_recipes(state: State<'_, AppState>) -> Result<Vec<RecipeSummary>, AppError> {
    let recipes = crate::recipes::load_recipes(&state.recipes_dir)?;
    Ok(recipes
        .into_iter()
        .map(|r| RecipeSummary {
            id: r.id,
            name: r.name,
            description: r.description,
            icon: r.icon,
            available: r.available,
            tags: r.tags,
        })
        .collect())
}

#[tauri::command]
pub async fn get_recipe_detail(
    state: State<'_, AppState>,
    id: String,
) -> Result<Recipe, AppError> {
    crate::recipes::get_recipe(&state.recipes_dir, &id)
}
