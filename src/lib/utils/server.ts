export function parsePorts(json: string): Record<string, number> {
  try { return JSON.parse(json); } catch { return {}; }
}

export function parseEnv(json: string): Record<string, string> {
  try { return JSON.parse(json); } catch { return {}; }
}

export const STATUS_COLORS: Record<string, string> = {
  starting: "text-cubelit-warning",
  running:  "text-cubelit-success",
  stopped:  "text-cubelit-error",
  created:  "text-cubelit-warning",
  error:    "text-cubelit-error",
};

export const STATUS_LABELS: Record<string, string> = {
  starting: "Starting",
  running:  "Online",
  stopped:  "Offline",
  created:  "Created",
  error:    "Error",
};
