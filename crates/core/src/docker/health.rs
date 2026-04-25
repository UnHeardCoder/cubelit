use bollard::Docker;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DockerStatus {
    pub available: bool,
    pub version: Option<String>,
    pub error: Option<String>,
}

pub async fn check_docker_status(docker: &Docker) -> DockerStatus {
    match docker.ping().await {
        Ok(_) => match docker.version().await {
            Ok(version) => DockerStatus {
                available: true,
                version: version.version,
                error: None,
            },
            Err(e) => DockerStatus {
                available: false,
                version: None,
                error: Some(e.to_string()),
            },
        },
        Err(e) => DockerStatus {
            available: false,
            version: None,
            error: Some(e.to_string()),
        },
    }
}
