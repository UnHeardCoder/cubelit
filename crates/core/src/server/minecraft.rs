//! Game-specific helpers that didn't fit on `ServerLifecycle` either because
//! they're pure (RCON wire format) or because they only apply to Minecraft.
//!
//! `send_minecraft_command` and `backup_server` are wired to the trait; the
//! RCON helpers and `copy_dir_recursive` are private implementation details.

use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::db::queries;
use crate::error::CoreError;

// ─── RCON helpers ────────────────────────────────────────────────────────────

async fn send_rcon_packet(
    stream: &mut TcpStream,
    req_id: i32,
    pkt_type: i32,
    body: &str,
) -> Result<(), CoreError> {
    let body_bytes = body.as_bytes();
    // length field covers: req_id (4) + type (4) + body + null terminator (1) + padding (1)
    let length = (4 + 4 + body_bytes.len() + 2) as i32;

    let mut packet = Vec::with_capacity(4 + length as usize);
    packet.extend_from_slice(&length.to_le_bytes());
    packet.extend_from_slice(&req_id.to_le_bytes());
    packet.extend_from_slice(&pkt_type.to_le_bytes());
    packet.extend_from_slice(body_bytes);
    packet.push(0); // null terminator
    packet.push(0); // padding

    stream.write_all(&packet).await?;
    Ok(())
}

async fn read_rcon_packet(stream: &mut TcpStream) -> Result<(i32, i32, String), CoreError> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let length = i32::from_le_bytes(len_buf) as usize;

    let mut data = vec![0u8; length];
    stream.read_exact(&mut data).await?;

    if data.len() < 8 {
        return Err(CoreError::Validation("Malformed RCON packet".into()));
    }

    let req_id = i32::from_le_bytes(data[0..4].try_into().unwrap());
    let pkt_type = i32::from_le_bytes(data[4..8].try_into().unwrap());

    // Body: bytes 8 .. length-2 (strip null terminator + padding)
    let body_end = length.saturating_sub(2);
    let body = if body_end > 8 {
        String::from_utf8_lossy(&data[8..body_end]).to_string()
    } else {
        String::new()
    };

    Ok((req_id, pkt_type, body))
}

// ─── Public lifecycle entrypoints ────────────────────────────────────────────

/// Send a command to a running Minecraft server via RCON and return the response.
///
/// Resolves the RCON host port from the server's persisted `port_mappings`
/// (looking for `25575/tcp`) and the password from its `RCON_PASSWORD` env
/// var (defaulting to `"minecraft"` — itzg's default). Returns
/// `CoreError::Validation` for any user-correctable failure mode (server
/// not running, RCON port not mapped, auth failed, network refused).
pub async fn send_minecraft_command(
    db: &sqlx::SqlitePool,
    id: &str,
    command: &str,
) -> Result<String, CoreError> {
    let server = queries::get_cubelit(db, id).await?;

    if server.status != "running" {
        return Err(CoreError::Validation("Server is not running".into()));
    }

    // Resolve RCON host port from port_mappings {"25575/tcp": <host_port>}
    let ports: HashMap<String, serde_json::Value> =
        serde_json::from_str(&server.port_mappings).unwrap_or_default();
    let rcon_port = ports
        .get("25575/tcp")
        .and_then(|v| v.as_u64())
        .map(|p| p as u16)
        .ok_or_else(|| {
            CoreError::Validation("RCON port (25575) is not mapped on this server".into())
        })?;

    // RCON password — falls back to itzg default if not in environment
    let env: HashMap<String, String> =
        serde_json::from_str(&server.environment).unwrap_or_default();
    let password = env
        .get("RCON_PASSWORD")
        .cloned()
        .unwrap_or_else(|| "minecraft".to_string());

    let addr = format!("127.0.0.1:{}", rcon_port);

    let mut stream = TcpStream::connect(&addr).await.map_err(|e| {
        CoreError::Validation(format!(
            "Cannot connect to RCON at {} — is the server fully started? ({})",
            addr, e
        ))
    })?;

    // Authenticate (packet type 3)
    send_rcon_packet(&mut stream, 1, 3, &password).await?;
    let (auth_id, _, _) = read_rcon_packet(&mut stream).await?;
    if auth_id == -1 {
        return Err(CoreError::Validation(
            "RCON authentication failed — wrong RCON_PASSWORD?".into(),
        ));
    }

    // Send command (packet type 2)
    send_rcon_packet(&mut stream, 2, 2, command).await?;
    let (_, _, response) = read_rcon_packet(&mut stream).await?;

    Ok(response)
}

/// Copy the server's data directory to a timestamped backup folder.
/// Returns the path of the created backup.
///
/// Backups are placed next to the server data folder under `<name>-backups/`
/// — keeping them on the same volume avoids cross-device copies but leaves
/// disk-space management to the user. Server name is sanitised by lowercasing
/// and replacing every non-alphanumeric character with `-` so the backup
/// folder is portable across filesystems.
pub async fn backup_server(db: &sqlx::SqlitePool, id: &str) -> Result<String, CoreError> {
    let server = queries::get_cubelit(db, id).await?;

    let src = std::path::Path::new(&server.volume_path);
    if !src.exists() {
        return Err(CoreError::Validation(
            "Server data directory not found".into(),
        ));
    }

    let timestamp = chrono::Local::now()
        .format("%Y-%m-%d_%H-%M-%S")
        .to_string();

    // Sanitise server name for use in a folder name
    let safe_name: String = server
        .name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
        .collect::<String>()
        .to_lowercase();

    // Backups live next to the server data folder: <parent>/<name>-backups/
    let backups_dir = src
        .parent()
        .unwrap_or(src)
        .join(format!("{}-backups", safe_name));
    let backup_path = backups_dir.join(format!("{}-{}", safe_name, timestamp));

    std::fs::create_dir_all(&backups_dir)?;
    copy_dir_recursive(src, &backup_path)?;

    Ok(backup_path.to_string_lossy().to_string())
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<(), CoreError> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}
