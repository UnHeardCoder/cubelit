export interface RecipeSummary {
  id: string;
  name: string;
  description: string;
  icon: string;
  available: boolean;
  tags: string[];
}

export interface Recipe {
  id: string;
  name: string;
  description: string;
  icon: string;
  available: boolean;
  docker_image: string;
  default_tag: string;
  ports: RecipePort[];
  environment: RecipeEnvVar[];
  volumes: RecipeVolume[];
  config_files: RecipeConfigFile[];
  mods: RecipeMods | null;
  estimated_disk_mb: number;
  tags: string[];
  /** Optional Docker CMD override passed verbatim to the container entrypoint. */
  server_cmd?: string[];
  /** Extra Linux capabilities granted to the primary container (HostConfig.cap_add). */
  cap_add?: string[];
  /** Optional dashboard behavior metadata (console / file tabs). */
  dashboard?: RecipeDashboard | null;
  /** Files written into the fresh volume before first boot ({ENV} tokens substituted). */
  seed_files?: RecipeSeedFile[];
}

export interface RecipeSeedFile {
  path: string;
  content: string;
}

export interface RecipePort {
  container_port: number;
  default_host_port: number;
  protocol: string;
  label: string;
  /** Semantic role: game | query | admin | web | rcon | rest. */
  role?: string | null;
}

export interface RecipeDashboard {
  command?: RecipeCommand | null;
  file_tabs?: RecipeFileTab[];
}

export type CommandMode = "source_rcon" | "docker_exec" | "external" | "none";

export interface RecipeCommand {
  mode: CommandMode;
  port_role?: string | null;
  password_env?: string | null;
  password_default?: string | null;
  exec_template?: string[];
  exec_user?: string | null;
  /** Read-only command the smoke harness sends to verify the console. */
  probe?: string | null;
  quick_commands?: RecipeQuickCommand[];
}

export interface RecipeQuickCommand {
  label: string;
  command: string;
}

export interface RecipeFileTab {
  path: string;
  label: string;
  file_types?: string[];
  upload?: boolean;
}

export interface RecipeEnvVar {
  key: string;
  default_value: string;
  label: string;
  type: "string" | "select" | "number" | "boolean" | "ram";
  options: string[];
}

export interface RecipeVolume {
  container_path: string;
  label: string;
}

export type ConfigFormat = "properties" | "ini" | "cfg" | "lua" | "json" | "text";

export interface RecipeConfigFile {
  path: string;
  format: ConfigFormat | string;
  label: string;
  fields?: RecipeConfigField[];
}

export interface RecipeConfigField {
  key: string;
  label: string;
  type: "string" | "number" | "boolean" | "select";
  /** For ini files: the section header, e.g. "[ServerSettings]". */
  section?: string | null;
  options?: string[];
  min?: number | null;
  max?: number | null;
  step?: number | null;
  default?: string | null;
}

export interface RecipeMods {
  supported: boolean;
  path: string | null;
  file_types: string[];
}
