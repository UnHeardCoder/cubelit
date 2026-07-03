import { invoke } from "@tauri-apps/api/core";

/** Send a command to the Minecraft server via RCON. Returns the server's response text. */
export async function sendMinecraftCommand(
  id: string,
  command: string,
): Promise<string> {
  return invoke<string>("send_minecraft_command", { id, command });
}

/**
 * Send a console command to any server, dispatched by the recipe's
 * `dashboard.command` descriptor (Source RCON / docker exec / external).
 * Returns the server's response text.
 */
export async function sendServerCommand(
  id: string,
  command: string,
): Promise<string> {
  return invoke<string>("send_server_command", { id, command });
}

/** Copy the server data directory to a timestamped backup folder. Returns the backup path. */
export async function backupServer(id: string): Promise<string> {
  return invoke<string>("backup_server", { id });
}
