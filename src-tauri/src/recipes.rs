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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    fn write_recipe(dir: &std::path::Path, filename: &str, json: &str) {
        let path = dir.join(filename);
        let mut f = std::fs::File::create(path).unwrap();
        f.write_all(json.as_bytes()).unwrap();
    }

    const MINIMAL_RECIPE: &str = r#"{
        "id": "test-game",
        "name": "Test Game",
        "description": "A test game server.",
        "icon": "test-game",
        "available": true,
        "docker_image": "example/test",
        "default_tag": "1.0.0",
        "ports": [{"container_port": 7777, "default_host_port": 7777, "protocol": "udp", "label": "Game Port"}],
        "environment": [{"key": "MAX_PLAYERS", "default_value": "16", "label": "Max Players", "type": "number", "options": []}],
        "volumes": [{"container_path": "/data", "label": "Data"}],
        "estimated_disk_mb": 500,
        "tags": ["test"]
    }"#;

    #[test]
    fn load_recipes_parses_valid_json() {
        let dir = tempdir().unwrap();
        write_recipe(dir.path(), "test-game.json", MINIMAL_RECIPE);
        let recipes = load_recipes(dir.path()).unwrap();
        assert_eq!(recipes.len(), 1);
        let r = &recipes[0];
        assert_eq!(r.id, "test-game");
        assert_eq!(r.docker_image, "example/test");
        assert_eq!(r.default_tag, "1.0.0");
        assert!(r.available);
        assert_eq!(r.ports[0].container_port, 7777);
    }

    #[test]
    fn load_recipes_skips_invalid_json() {
        let dir = tempdir().unwrap();
        write_recipe(dir.path(), "broken.json", "{ not valid json }");
        let recipes = load_recipes(dir.path()).unwrap();
        assert!(recipes.is_empty());
    }

    #[test]
    fn load_recipes_ignores_non_json_files() {
        let dir = tempdir().unwrap();
        write_recipe(dir.path(), "readme.txt", "not a recipe");
        let recipes = load_recipes(dir.path()).unwrap();
        assert!(recipes.is_empty());
    }

    #[test]
    fn load_recipes_returns_empty_for_missing_dir() {
        let recipes = load_recipes(std::path::Path::new("/nonexistent/path/xyz")).unwrap();
        assert!(recipes.is_empty());
    }

    #[test]
    fn get_recipe_finds_by_id() {
        let dir = tempdir().unwrap();
        write_recipe(dir.path(), "test-game.json", MINIMAL_RECIPE);
        let recipe = get_recipe(dir.path(), "test-game").unwrap();
        assert_eq!(recipe.id, "test-game");
    }

    #[test]
    fn get_recipe_returns_not_found_for_missing_id() {
        let dir = tempdir().unwrap();
        write_recipe(dir.path(), "test-game.json", MINIMAL_RECIPE);
        let err = get_recipe(dir.path(), "does-not-exist").unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
    }
}
