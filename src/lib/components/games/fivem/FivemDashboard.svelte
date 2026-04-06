<script lang="ts">
  import type { Cubelit } from "$lib/types/server";
  import type { FileEntry } from "$lib/types/files";
  import { listServerFiles, copyFileToServer, deleteServerFile, getServerLogs } from "$lib/api/files";
  import { updateServerSettings } from "$lib/api/docker";
  import { getRecipeDetail } from "$lib/api/recipes";
  import { parsePorts, parseEnv, STATUS_COLORS } from "$lib/utils/server";
  import type { Recipe } from "$lib/types/recipe";
  import StatsCards from "$lib/components/StatsCards.svelte";
  import Button from "$lib/components/Button.svelte";
  import Modal from "$lib/components/Modal.svelte";
  import LogViewer from "$lib/components/LogViewer.svelte";

  interface Props {
    server: Cubelit;
  }

  let { server }: Props = $props();

  let activeTab = $state<"overview" | "resources" | "settings" | "logs">("overview");
  let resources = $state<FileEntry[]>([]);
  let logs = $state<string[]>([]);
  let resourcesLoading = $state(false);
  let logsLoading = $state(false);
  let resourcesError = $state<string | null>(null);
  let copiedField = $state<string | null>(null);

  let deleteResourceName = $state<string | null>(null);
  let showDeleteResourceModal = $state(false);

  // ─── Settings state ───────────────────────────────────────────────────────
  // These keys are auto-managed and should not be edited directly
  const READONLY_KEYS = ["LICENSE_KEY", "MYSQL_CONNECTION_STRING", "NO_DEFAULT_CONFIG"];
  let recipe = $state<Recipe | null>(null);
  let editEnv = $state<Record<string, string>>({});
  let settingsLoading = $state(false);
  let settingsSaving = $state(false);
  let settingsError = $state<string | null>(null);
  let showApplyModal = $state(false);

  function getConnectionString(): string {
    const env = parseEnv(server.environment);
    return env["MYSQL_CONNECTION_STRING"] ?? "N/A";
  }

  function getAddress(): string {
    const ports = parsePorts(server.port_mappings);
    const firstPort = Object.values(ports)[0];
    return firstPort ? `localhost:${firstPort}` : "";
  }

  function getTxAdminPort(): number | null {
    const ports = parsePorts(server.port_mappings);
    return ports["40120/tcp"] ?? null;
  }

  async function openTxAdmin() {
    const port = getTxAdminPort();
    if (!port) return;
    try {
      const { openUrl } = await import("@tauri-apps/plugin-opener");
      await openUrl(`http://localhost:${port}`);
    } catch (e) {
      console.error("Failed to open txAdmin:", e);
    }
  }

  async function loadLogs() {
    logsLoading = true;
    try {
      logs = await getServerLogs(server.id, 200);
    } catch {
      logs = [];
    } finally { logsLoading = false; }
  }

  async function copyToClipboard(text: string, field: string) {
    try {
      await navigator.clipboard.writeText(text);
      copiedField = field;
      setTimeout(() => copiedField = null, 2000);
    } catch { /* ignore */ }
  }

  async function loadResources() {
    resourcesLoading = true;
    resourcesError = null;
    try {
      resources = await listServerFiles(server.id, "resources");
    } catch {
      resourcesError = "Failed to load resources.";
      resources = [];
    } finally { resourcesLoading = false; }
  }

  async function uploadResource() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({ title: "Select Resource File" });
      if (selected) {
        const filename = (selected as string).split(/[/\\]/).pop() ?? "resource";
        await copyFileToServer(server.id, selected as string, `resources/${filename}`);
        await loadResources();
      }
    } catch (e) {
      console.error("Failed to upload resource:", e);
    }
  }

  async function confirmDeleteResource() {
    if (!deleteResourceName) return;
    try {
      await deleteServerFile(server.id, `resources/${deleteResourceName}`);
      await loadResources();
    } catch (e) {
      console.error("Failed to delete resource:", e);
    } finally {
      deleteResourceName = null;
      showDeleteResourceModal = false;
    }
  }

  async function loadSettings() {
    settingsLoading = true;
    settingsError = null;
    try {
      recipe = await getRecipeDetail(server.recipe_id);
      const current = parseEnv(server.environment);
      // Only expose recipe-defined fields in the edit form
      editEnv = Object.fromEntries(
        recipe.environment
          .filter(f => !READONLY_KEYS.includes(f.key))
          .map(f => [f.key, current[f.key] ?? f.default_value])
      );
    } catch {
      settingsError = "Failed to load settings.";
    } finally {
      settingsLoading = false;
    }
  }

  async function applySettings() {
    settingsSaving = true;
    settingsError = null;
    showApplyModal = false;
    // Merge editable fields back with the preserved read-only keys
    const current = parseEnv(server.environment);
    const merged: Record<string, string> = { ...current, ...editEnv };
    try {
      await updateServerSettings(server.id, merged);
    } catch (e) {
      settingsError = String(e);
    } finally {
      settingsSaving = false;
    }
  }

  $effect(() => {
    if (activeTab === "resources") loadResources();
    if (activeTab === "settings") loadSettings();
  });

  $effect(() => {
    if (activeTab === "logs") {
      loadLogs();
      if (server.status === "running") {
        const interval = setInterval(loadLogs, 3000);
        return () => clearInterval(interval);
      }
    }
  });

  const tabs = [
    { id: "overview" as const, label: "Overview" },
    { id: "resources" as const, label: "Resources" },
    { id: "settings" as const, label: "Settings" },
    { id: "logs" as const, label: "Logs" },
  ];
