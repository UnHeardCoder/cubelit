use sqlx::SqlitePool;

use crate::db::models::Cubelit;
use crate::error::AppError;

pub async fn insert_cubelit(db: &SqlitePool, cubelit: &Cubelit) -> Result<(), AppError> {
    sqlx::query!(
        "INSERT INTO cubelits (id, name, game, recipe_id, docker_image, container_id, status, port_mappings, environment, volume_path, container_mount_path, sidecar_container_id, sidecar_image, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        cubelit.id,
        cubelit.name,
        cubelit.game,
        cubelit.recipe_id,
        cubelit.docker_image,
        cubelit.container_id,
        cubelit.status,
        cubelit.port_mappings,
        cubelit.environment,
        cubelit.volume_path,
        cubelit.container_mount_path,
        cubelit.sidecar_container_id,
        cubelit.sidecar_image,
        cubelit.created_at,
        cubelit.updated_at,
    )
    .execute(db)
    .await?;

    Ok(())
}

pub async fn get_cubelit(db: &SqlitePool, id: &str) -> Result<Cubelit, AppError> {
    let cubelit = sqlx::query_as!(
        Cubelit,
        "SELECT id, name, game, recipe_id, docker_image, container_id, status, port_mappings, environment, volume_path, container_mount_path, sidecar_container_id, sidecar_image, created_at, updated_at FROM cubelits WHERE id = ?",
        id
    )
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Cubelit with id '{}' not found", id)))?;

    Ok(cubelit)
}

pub async fn list_cubelits(db: &SqlitePool) -> Result<Vec<Cubelit>, AppError> {
    let cubelits = sqlx::query_as!(
        Cubelit,
        "SELECT id, name, game, recipe_id, docker_image, container_id, status, port_mappings, environment, volume_path, container_mount_path, sidecar_container_id, sidecar_image, created_at, updated_at FROM cubelits ORDER BY created_at DESC"
    )
    .fetch_all(db)
    .await?;

    Ok(cubelits)
}

pub async fn update_cubelit_status(
    db: &SqlitePool,
    id: &str,
    status: &str,
    container_id: Option<&str>,
) -> Result<(), AppError> {
    let now = chrono::Utc::now().to_rfc3339();

    if let Some(cid) = container_id {
        sqlx::query!(
            "UPDATE cubelits SET status = ?, container_id = ?, updated_at = ? WHERE id = ?",
            status,
            cid,
            now,
            id,
        )
        .execute(db)
        .await?;
    } else {
        sqlx::query!(
            "UPDATE cubelits SET status = ?, updated_at = ? WHERE id = ?",
            status,
            now,
            id,
        )
        .execute(db)
        .await?;
    }

    Ok(())
}

pub async fn update_cubelit_environment(
    db: &SqlitePool,
    id: &str,
    environment: &str,
) -> Result<(), AppError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query!(
        "UPDATE cubelits SET environment = ?, updated_at = ? WHERE id = ?",
        environment,
        now,
        id,
    )
    .execute(db)
    .await?;
    Ok(())
}

pub async fn update_cubelit_sidecar(
    db: &SqlitePool,
    id: &str,
    sidecar_container_id: &str,
    sidecar_image: &str,
) -> Result<(), AppError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query!(
        "UPDATE cubelits SET sidecar_container_id = ?, sidecar_image = ?, updated_at = ? WHERE id = ?",
        sidecar_container_id,
        sidecar_image,
        now,
        id,
    )
    .execute(db)
    .await?;
    Ok(())
}

pub async fn update_cubelit_name(
    db: &SqlitePool,
    id: &str,
    name: &str,
) -> Result<(), AppError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query!(
        "UPDATE cubelits SET name = ?, updated_at = ? WHERE id = ?",
        name,
        now,
        id,
    )
    .execute(db)
    .await?;
    Ok(())
}

pub async fn delete_cubelit(db: &SqlitePool, id: &str) -> Result<(), AppError> {
    sqlx::query!("DELETE FROM cubelits WHERE id = ?", id)
        .execute(db)
        .await?;

    Ok(())
}
