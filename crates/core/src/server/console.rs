//! Generic console-command transport.
//!
//! Three ways a running server can receive an admin command:
//!   * `source_rcon` — Source RCON over TCP (Minecraft, CS2, ARK, Project Zomboid).
//!   * `docker_exec` — run a fixed in-container helper (Bedrock `send-command`,
//!     Rust `rcon`, Palworld `rcon-cli`). The user's command is passed as a
//!     SINGLE trailing argument — never shell-joined — so it can't break out.
//!   * `external` / `none` — no in-app console (FiveM txAdmin, Valheim).
//!
//! [`send_server_command`] resolves the recipe's `dashboard.command` descriptor
//! and dispatches. [`send_rcon_command`] is the descriptor-free RCON primitive
//! reused by the Minecraft compatibility wrapper.

use std::collections::HashMap;
use std::path::Path;

use tokio::net::TcpStream;

use crate::db::queries;
use crate::error::CoreError;
use crate::recipes::{self, Recipe};

use super::minecraft::{read_rcon_packet, send_rcon_packet};

/// Dispatch a console command using the server's recipe metadata.
///
/// Returns `CoreError::Validation` for every user-correctable failure (server
/// stopped, no console support, RCON not mapped, auth failed, exec failed).
pub async fn send_server_command(
    docker: &bollard::Docker,
    db: &sqlx::SqlitePool,
    recipes_dir: &Path,
    id: &str,
    command: &str,
) -> Result<String, CoreError> {
    let server = queries::get_cubelit(db, id).await?;
    if server.status != "running" {
        return Err(CoreError::Validation("Server is not running".into()));
    }

    let recipe = recipes::get_recipe(recipes_dir, &server.recipe_id)?;
    let cmd_meta = recipe
        .dashboard
        .as_ref()
        .and_then(|d| d.command.as_ref())
        .ok_or_else(|| {
            CoreError::Validation("This server type has no in-app console".into())
        })?;

    match cmd_meta.mode.as_str() {
        "source_rcon" => {
            let port_key = resolve_rcon_port_key(&recipe, cmd_meta.port_role.as_deref())
                .unwrap_or_else(|| "25575/tcp".to_string());
            let password_env = cmd_meta.password_env.as_deref().unwrap_or("RCON_PASSWORD");
            let password_default = cmd_meta.password_default.as_deref().unwrap_or("");
            send_rcon_command(db, id, &port_key, password_env, password_default, command).await
        }
        "docker_exec" => {
            if cmd_meta.exec_template.is_empty() {
                return Err(CoreError::Validation(
                    "Recipe declares docker_exec but no exec_template".into(),
                ));
            }
            let container_id = server.container_id.clone().ok_or_else(|| {
                CoreError::Validation("No container associated with this server".into())
            })?;
            exec_command(
                docker,
                &container_id,
                &cmd_meta.exec_template,
                cmd_meta.exec_user.as_deref(),
                command,
            )
            .await
        }
        "external" | "none" => Err(CoreError::Validation(
            "This server is managed through an external panel, not an in-app console".into(),
        )),
        other => Err(CoreError::Validation(format!(
            "Unknown console command mode '{other}'"
        ))),
    }
}

/// Resolve `"<container_port>/<protocol>"` for the recipe port carrying the
/// given semantic role. Returns `None` if no port matches (caller falls back).
fn resolve_rcon_port_key(recipe: &Recipe, port_role: Option<&str>) -> Option<String> {
    let role = port_role?;
    recipe
        .ports
        .iter()
        .find(|p| p.role.as_deref() == Some(role))
        .map(|p| format!("{}/{}", p.container_port, p.protocol))
}

