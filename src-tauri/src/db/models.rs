use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Cubelit {
    pub id: String,
    pub name: String,
    pub game: String,
    pub recipe_id: String,
    pub docker_image: String,
    pub container_id: Option<String>,
    pub status: String,
    pub port_mappings: String,
    pub environment: String,
    pub volume_path: String,
    pub container_mount_path: String,
    pub sidecar_container_id: Option<String>,
    pub sidecar_image: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CubelitStatus {
    Created,
    Running,
    Stopped,
    Error,
}

impl CubelitStatus {
    pub fn as_str(&self) -> &str {
        match self {
            CubelitStatus::Created => "created",
            CubelitStatus::Running => "running",
            CubelitStatus::Stopped => "stopped",
            CubelitStatus::Error => "error",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "running" => CubelitStatus::Running,
            "stopped" => CubelitStatus::Stopped,
            "error" => CubelitStatus::Error,
            _ => CubelitStatus::Created,
        }
    }
}
