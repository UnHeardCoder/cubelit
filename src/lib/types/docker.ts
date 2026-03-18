export interface DockerStatus {
  available: boolean;
  version: string | null;
  error: string | null;
}

export interface WslStatus {
  wsl_installed: boolean;
  wsl2_enabled: boolean;
  reboot_required: boolean;
}

export interface ContainerStats {
  cpu_percent: number;
  memory_usage_mb: number;
  memory_limit_mb: number;
}
