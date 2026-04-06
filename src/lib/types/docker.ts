/** Basic Docker reachability details used by the legacy store state. */
export interface DockerStatus {
  available: boolean;
  version: string | null;
  error: string | null;
}

/** Docker availability states returned by the onboarding diagnostics command. */
export type DockerState =
  | "ready"
  | "not_installed"
  | "not_running"
  | "permission_denied"
  | "unknown";

/** Windows WSL readiness states returned by the onboarding diagnostics command. */
export type WslState =
  | "not_applicable"
  | "ok"
  | "needs_install"
  | "needs_default_v2"
  | "reboot_required"
  | "check_failed";

/** Legacy Windows WSL status shape used by older onboarding checks. */
export interface WslStatus {
  wsl_installed: boolean;
  wsl2_enabled: boolean;
  reboot_required: boolean;
}

/** Full onboarding diagnostics consumed by the Docker setup gate. */
export interface OnboardingStatus {
  platform: "windows" | "linux" | "macos";
  docker: {
    state: DockerState;
    installed: boolean | null;
    version: string | null;
    error: string | null;
  };
  wsl: {
    state: WslState;
    wsl_installed: boolean | null;
    wsl2_enabled: boolean | null;
    reboot_required: boolean;
    error: string | null;
  };
}

/** Container resource usage reported by the live stats stream. */
export interface ContainerStats {
  cpu_percent: number;
  memory_usage_mb: number;
  memory_limit_mb: number;
}