</script>

<!-- Apply Settings Modal -->
<Modal bind:open={showApplyModal} onclose={() => showApplyModal = false} title="Apply Settings">
  <p class="text-sm text-cubelit-muted mb-6">
    Applying settings will <span class="text-cubelit-warning font-medium">restart your server</span>. Continue?
  </p>
  <div class="flex gap-3 justify-end">
    <Button variant="ghost" onclick={() => showApplyModal = false}>Cancel</Button>
    <Button variant="primary" onclick={applySettings}>Apply & Restart</Button>
  </div>
</Modal>

<!-- Delete Resource Modal -->
<Modal bind:open={showDeleteResourceModal} onclose={() => { deleteResourceName = null; showDeleteResourceModal = false; }} title="Delete Resource">
  <p class="text-sm text-cubelit-muted mb-6">Delete <span class="text-cubelit-text font-medium">{deleteResourceName}</span>? This cannot be undone.</p>
  <div class="flex gap-3 justify-end">
    <Button variant="ghost" onclick={() => deleteResourceName = null}>Cancel</Button>
    <Button variant="danger" onclick={confirmDeleteResource}>Delete</Button>
  </div>
</Modal>

<!-- Tabs -->
<div class="flex gap-1 mb-6 border-b border-cubelit-border">
  {#each tabs as tab}
    <button
      type="button"
      class="px-4 py-2.5 text-sm font-medium transition-colors relative {activeTab === tab.id
        ? 'text-cubelit-accent'
        : 'text-cubelit-muted hover:text-cubelit-text'}"
      onclick={() => activeTab = tab.id}
    >
      {tab.label}
      {#if activeTab === tab.id}
        <div class="absolute bottom-0 left-0 right-0 h-0.5 bg-cubelit-accent rounded-t"></div>
      {/if}
    </button>
  {/each}
</div>

{#if activeTab === "overview"}
  <!-- txAdmin setup notice -->
  <div class="mb-4 bg-cubelit-accent/10 border border-cubelit-accent/30 rounded-xl p-5 space-y-3">
    <div class="flex items-start justify-between gap-4">
      <div>
        <p class="text-sm font-semibold text-cubelit-text">Complete server setup in txAdmin</p>
        <p class="text-xs text-cubelit-muted mt-1">
          FiveM is managed via txAdmin. Open the panel, complete the wizard, and deploy a server recipe to install resources and set your license key.
        </p>
      </div>
      <button
        type="button"
        class="shrink-0 flex items-center gap-1.5 px-3 py-1.5 bg-cubelit-accent hover:bg-cubelit-accent-hover text-white text-xs font-medium rounded-lg transition-colors"
        onclick={openTxAdmin}
      >
        <svg class="w-3.5 h-3.5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" d="M13.5 6H5.25A2.25 2.25 0 0 0 3 8.25v10.5A2.25 2.25 0 0 0 5.25 21h10.5A2.25 2.25 0 0 0 18 18.75V10.5m-10.5 6L21 3m0 0h-5.25M21 3v5.25" />
        </svg>
        Open txAdmin
      </button>
    </div>
    <ol class="text-xs text-cubelit-muted space-y-1 list-decimal list-inside">
      <li>Click <span class="text-cubelit-text font-medium">Open txAdmin</span> above</li>
      <li>Follow the setup wizard — enter your <span class="text-cubelit-text font-medium">FiveM license key</span> when prompted</li>
      <li>Choose a server recipe (e.g. <span class="text-cubelit-text font-mono">CFX Default</span>) to deploy resources</li>
      <li>Server restarts automatically with all resources loaded</li>
    </ol>
  </div>

  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
    <div class="bg-cubelit-surface border border-cubelit-border rounded-xl p-5">
      <h3 class="text-sm font-medium text-cubelit-muted mb-3">Status</h3>
      <p class="text-lg font-semibold capitalize {STATUS_COLORS[server.status] ?? 'text-cubelit-muted'}">
        {server.status}
      </p>
    </div>

    <div class="bg-cubelit-surface border border-cubelit-border rounded-xl p-5">
      <h3 class="text-sm font-medium text-cubelit-muted mb-3">Game Port</h3>
      <p class="text-cubelit-text font-mono text-sm">{getAddress()}</p>
    </div>

    <StatsCards serverId={server.id} serverStatus={server.status} />

    <div class="bg-cubelit-surface border border-cubelit-border rounded-xl p-5">
      <h3 class="text-sm font-medium text-cubelit-muted mb-3">Image</h3>
      <p class="text-cubelit-text text-sm font-mono">{server.docker_image}</p>
    </div>

    <div class="bg-cubelit-surface border border-cubelit-border rounded-xl p-5">
      <h3 class="text-sm font-medium text-cubelit-muted mb-3">Server Files</h3>
      <p class="text-cubelit-text text-sm font-mono truncate">{server.volume_path}</p>
    </div>

    <!-- Database Section -->
    {#if server.sidecar_container_id}
      {@const dbHost = `cubelit-${server.id}-db`}
      {@const dbPassword = parseEnv(server.environment)['DB_PASSWORD'] ?? ''}
      <div class="md:col-span-2 bg-cubelit-surface border border-cubelit-border rounded-xl p-5 space-y-3">
        <div class="flex items-center justify-between">
          <h3 class="text-sm font-medium text-cubelit-muted">Database (MariaDB)</h3>
          <span class="text-xs px-2 py-0.5 rounded-full {server.status === 'running' ? 'bg-cubelit-success/20 text-cubelit-success' : 'bg-cubelit-muted/20 text-cubelit-muted'}">
            {server.status === "running" ? "Running" : "Stopped"}
          </span>
        </div>

        <p class="text-xs text-cubelit-muted bg-cubelit-bg border border-cubelit-border rounded-lg px-3 py-2">
          Use these credentials in txAdmin — <span class="text-cubelit-warning font-medium">not localhost</span>, the database runs in a separate container.
        </p>

        <div class="space-y-2">
          {#each [
            { label: "Host", value: dbHost, field: "host" },
            { label: "Port", value: "3306", field: "port" },
            { label: "Username", value: "root", field: "user" },
            { label: "Password", value: dbPassword || "(no password)", field: "pass", copyValue: dbPassword },
            { label: "Database", value: "fivem", field: "db" },
            { label: "Connection String", value: getConnectionString().slice(0, 40) + "...", field: "conn", copyValue: getConnectionString() },
          ] as row}
            <div class="flex items-center justify-between">
              <span class="text-xs text-cubelit-muted">{row.label}</span>
	            <button
	              type="button"
	              class="text-xs font-mono text-cubelit-text hover:text-cubelit-accent transition-colors max-w-[60%] truncate"
	              onclick={() => copyToClipboard(row.copyValue ?? row.value, row.field)}
	            >
                {copiedField === row.field ? "Copied!" : row.value}
              </button>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>

{:else if activeTab === "resources"}
  <div class="space-y-4">
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-medium text-cubelit-text">Server Resources</h3>
      <Button size="sm" onclick={uploadResource}>Upload Resource</Button>
    </div>

    {#if resourcesError}
      <p class="text-xs text-cubelit-error py-4 text-center">{resourcesError}</p>
    {:else if resourcesLoading}
      <p class="text-cubelit-muted text-sm py-8 text-center">Loading resources...</p>
    {:else if resources.length === 0}
      <div class="text-center py-12 bg-cubelit-surface border border-cubelit-border rounded-xl">
        <p class="text-cubelit-muted text-sm mb-2">No resources found</p>
        <p class="text-cubelit-muted/70 text-xs">Upload resource files to add functionality to your FiveM server</p>
      </div>
    {:else}
      <div class="space-y-2">
        {#each resources as resource}
          <div class="flex items-center justify-between bg-cubelit-surface border border-cubelit-border rounded-xl px-4 py-3">
            <div class="flex items-center gap-3">
              {#if resource.is_dir}
                <svg class="w-4 h-4 text-cubelit-accent" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12.75V12A2.25 2.25 0 0 1 4.5 9.75h15A2.25 2.25 0 0 1 21.75 12v.75m-8.69-6.44-2.12-2.12a1.5 1.5 0 0 0-1.061-.44H4.5A2.25 2.25 0 0 0 2.25 6v12a2.25 2.25 0 0 0 2.25 2.25h15A2.25 2.25 0 0 0 21.75 18V9a2.25 2.25 0 0 0-2.25-2.25h-5.379a1.5 1.5 0 0 1-1.06-.44Z" />
                </svg>
              {:else}
                <svg class="w-4 h-4 text-cubelit-muted" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" />
                </svg>
              {/if}
              <div>
                <p class="text-sm text-cubelit-text">{resource.name}</p>
                {#if !resource.is_dir}
                  <p class="text-xs text-cubelit-muted">{(resource.size / 1024).toFixed(1)} KB</p>
                {/if}
              </div>
            </div>
            <button
              type="button"
              class="text-xs text-cubelit-error hover:text-cubelit-error/80 transition-colors"
              onclick={() => { deleteResourceName = resource.name; showDeleteResourceModal = true; }}
            >
              Remove
            </button>
          </div>
        {/each}
      </div>
    {/if}
  </div>

{:else if activeTab === "settings"}
  <div class="space-y-4">
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-medium text-cubelit-text">Server Configuration</h3>
      <Button size="sm" onclick={() => showApplyModal = true} disabled={settingsSaving || settingsLoading}>
        {settingsSaving ? "Applying…" : "Apply Changes"}
      </Button>
    </div>

    {#if settingsError}
      <p class="text-xs text-cubelit-error px-3 py-2 bg-cubelit-error/5 border border-cubelit-error/30 rounded-lg">{settingsError}</p>
    {/if}

    {#if settingsLoading}
      <p class="text-cubelit-muted text-sm py-8 text-center">Loading settings...</p>
    {:else if recipe}
      <div class="bg-cubelit-surface border border-cubelit-border rounded-xl p-5 space-y-4">
        {#each recipe.environment.filter(f => !READONLY_KEYS.includes(f.key)) as field}
          <div class="space-y-1">
            <label class="text-xs font-medium text-cubelit-muted" for={`fivem-${field.key}`}>{field.label}</label>
            {#if field.type === "boolean"}
              <div class="flex items-center gap-2">
                <input
                  id={`fivem-${field.key}`}
                  type="checkbox"
                  class="w-4 h-4 accent-cubelit-accent"
                  checked={editEnv[field.key]?.toLowerCase() === "true"}
                  onchange={(e) => editEnv[field.key] = (e.currentTarget as HTMLInputElement).checked ? "TRUE" : "FALSE"}
                />
                <span class="text-xs text-cubelit-muted font-mono">{field.key}</span>
              </div>
            {:else if field.options.length > 0}
              <div class="relative">
                <select
                  id={`fivem-${field.key}`}
                  class="w-full appearance-none bg-cubelit-bg border border-cubelit-border rounded-lg px-3 py-1.5 pr-8 text-sm text-cubelit-text focus:outline-none focus:border-cubelit-accent"
                  bind:value={editEnv[field.key]}
                >
                  {#each field.options as opt}
                    <option value={opt} style="background-color:#23272f;color:#f5f5f6;">{opt}</option>
                  {/each}
                </select>
                <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2.5">
                  <svg class="w-4 h-4 text-cubelit-muted" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" d="m19.5 8.25-7.5 7.5-7.5-7.5" />
                  </svg>
                </div>
              </div>
            {:else}
              <input
                id={`fivem-${field.key}`}
                type="text"
                class="w-full bg-cubelit-bg border border-cubelit-border rounded-lg px-3 py-1.5 text-sm text-cubelit-text focus:outline-none focus:border-cubelit-accent"
                bind:value={editEnv[field.key]}
              />
            {/if}
          </div>
        {/each}
      </div>
      <p class="text-xs text-cubelit-muted">Changes take effect after the server restarts. License key and database settings are managed automatically.</p>
    {/if}
  </div>

{:else if activeTab === "logs"}
  <LogViewer lines={logs} loading={logsLoading} onRefresh={loadLogs}>
    {#snippet helpContent()}
      <div class="bg-cubelit-surface border border-cubelit-border rounded-lg px-4 py-3 space-y-1.5">
        <p class="text-xs font-medium text-cubelit-text">What you're seeing</p>
        <ul class="text-xs text-cubelit-muted space-y-1 list-disc list-inside">
          <li><span class="text-cubelit-text">3 resources found</span> — correct, txAdmin starts with just yarn/webpack/monitor until you complete the setup wizard.</li>
          <li><span class="text-cubelit-text">Restart loop</span> — txAdmin keeps cycling FiveM until the wizard is completed. Open txAdmin on port 40120 to stop it.</li>
          <li><span class="text-cubelit-text font-mono">HTTP 429</span> — cfx.re rate-limits license auth after many restarts. Wait a few minutes if this appears.</li>
        </ul>
      </div>
    {/snippet}
  </LogViewer>
{/if}
