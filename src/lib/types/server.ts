export interface Cubelit {
  id: string;
  name: string;
  game: string;
  recipe_id: string;
  docker_image: string;
  container_id: string | null;
  status: CubelitStatus;
  port_mappings: string;
  environment: string;
  volume_path: string;
  container_mount_path: string;
  sidecar_container_id: string | null;
  sidecar_image: string | null;
  created_at: string;
  updated_at: string;
}

export type CubelitStatus = "created" | "starting" | "running" | "stopped" | "error";

export interface ServerCreateProgress {
  step: "preparing" | "pulling" | "creating" | "starting" | "ready";
  progress: number | null;
  message: string;
}

export interface CreateServerConfig {
  name: string;
  recipe_id: string;
  port_overrides?: Record<string, number>;
  env_overrides?: Record<string, string>;
  volume_path?: string;
  /** Override the recipe's default image tag, e.g. "java17", "java8", "latest" */
  tag_override?: string;
}
