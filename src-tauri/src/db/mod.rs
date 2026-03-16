pub mod models;
pub mod queries;

use sqlx::SqlitePool;

use crate::error::AppError;

pub async fn run_migrations(db: &SqlitePool) -> Result<(), AppError> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS cubelits (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            game TEXT NOT NULL,
            recipe_id TEXT NOT NULL,
            docker_image TEXT NOT NULL,
            container_id TEXT,
            status TEXT NOT NULL DEFAULT 'created',
            port_mappings TEXT NOT NULL DEFAULT '{}',
            environment TEXT NOT NULL DEFAULT '{}',
            volume_path TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
    )
    .execute(db)
    .await?;

    // Migration: add container_mount_path column
    sqlx::query(
        "ALTER TABLE cubelits ADD COLUMN container_mount_path TEXT NOT NULL DEFAULT '/data'",
    )
    .execute(db)
    .await
    .ok();

    // Migration: add sidecar columns for FiveM MariaDB
    sqlx::query("ALTER TABLE cubelits ADD COLUMN sidecar_container_id TEXT")
        .execute(db)
        .await
        .ok();

    sqlx::query("ALTER TABLE cubelits ADD COLUMN sidecar_image TEXT")
        .execute(db)
        .await
        .ok();

    Ok(())
}
