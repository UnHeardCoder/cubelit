pub mod models;
pub mod queries;

use sqlx::SqlitePool;

use crate::error::CoreError;

pub async fn run_migrations(db: &SqlitePool) -> Result<(), CoreError> {
    sqlx::migrate!("./migrations").run(db).await?;
    Ok(())
}
