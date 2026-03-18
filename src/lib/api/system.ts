import { invoke } from "@tauri-apps/api/core";
import type { WslStatus } from "$lib/types/docker";

export async function checkPort(port: number): Promise<boolean> {
  return invoke<boolean>("check_port", { port });
}

export async function suggestPort(defaultPort: number): Promise<number> {
  return invoke<number>("suggest_port", { defaultPort });
}

export async function checkWslStatus(): Promise<WslStatus> {
  return invoke<WslStatus>("check_wsl_status");
}

export async function enableWsl2(): Promise<void> {
  return invoke("enable_wsl2");
}

export async function setWslDefaultVersion(): Promise<void> {
  return invoke("set_wsl_default_version");
}
