use sqlx::SqlitePool;

use crate::db::models::Cubelit;
use crate::error::CoreError;

pub async fn insert_cubelit(db: &SqlitePool, cubelit: &Cubelit) -> Result<(), CoreError> {
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

pub async fn get_cubelit(db: &SqlitePool, id: &str) -> Result<Cubelit, CoreError> {
    let cubelit = sqlx::query_as!(
        Cubelit,
        "SELECT id, name, game, recipe_id, docker_image, container_id, status, port_mappings, environment, volume_path, container_mount_path, sidecar_container_id, sidecar_image, created_at, updated_at FROM cubelits WHERE id = ?",
        id
    )
    .fetch_optional(db)
    .await?
    .ok_or_else(|| CoreError::NotFound(format!("Cubelit with id '{}' not found", id)))?;

    Ok(cubelit)
}

pub async fn list_cubelits(db: &SqlitePool) -> Result<Vec<Cubelit>, CoreError> {
    let cubelits = sqlx::query_as!(
        Cubelit,
        "SELECT id, name, game, recipe_id, docker_image, container_id, status, port_mappings, environment, volume_path, container_mount_path, sidecar_container_id, sidecar_image, created_at, updated_at FROM cubelits ORDER BY created_at DESC"
    )
    .fetch_all(db)
    .await?;

    Ok(cubelits)
}

/// Update a server's `status`, optionally also rewriting `container_id`.
///
/// `container_id` semantics:
///   * `None`              — leave the existing column value alone.
///   * `Some(None)`        — explicitly clear it (write SQL `NULL`).
///   * `Some(Some(value))` — set it to `value`.
///
/// The two-level Option is the smallest API change that lets a caller
/// distinguish "skip" from "clear". Using a single `Option<&str>` (with
/// `None` overloaded for both) silently broke `update_server_settings`,
/// which expected `Some("")` to clear the FK but instead persisted an
/// empty string — see the v0.1.8 CodeRabbit review.
pub async fn update_cubelit_status(
    db: &SqlitePool,
    id: &str,
    status: &str,
    container_id: Option<Option<&str>>,
) -> Result<(), CoreError> {
    let now = chrono::Utc::now().to_rfc3339();

    match container_id {
        Some(cid) => {
            // sqlx binds `Option<&str>` as NULL when None, so this single
            // branch covers both "set to value" and "set to NULL".
            sqlx::query!(
                "UPDATE cubelits SET status = ?, container_id = ?, updated_at = ? WHERE id = ?",
                status,
                cid,
                now,
                id,
            )
            .execute(db)
            .await?;
        }
        None => {
            sqlx::query!(
                "UPDATE cubelits SET status = ?, updated_at = ? WHERE id = ?",
                status,
                now,
                id,
            )
            .execute(db)
            .await?;
        }
    }

    Ok(())
}

pub async fn update_cubelit_environment(
    db: &SqlitePool,
    id: &str,
    environment: &str,
) -> Result<(), CoreError> {
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
) -> Result<(), CoreError> {
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
) -> Result<(), CoreError> {
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

pub async fn delete_cubelit(db: &SqlitePool, id: &str) -> Result<(), CoreError> {
    sqlx::query!("DELETE FROM cubelits WHERE id = ?", id)
        .execute(db)
        .await?;

    Ok(())
}
