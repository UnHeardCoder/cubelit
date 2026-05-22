use std::collections::{HashMap, HashSet};

use bollard::models::{
    ContainerCreateBody, HostConfig, PortBinding, RestartPolicy, RestartPolicyNameEnum,
};
use bollard::query_parameters::{
    CreateContainerOptions, RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
};
use bollard::Docker;

use crate::db::models::Cubelit;
use crate::error::CoreError;

pub async fn create_container(
    docker: &Docker,
    cubelit: &Cubelit,
    extra_binds: &[String],
    server_cmd: Option<Vec<String>>,
) -> Result<String, CoreError> {
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

    // bollard 0.20: exposed_ports is Vec<String> (port keys only, e.g. "25565/tcp").
    let mut exposed_ports: Vec<String> = Vec::new();
    let mut port_bindings: HashMap<String, Option<Vec<PortBinding>>> = HashMap::new();

    for (container_port, host_port) in &port_mappings {
        let port_key = if container_port.contains('/') {
            container_port.clone()
        } else {
            format!("{}/tcp", container_port)
        };

        exposed_ports.push(port_key.clone());
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

    let binds = build_container_binds(cubelit, extra_binds);

    let host_config = HostConfig {
        port_bindings: Some(port_bindings),
        binds: Some(binds),
        restart_policy: Some(RestartPolicy {
            name: Some(RestartPolicyNameEnum::UNLESS_STOPPED),
            maximum_retry_count: None,
        }),
        ..Default::default()
    };

    let config = ContainerCreateBody {
        image: Some(cubelit.docker_image.clone()),
        env: Some(env),
        exposed_ports: Some(exposed_ports),
        host_config: Some(host_config),
        labels: Some(labels),
        cmd: server_cmd,
        ..Default::default()
    };

    let container_name = format!("cubelit-{}", cubelit.id);
    let options = CreateContainerOptions {
        name: Some(container_name),
        platform: String::from(""),
    };

    let response = docker.create_container(Some(options), config).await?;
    Ok(response.id)
}

fn build_container_binds(cubelit: &Cubelit, extra_binds: &[String]) -> Vec<String> {
    let primary_bind = format!("{}:{}", cubelit.volume_path, cubelit.container_mount_path);
    let mut seen_targets = HashSet::new();
    let mut binds = Vec::with_capacity(1 + extra_binds.len());

    for bind in std::iter::once(primary_bind).chain(extra_binds.iter().cloned()) {
        if let Some(target) = bind_container_target(&bind) {
            if !seen_targets.insert(target.to_string()) {
                continue;
            }
        }
        binds.push(bind);
    }

    binds
}

fn bind_container_target(bind: &str) -> Option<&str> {
    let mut parts = bind.rsplit(':');
    let last = parts.next()?;
    let target = if last.starts_with('/') {
        last
    } else {
        parts.next()?
    };

    target.starts_with('/').then_some(target)
}

pub async fn start_container(docker: &Docker, container_id: &str) -> Result<(), CoreError> {
    docker
        .start_container(container_id, None::<StartContainerOptions>)
        .await?;
    Ok(())
}

pub async fn stop_container(docker: &Docker, container_id: &str) -> Result<(), CoreError> {
    docker
        .stop_container(
            container_id,
            Some(StopContainerOptions {
                t: Some(10),
                ..Default::default()
            }),
        )
        .await?;
    Ok(())
}

pub async fn restart_container(docker: &Docker, container_id: &str) -> Result<(), CoreError> {
    docker
        .restart_container(
            container_id,
            Some(bollard::query_parameters::RestartContainerOptions {
                t: Some(10),
                ..Default::default()
            }),
        )
        .await?;
    Ok(())
}

pub async fn remove_container(docker: &Docker, container_id: &str) -> Result<(), CoreError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cubelit() -> Cubelit {
        Cubelit {
            id: "test-id".to_string(),
            name: "Test Server".to_string(),
            game: "Test Game".to_string(),
            recipe_id: "test".to_string(),
            docker_image: "test/image:latest".to_string(),
            container_id: None,
            status: "created".to_string(),
            port_mappings: "{}".to_string(),
            environment: "{}".to_string(),
            volume_path: "/srv/cubelit/test".to_string(),
            container_mount_path: "/data".to_string(),
            sidecar_container_id: None,
            sidecar_image: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn build_container_binds_keeps_primary_volume_first() {
        let cubelit = test_cubelit();
        let binds = build_container_binds(
            &cubelit,
            &[
                "/srv/cubelit/test/config:/config".to_string(),
                "/srv/cubelit/test/logs:/logs".to_string(),
            ],
        );

        assert_eq!(
            binds,
            vec![
                "/srv/cubelit/test:/data",
                "/srv/cubelit/test/config:/config",
                "/srv/cubelit/test/logs:/logs",
            ]
        );
    }

    #[test]
    fn build_container_binds_skips_duplicate_container_targets() {
        let cubelit = test_cubelit();
        let binds = build_container_binds(
            &cubelit,
            &[
                "/other/data:/data".to_string(),
                "/srv/cubelit/test/config:/config".to_string(),
                "/other/config:/config:ro".to_string(),
            ],
        );

        assert_eq!(
            binds,
            vec![
                "/srv/cubelit/test:/data",
                "/srv/cubelit/test/config:/config",
            ]
        );
    }

    #[test]
    fn build_container_binds_handles_windows_style_host_paths() {
        let cubelit = test_cubelit();
        let binds = build_container_binds(
            &cubelit,
            &[
                r"C:\cubelit\config:/config".to_string(),
                r"D:\other\config:/config:ro".to_string(),
            ],
        );

        assert_eq!(
            binds,
            vec!["/srv/cubelit/test:/data", r"C:\cubelit\config:/config"]
        );
    }
}
