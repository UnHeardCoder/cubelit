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
}

export interface RecipePort {
  container_port: number;
  default_host_port: number;
  protocol: string;
  label: string;
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

export interface RecipeConfigFile {
  path: string;
  format: string;
  label: string;
}

export interface RecipeMods {
  supported: boolean;
  path: string | null;
  file_types: string[];
}
