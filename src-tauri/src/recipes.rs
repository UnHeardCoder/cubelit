use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub docker_image: String,
    pub default_tag: String,
    pub ports: Vec<RecipePort>,
    pub environment: Vec<RecipeEnvVar>,
    pub volumes: Vec<RecipeVolume>,
    #[serde(default)]
    pub config_files: Vec<RecipeConfigFile>,
    #[serde(default)]
    pub mods: Option<RecipeMods>,
    #[serde(default)]
    pub available: bool,
    #[serde(default)]
    pub estimated_disk_mb: u32,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipePort {
    pub container_port: u16,
    pub default_host_port: u16,
    pub protocol: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeEnvVar {
    pub key: String,
    pub default_value: String,
    pub label: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default)]
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeVolume {
    pub container_path: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeConfigFile {
    pub path: String,
    pub format: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeMods {
    pub supported: bool,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub file_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub available: bool,
    pub tags: Vec<String>,
}

pub fn load_recipes(recipes_dir: &Path) -> Result<Vec<Recipe>, AppError> {
    let mut recipes = Vec::new();

    if !recipes_dir.exists() {
        return Ok(recipes);
    }

    for entry in std::fs::read_dir(recipes_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            let content = std::fs::read_to_string(&path)?;
            match serde_json::from_str::<Recipe>(&content) {
                Ok(recipe) => recipes.push(recipe),
                Err(e) => {
                    eprintln!("Failed to parse recipe {:?}: {}", path, e);
                }
            }
        }
    }

    recipes.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(recipes)
}

pub fn get_recipe(recipes_dir: &Path, id: &str) -> Result<Recipe, AppError> {
    let recipes = load_recipes(recipes_dir)?;
    recipes
        .into_iter()
        .find(|r| r.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Recipe '{}' not found", id)))
}
