use bollard::Docker;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::path::PathBuf;

use crate::error::AppError;

pub struct AppState {
    pub docker: Docker,
    pub db: SqlitePool,
    pub data_dir: PathBuf,
    pub recipes_dir: PathBuf,
}

impl AppState {
    pub async fn new(data_dir: PathBuf, recipes_dir: PathBuf) -> Result<Self, AppError> {
        let docker = Docker::connect_with_local_defaults()?;

        std::fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("cubelit.db");
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

        let db = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        sqlx::query("PRAGMA journal_mode=WAL;")
            .execute(&db)
            .await?;

        crate::db::run_migrations(&db).await?;

        Ok(Self {
            docker,
            db,
            data_dir,
            recipes_dir,
        })
    }
}
