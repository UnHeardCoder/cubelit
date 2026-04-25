use tauri::State;

use cubelit_core::recipes::{self, Recipe, RecipeSummary};

use crate::error::CoreError;
use crate::state::AppState;

#[tauri::command]
pub async fn list_recipes(state: State<'_, AppState>) -> Result<Vec<RecipeSummary>, CoreError> {
    let recipes = recipes::load_recipes(&state.recipes_dir)?;
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
) -> Result<Recipe, CoreError> {
    recipes::get_recipe(&state.recipes_dir, &id)
}
