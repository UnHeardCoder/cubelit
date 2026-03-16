import { invoke } from "@tauri-apps/api/core";
import type { Cubelit, CreateServerConfig } from "$lib/types/server";

export async function listCubelits(): Promise<Cubelit[]> {
  return invoke<Cubelit[]>("list_cubelits");
}

export async function getCubelit(id: string): Promise<Cubelit> {
  return invoke<Cubelit>("get_cubelit", { id });
}

export async function createServer(config: CreateServerConfig): Promise<Cubelit> {
  return invoke<Cubelit>("create_server", { config });
}

export async function startServer(id: string): Promise<void> {
  return invoke("start_server", { id });
}

export async function stopServer(id: string): Promise<void> {
  return invoke("stop_server", { id });
}

export async function restartServer(id: string): Promise<void> {
  return invoke("restart_server", { id });
}

export async function deleteServer(id: string, deleteData: boolean): Promise<void> {
  return invoke("delete_server", { id, deleteData });
}

export async function syncServerStatus(id: string): Promise<Cubelit> {
  return invoke<Cubelit>("sync_server_status", { id });
}

export async function syncAllStatuses(): Promise<Cubelit[]> {
  return invoke<Cubelit[]>("sync_all_statuses");
}

export async function renameServer(id: string, name: string): Promise<Cubelit> {
  return invoke<Cubelit>("rename_server", { id, name });
}