/// Send one command over Source RCON to `127.0.0.1:<host-port>` and return the
/// body of the response packet.
///
/// `port_key` is the `"<container_port>/<protocol>"` key into the server's
/// persisted `port_mappings`; `password_env` / `password_default` resolve the
/// RCON password from the server's environment.
pub async fn send_rcon_command(
    db: &sqlx::SqlitePool,
    id: &str,
    port_key: &str,
    password_env: &str,
    password_default: &str,
    command: &str,
) -> Result<String, CoreError> {
    let server = queries::get_cubelit(db, id).await?;

    if server.status != "running" {
        return Err(CoreError::Validation("Server is not running".into()));
    }

    let ports: HashMap<String, serde_json::Value> =
        serde_json::from_str(&server.port_mappings).unwrap_or_default();
    let rcon_port = ports
        .get(port_key)
        .and_then(|v| v.as_u64())
        .map(|p| p as u16)
        .ok_or_else(|| {
            CoreError::Validation(format!(
                "RCON port ({port_key}) is not mapped on this server"
            ))
        })?;

    let env: HashMap<String, String> =
        serde_json::from_str(&server.environment).unwrap_or_default();
    let password = env
        .get(password_env)
        .cloned()
        .unwrap_or_else(|| password_default.to_string());

    let addr = format!("127.0.0.1:{}", rcon_port);

    let mut stream = TcpStream::connect(&addr).await.map_err(|e| {
        CoreError::Validation(format!(
            "Cannot connect to RCON at {} — is the server fully started? ({})",
            addr, e
        ))
    })?;

    let exchange = async {
        // Authenticate (packet type 3). Valve-conformant servers (CS2, Project
        // Zomboid) send an empty RESPONSE_VALUE (type 0) *before* the real
        // AUTH_RESPONSE (type 2) — skip until the auth response arrives.
        send_rcon_packet(&mut stream, 1, 3, &password).await?;
        let mut auth_id = None;
        for _ in 0..4 {
            let (id, ptype, _) = read_rcon_packet(&mut stream).await?;
            if ptype == 2 {
                auth_id = Some(id);
                break;
            }
        }
        let auth_id = auth_id.ok_or_else(|| {
            CoreError::Validation("RCON server sent no auth response".into())
        })?;
        if auth_id == -1 {
            return Err(CoreError::Validation(format!(
                "RCON authentication failed — wrong {password_env}?"
            )));
        }

        // Send command (packet type 2); the output arrives as RESPONSE_VALUE
        // (type 0). Skip any stray leftover packets of other types.
        send_rcon_packet(&mut stream, 2, 2, command).await?;
        for _ in 0..4 {
            let (_, ptype, body) = read_rcon_packet(&mut stream).await?;
            if ptype == 0 {
                return Ok(body);
            }
        }
        Err(CoreError::Validation(
            "RCON server sent no command response".into(),
        ))
    };

    // A wedged server that accepts the TCP connection but never replies must
    // not hang the IPC call forever.
    tokio::time::timeout(std::time::Duration::from_secs(10), exchange)
        .await
        .map_err(|_| {
            CoreError::Validation("RCON timed out waiting for a response".into())
        })?
}

/// Run `exec_template + [command]` inside the container and return its combined
/// stdout/stderr. The user `command` is always a single argv element, so no
/// shell metacharacters are ever interpreted.
pub async fn exec_command(
    docker: &bollard::Docker,
    container_id: &str,
    exec_template: &[String],
    exec_user: Option<&str>,
    command: &str,
) -> Result<String, CoreError> {
    use bollard::exec::{CreateExecOptions, StartExecResults};
    use futures_util::StreamExt;

    let mut argv: Vec<String> = exec_template.to_vec();
    argv.push(command.to_string());

    let exchange = async {
        let exec = docker
            .create_exec(
                container_id,
                CreateExecOptions {
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    cmd: Some(argv),
                    user: exec_user.map(|u| u.to_string()),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| {
                CoreError::Validation(format!("Failed to start console command: {e}"))
            })?;

        let mut output = String::new();
        match docker
            .start_exec(&exec.id, None)
            .await
            .map_err(|e| CoreError::Validation(format!("Failed to run console command: {e}")))?
        {
            StartExecResults::Attached { output: mut stream, .. } => {
                while let Some(item) = stream.next().await {
                    match item {
                        Ok(log) => output.push_str(&log.to_string()),
                        Err(_) => break,
                    }
                }
            }
            StartExecResults::Detached => {}
        }

        Ok(output.trim_end().to_string())
    };

    // Same requirement as the RCON path: a helper that never exits (blocked on
    // stdin, broken entrypoint) must not hang the IPC call forever.
    tokio::time::timeout(std::time::Duration::from_secs(30), exchange)
        .await
        .map_err(|_| {
            CoreError::Validation("Console command timed out waiting for a response".into())
        })?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipes::{Recipe, RecipePort};

    fn recipe_with_ports(ports: Vec<RecipePort>) -> Recipe {
        Recipe {
            id: "t".into(),
            name: "T".into(),
            description: String::new(),
            icon: "t".into(),
            docker_image: "img".into(),
            default_tag: "1".into(),
            ports,
            environment: vec![],
            volumes: vec![],
            config_files: vec![],
            mods: None,
            available: true,
            estimated_disk_mb: 0,
            tags: vec![],
            server_cmd: None,
            cap_add: vec![],
            readiness: None,
            dashboard: None,
            seed_files: vec![],
        }
    }

    fn port(cp: u16, proto: &str, role: Option<&str>) -> RecipePort {
        RecipePort {
            container_port: cp,
            default_host_port: cp,
            protocol: proto.into(),
            label: "p".into(),
            role: role.map(|r| r.to_string()),
        }
    }

    #[test]
    fn resolve_rcon_port_key_matches_role() {
        let r = recipe_with_ports(vec![
            port(27015, "udp", Some("game")),
            port(27020, "tcp", Some("rcon")),
        ]);
        assert_eq!(
            resolve_rcon_port_key(&r, Some("rcon")),
            Some("27020/tcp".to_string())
        );
    }

    #[test]
    fn resolve_rcon_port_key_none_when_no_role_or_match() {
        let r = recipe_with_ports(vec![port(27015, "tcp", Some("game"))]);
        assert_eq!(resolve_rcon_port_key(&r, None), None);
        assert_eq!(resolve_rcon_port_key(&r, Some("rcon")), None);
    }
}
