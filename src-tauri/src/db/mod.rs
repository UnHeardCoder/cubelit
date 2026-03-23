pub mod models;
pub mod queries;

use sqlx::SqlitePool;

use crate::error::AppError;

pub async fn run_migrations(db: &SqlitePool) -> Result<(), AppError> {
    sqlx::migrate!("./migrations").run(db).await?;
    Ok(())
}
