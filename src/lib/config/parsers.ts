/**
 * Format-aware get/set helpers for editing known server config files.
 *
 * Two formats support friendly per-field editing:
 *   - `properties` — flat `key=value` lines (Minecraft `server.properties`).
 *   - `ini`        — sectioned `[Section]` + `key=value` (ARK `Game.ini` etc.).
 *
 * Everything else (`cfg`, `lua`, `json`, `text`) is edited as raw text by the
 * UI, so it has no structured parser here.
 *
 * The `ini` helpers are lifted verbatim (behavior-preserving) from the old ARK
 * special-case in `GenericDashboard.svelte` so the migration can't regress.
 */

export type ConfigFormat = "properties" | "ini" | "cfg" | "lua" | "json" | "text";

/** Formats whose individual keys can be parsed into friendly form fields. */
export function supportsFields(format: string): boolean {
  return format === "properties" || format === "ini";
}

/**
 * Read a field's value. For `ini`, `section` is the bracketed header
 * (e.g. `"[ServerSettings]"`). Returns `null` when the key is absent.
 */
export function getField(
  content: string,
  format: string,
  key: string,
  section?: string | null,
): string | null {
  if (format === "ini") return parseIniValue(content, section ?? "", key);
  if (format === "properties") return parsePropertiesValue(content, key);
  return null;
}

/** Return a new copy of `content` with `key` set to `value`. */
export function setField(
  content: string,
  format: string,
  key: string,
  value: string,
  section?: string | null,
): string {
  if (format === "ini") return upsertIniValue(content, section ?? "", key, value);
  if (format === "properties") return upsertPropertiesValue(content, key, value);
  return content;
}

/** Coerce a stored string into a boolean (`"true"`/`"TRUE"` → true). */
export function asBool(value: string | null, fallback = false): boolean {
  if (value === null) return fallback;
  return value.toLowerCase() === "true";
}

// ─── properties ───────────────────────────────────────────────────────────

function parsePropertiesValue(content: string, key: string): string | null {
  for (const rawLine of content.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line || line.startsWith("#") || line.startsWith("!")) continue;
    const index = line.indexOf("=");
    if (index === -1) continue;
    if (line.slice(0, index).trim() === key) return line.slice(index + 1).trim();
  }
  return null;
}

function upsertPropertiesValue(content: string, key: string, value: string): string {
  const lines = content.replace(/\r\n/g, "\n").split("\n");
  for (let i = 0; i < lines.length; i += 1) {
    const line = lines[i].trim();
    if (!line || line.startsWith("#") || line.startsWith("!")) continue;
    const index = line.indexOf("=");
    if (index !== -1 && line.slice(0, index).trim() === key) {
      lines[i] = `${key}=${value}`;
      return lines.join("\n");
    }
  }
  // Append, ensuring a trailing newline before the new key.
  const prefix = content.trim().length > 0 ? content.replace(/\r\n/g, "\n").replace(/\n*$/, "\n") : "";
  return `${prefix}${key}=${value}\n`;
}

// ─── ini (sectioned) ────────────────────────────────────────────────────────

export function parseIniValue(content: string, section: string, key: string): string | null {
  let inSection = false;
  for (const rawLine of content.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line || line.startsWith(";") || line.startsWith("#")) continue;
    if (line.startsWith("[") && line.endsWith("]")) {
      inSection = line === section;
      continue;
    }
    if (!inSection) continue;
    const index = line.indexOf("=");
    if (index === -1) continue;
    if (line.slice(0, index).trim() === key) return line.slice(index + 1).trim();
  }
  return null;
}

export function upsertIniValue(content: string, section: string, key: string, value: string): string {
  const lines = content.replace(/\r\n/g, "\n").split("\n");
  let sectionStart = -1;
  let sectionEnd = lines.length;

  for (let i = 0; i < lines.length; i += 1) {
    if (lines[i].trim() === section) {
      sectionStart = i;
      for (let j = i + 1; j < lines.length; j += 1) {
        const line = lines[j].trim();
        if (line.startsWith("[") && line.endsWith("]")) {
          sectionEnd = j;
          break;
        }
      }
      break;
    }
  }

  if (sectionStart === -1) {
    const prefix = content.trim().length > 0 ? `${content.replace(/\r\n/g, "\n").replace(/\n*$/, "\n\n")}` : "";
    return `${prefix}${section}\n${key}=${value}\n`;
  }

  for (let i = sectionStart + 1; i < sectionEnd; i += 1) {
    const line = lines[i];
    const index = line.indexOf("=");
    if (index !== -1 && line.slice(0, index).trim() === key) {
      lines[i] = `${key}=${value}`;
      return lines.join("\n");
    }
  }

  lines.splice(sectionEnd, 0, `${key}=${value}`);
  return lines.join("\n");
}
