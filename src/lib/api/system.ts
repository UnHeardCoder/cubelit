import { invoke } from "@tauri-apps/api/core";

export async function checkPort(port: number): Promise<boolean> {
  return invoke<boolean>("check_port", { port });
}

export async function suggestPort(defaultPort: number): Promise<number> {
  return invoke<number>("suggest_port", { defaultPort });
}
