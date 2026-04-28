use std::path::{Path, PathBuf};

use include_dir::{include_dir, Dir};

use cubelit_core::error::{CoreError, CoreResult};
use cubelit_core::server::LocalServerHost;

static EMBEDDED_RECIPES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../src-tauri/recipes");

/// Match desktop: Linux `~/.config/cubelit`, Windows/macOS under `dirs::data_dir()`.
pub fn resolve_data_dir() -> PathBuf {
    std::env::var_os("CUBELIT_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            #[cfg(target_os = "linux")]
            {
                dirs::config_dir()
                    .expect("Linux: XDG config directory not available")
                    .join("cubelit")
            }
            #[cfg(not(target_os = "linux"))]
            {
                dirs::data_dir()
                    .expect("data directory not available")
                    .join("cubelit")
            }
        })
}

fn needs_recipe_seed(dir: &Path) -> bool {
    if !dir.exists() {
        return true;
    }
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return true;
    };
    let json_count = read_dir
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .count();
    json_count == 0
}

pub fn seed_embedded_recipes_if_needed(dir: &Path) -> CoreResult<()> {
    if !needs_recipe_seed(dir) {
        return Ok(());
    }
    std::fs::create_dir_all(dir)?;
    for file in EMBEDDED_RECIPES.files() {
        let rel = file.path();
        let dest = dir.join(rel);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&dest, file.contents())?;
    }
    Ok(())
}

/// `--recipes-dir` and `CUBELIT_RECIPES_DIR` skip seeding; default path is seeded from the embedded bundle.
pub fn resolve_recipes_dir(cli_flag: Option<&PathBuf>) -> CoreResult<PathBuf> {
    if let Some(p) = cli_flag {
        return Ok(p.clone());
    }
    if let Ok(s) = std::env::var("CUBELIT_RECIPES_DIR") {
        return Ok(PathBuf::from(s));
    }
    let base = dirs::data_dir().ok_or_else(|| {
        CoreError::Validation(
            "Could not resolve data directory for default recipes path".into(),
        )
    })?;
    let recipes = base.join("cubelit").join("recipes");
    seed_embedded_recipes_if_needed(&recipes)?;
    Ok(recipes)
}

pub struct Context {
    pub host: LocalServerHost,
}

impl Context {
    pub async fn new(recipes_flag: Option<PathBuf>) -> CoreResult<Self> {
        let data_dir = resolve_data_dir();
        std::fs::create_dir_all(&data_dir)?;
        let recipes_dir = resolve_recipes_dir(recipes_flag.as_ref())?;
        let host = LocalServerHost::new(data_dir, recipes_dir).await?;
        Ok(Self { host })
    }
}
