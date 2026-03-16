import { invoke } from "@tauri-apps/api/core";
import type { DockerStatus, ContainerStats } from "$lib/types/docker";

export async function checkDockerStatus(): Promise<DockerStatus> {
  return invoke<DockerStatus>("check_docker_status");
}

export async function getServerStats(id: string): Promise<ContainerStats> {
  return invoke<ContainerStats>("get_server_stats", { id });
}

export async function updateServerSettings(
  id: string,
  environment: Record<string, string>
): Promise<void> {
  return invoke("update_server_settings", { id, environment });
}
