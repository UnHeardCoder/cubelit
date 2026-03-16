import { invoke } from "@tauri-apps/api/core";
import type { FileEntry } from "$lib/types/files";

export async function listServerFiles(
  id: string,
  subpath?: string
): Promise<FileEntry[]> {
  return invoke<FileEntry[]>("list_server_files", { id, subpath: subpath ?? null });
}

export async function copyFileToServer(
  id: string,
  sourcePath: string,
  destSubpath: string
): Promise<void> {
  return invoke("copy_file_to_server", { id, sourcePath, destSubpath });
}

export async function deleteServerFile(
  id: string,
  filepath: string
): Promise<void> {
  return invoke("delete_server_file", { id, filepath });
}

export async function getServerLogs(id: string, lines?: number): Promise<string[]> {
  return invoke<string[]>("get_server_logs", { id, lines: lines ?? 100 });
}
