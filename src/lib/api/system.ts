import { invoke } from "@tauri-apps/api/core";
import type { OnboardingStatus, WslStatus } from "$lib/types/docker";

/** Checks whether a host port is currently available for binding. */
export async function checkPort(port: number): Promise<boolean> {
  return invoke<boolean>("check_port", { port });
}

/** Suggests the next available host port starting from the requested default. */
export async function suggestPort(defaultPort: number): Promise<number> {
  return invoke<number>("suggest_port", { defaultPort });
}

/** Fetches the combined Docker and platform onboarding diagnostics for the setup gate. */
export async function getOnboardingStatus(): Promise<OnboardingStatus> {
  return invoke<OnboardingStatus>("get_onboarding_status");
}

/** Returns the legacy Windows-only WSL readiness payload used by older flows. */
export async function checkWslStatus(): Promise<WslStatus> {
  return invoke<WslStatus>("check_wsl_status");
}

/** Launches the Windows helper that enables WSL-related optional features. */
export async function enableWsl2(): Promise<void> {
  return invoke("enable_wsl2");
}

/** Sets WSL version 2 as the default for new Linux distributions on Windows. */
export async function setWslDefaultVersion(): Promise<void> {
  return invoke("set_wsl_default_version");
}
