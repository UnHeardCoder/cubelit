use std::collections::HashMap;

use bollard::container::{
    Config, CreateContainerOptions, RemoveContainerOptions, StartContainerOptions,
    StopContainerOptions,
};
use bollard::models::{HostConfig, PortBinding, RestartPolicy, RestartPolicyNameEnum};
use bollard::Docker;

use crate::db::models::Cubelit;
use crate::error::AppError;

pub async fn create_container(docker: &Docker, cubelit: &Cubelit, extra_binds: &[String]) -> Result<String, AppError> {
    let port_mappings: HashMap<String, u16> =
        serde_json::from_str(&cubelit.port_mappings).unwrap_or_default();

    let env_vars: HashMap<String, String> =
        serde_json::from_str(&cubelit.environment).unwrap_or_default();

    // Filter out internal/incompatible keys that must not be Docker env vars.
    // FRAMEWORK: Cubelit-only metadata.
    // LICENSE_KEY: must not be present when NO_DEFAULT_CONFIG=1 (txAdmin mode) —
    //   the spritsail/fivem entrypoint exits with error if both are set.
    const FILTERED_KEYS: &[&str] = &["FRAMEWORK", "LICENSE_KEY", "DB_HOST_PORT"];
    let env: Vec<String> = env_vars
        .iter()
        .filter(|(k, _)| !FILTERED_KEYS.contains(&k.as_str()))
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();

    let mut exposed_ports: HashMap<String, HashMap<(), ()>> = HashMap::new();
    let mut port_bindings: HashMap<String, Option<Vec<PortBinding>>> = HashMap::new();

    for (container_port, host_port) in &port_mappings {
        let port_key = if container_port.contains('/') {
            container_port.clone()
        } else {
            format!("{}/tcp", container_port)
        };

        exposed_ports.insert(port_key.clone(), HashMap::new());
        port_bindings.insert(
            port_key,
            Some(vec![PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some(host_port.to_string()),
            }]),
        );
    }

    let mut labels = HashMap::new();
    labels.insert("cubelit.id".to_string(), cubelit.id.clone());
    labels.insert("cubelit.game".to_string(), cubelit.game.clone());
    labels.insert("cubelit.managed".to_string(), "true".to_string());
    labels.insert("cubelit.role".to_string(), "primary".to_string());

    let mut binds = vec![format!("{}:{}", cubelit.volume_path, cubelit.container_mount_path)];
    binds.extend_from_slice(extra_binds);

    let host_config = HostConfig {
        port_bindings: Some(port_bindings),
        binds: Some(binds),
        restart_policy: Some(RestartPolicy {
            name: Some(RestartPolicyNameEnum::UNLESS_STOPPED),
            maximum_retry_count: None,
        }),
        ..Default::default()
    };

    let config = Config {
        image: Some(cubelit.docker_image.clone()),
        env: Some(env),
        exposed_ports: Some(exposed_ports),
        host_config: Some(host_config),
        labels: Some(labels),
        ..Default::default()
    };

    let container_name = format!("cubelit-{}", cubelit.id);
    let options = CreateContainerOptions {
        name: container_name,
        platform: None,
    };

    let response = docker.create_container(Some(options), config).await?;
    Ok(response.id)
}

pub async fn start_container(docker: &Docker, container_id: &str) -> Result<(), AppError> {
    docker
        .start_container(container_id, None::<StartContainerOptions<String>>)
        .await?;
    Ok(())
}

pub async fn stop_container(docker: &Docker, container_id: &str) -> Result<(), AppError> {
    docker
        .stop_container(container_id, Some(StopContainerOptions { t: 10 }))
        .await?;
    Ok(())
}

pub async fn restart_container(docker: &Docker, container_id: &str) -> Result<(), AppError> {
    docker.restart_container(container_id, Some(bollard::container::RestartContainerOptions { t: 10 })).await?;
    Ok(())
}

pub async fn remove_container(docker: &Docker, container_id: &str) -> Result<(), AppError> {
    docker
        .remove_container(
            container_id,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        )
        .await?;
    Ok(())
}
