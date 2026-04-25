pub mod queries;

pub use cubelit_core::db::models;

use sqlx::SqlitePool;

use crate::error::CoreError;

pub async fn run_migrations(db: &SqlitePool) -> Result<(), CoreError> {
    sqlx::migrate!("./migrations").run(db).await?;
    Ok(())
}
