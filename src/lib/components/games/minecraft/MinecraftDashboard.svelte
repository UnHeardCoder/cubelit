<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import type { Cubelit } from "$lib/types/server";
  import type { FileEntry } from "$lib/types/files";
  import { listServerFiles, copyFileToServer, deleteServerFile, getServerLogs } from "$lib/api/files";
  import { sendMinecraftCommand, backupServer } from "$lib/api/minecraft";
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

  let activeTab = $state<"overview" | "mods" | "plugins" | "worlds" | "settings" | "logs">("overview");
  let mods = $state<FileEntry[]>([]);
  let plugins = $state<FileEntry[]>([]);
  let worlds = $state<FileEntry[]>([]);
  let logs = $state<string[]>([]);
  let modsLoading = $state(false);
  let pluginsLoading = $state(false);
  let worldsLoading = $state(false);
  let logsLoading = $state(false);
  let modsError = $state<string | null>(null);
  let pluginsError = $state<string | null>(null);
  let worldsError = $state<string | null>(null);

  let deleteModName = $state<string | null>(null);
  let showDeleteModModal = $state(false);
  let deletePluginName = $state<string | null>(null);
  let showDeletePluginModal = $state(false);
  let fileError = $state<string | null>(null);

  // ─── Drag-and-drop state ──────────────────────────────────────────────────
  let modsDropDepth = $state(0);
  let pluginsDropDepth = $state(0);
  let unlistenDragDrop: (() => void) | null = null;

  // ─── Settings state ───────────────────────────────────────────────────────
  let recipe = $state<Recipe | null>(null);
  let editEnv = $state<Record<string, string>>({});
  let settingsLoading = $state(false);
  let settingsSaving = $state(false);
  let settingsError = $state<string | null>(null);
  let showApplyModal = $state(false);

  // ─── Console state ────────────────────────────────────────────────────────
  let commandInput = $state("");
  let commandHistory = $state<string[]>([]);
  let historyIndex = $state(-1);
  let consoleOutput = $state<{ cmd: string; response: string; error?: boolean }[]>([]);
  let commandLoading = $state(false);
  let backupLoading = $state(false);
  let backupMessage = $state<string | null>(null);

  // Username persisted in localStorage
  let mcUsername = $state(
    typeof window !== "undefined" ? (localStorage.getItem("mc_username") ?? "") : ""
  );
  $effect(() => {
    if (typeof window !== "undefined") localStorage.setItem("mc_username", mcUsername);
  });

  const isRunning = $derived(server.status === "running");

  // ─── Public IP ────────────────────────────────────────────────────────────
  let publicIp = $state<string | null>(null);

  function getPort(): string {
    const ports = parsePorts(server.port_mappings);
    return String(Object.values(ports)[0] ?? "");
  }

  function getAddress(): string {
    const ports = parsePorts(server.port_mappings);
    const firstPort = Object.values(ports)[0];
    return firstPort ? `localhost:${firstPort}` : "";
  }

  async function loadMods() {
    modsLoading = true;
    modsError = null;
    try {
      mods = (await listServerFiles(server.id, "mods")).filter(f => f.name.endsWith(".jar"));
    } catch {
      modsError = "Failed to load mods.";
      mods = [];
    } finally { modsLoading = false; }
  }

  async function loadPlugins() {
    pluginsLoading = true;
    pluginsError = null;
    try {
      plugins = (await listServerFiles(server.id, "plugins")).filter(f => f.name.endsWith(".jar"));
    } catch {
      pluginsError = "Failed to load plugins.";
      plugins = [];
    } finally { pluginsLoading = false; }
  }

  async function loadWorlds() {
    worldsLoading = true;
    worldsError = null;
    try {
      worlds = (await listServerFiles(server.id)).filter(
        f => f.is_dir && !f.name.startsWith(".") && !["logs", "mods", "plugins", "config", "crash-reports"].includes(f.name)
      );
    } catch {
      worldsError = "Failed to load worlds.";
      worlds = [];
    } finally { worldsLoading = false; }
  }

  async function loadLogs() {
    logsLoading = true;
    try {
      logs = await getServerLogs(server.id, 200);
    } catch {
      logs = [];
    } finally { logsLoading = false; }
  }

  async function loadSettings() {
    settingsLoading = true;
    settingsError = null;
    try {
      recipe = await getRecipeDetail(server.recipe_id);
      editEnv = { ...parseEnv(server.environment) };
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
    try {
      await updateServerSettings(server.id, editEnv);
    } catch (e) {
      settingsError = String(e);
    } finally {
      settingsSaving = false;
    }
  }

  async function uploadMod() {
    fileError = null;
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        title: "Select Mod File",
        filters: [{ name: "Mod Files", extensions: ["jar"] }],
      });
      if (selected) {
        await copyFileToServer(server.id, selected as string, `mods/${(selected as string).split(/[/\\]/).pop()}`);
        await loadMods();
      }
    } catch (e) {
      fileError = String(e);
    }
  }

  async function uploadPlugin() {
    fileError = null;
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        title: "Select Plugin File",
        filters: [{ name: "Plugin Files", extensions: ["jar"] }],
      });
      if (selected) {
        await copyFileToServer(server.id, selected as string, `plugins/${(selected as string).split(/[/\\]/).pop()}`);
        await loadPlugins();
      }
    } catch (e) {
      fileError = String(e);
    }
  }

  async function confirmDeleteMod() {
    if (!deleteModName) return;
    fileError = null;
    try {
      await deleteServerFile(server.id, `mods/${deleteModName}`);
      await loadMods();
    } catch (e) {
      fileError = String(e);
    } finally {
      deleteModName = null;
      showDeleteModModal = false;
    }
  }

  async function confirmDeletePlugin() {
    if (!deletePluginName) return;
    fileError = null;
    try {
      await deleteServerFile(server.id, `plugins/${deletePluginName}`);
      await loadPlugins();
    } catch (e) {
      fileError = String(e);
    } finally {
      deletePluginName = null;
      showDeletePluginModal = false;
    }
  }

  async function openFolder(subpath?: string) {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const sep = server.volume_path.includes("\\") ? "\\" : "/";
      const path = subpath ? `${server.volume_path}${sep}${subpath}` : server.volume_path;
      await invoke("open_folder", { path });
    } catch (e) {
      console.error("Failed to open folder:", e);
    }
  }

  // ─── Console helpers ──────────────────────────────────────────────────────

  async function runCommand(cmd: string) {
    const trimmed = cmd.trim();
    if (!trimmed || commandLoading) return;

    commandLoading = true;
    commandHistory = [trimmed, ...commandHistory.filter(c => c !== trimmed)].slice(0, 50);
    historyIndex = -1;
    commandInput = "";

    try {
      const response = await sendMinecraftCommand(server.id, trimmed);
      consoleOutput = [{ cmd: trimmed, response: response || "(no output)" }, ...consoleOutput].slice(0, 100);
    } catch (e) {
      consoleOutput = [{ cmd: trimmed, response: String(e), error: true }, ...consoleOutput].slice(0, 100);
    } finally {
      commandLoading = false;
    }
  }

  function handleCommandKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      runCommand(commandInput);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      const next = Math.min(historyIndex + 1, commandHistory.length - 1);
      historyIndex = next;
      commandInput = commandHistory[next] ?? "";
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      const next = Math.max(historyIndex - 1, -1);
      historyIndex = next;
      commandInput = next === -1 ? "" : (commandHistory[next] ?? "");
    }
  }

  async function handleBackup() {
    backupLoading = true;
    backupMessage = null;
    try {
      const path = await backupServer(server.id);
      backupMessage = `Backup saved to: ${path}`;
    } catch (e) {
      backupMessage = `Backup failed: ${e}`;
    } finally {
      backupLoading = false;
    }
  }

  onMount(async () => {
    // Fetch public IP for the connection card
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      publicIp = await invoke<string>("get_public_ip");
    } catch {
      publicIp = null;
    }

    if (activeTab === "mods") loadMods();
    if (activeTab === "plugins") loadPlugins();
    if (activeTab === "worlds") loadWorlds();

    // Set up Tauri drag-and-drop listener for .jar files
    try {
      const { getCurrentWebview } = await import("@tauri-apps/api/webview");
      const unlisten = await getCurrentWebview().onDragDropEvent((event) => {
        if (event.payload.type === "drop") {
          const jarPaths = event.payload.paths.filter((p: string) => p.endsWith(".jar"));
          if (jarPaths.length === 0) return;

          if (activeTab === "mods") {
            Promise.all(
              jarPaths.map((p: string) =>
                copyFileToServer(server.id, p, `mods/${p.split(/[/\\]/).pop()}`)
              )
            ).then(() => loadMods()).catch((e) => { fileError = String(e); });
            modsDropDepth = 0;
          } else if (activeTab === "plugins") {
            Promise.all(
              jarPaths.map((p: string) =>
                copyFileToServer(server.id, p, `plugins/${p.split(/[/\\]/).pop()}`)
              )
            ).then(() => loadPlugins()).catch((e) => { fileError = String(e); });
            pluginsDropDepth = 0;
          }
        }
      });
      unlistenDragDrop = unlisten;
    } catch (e) {
      console.error("Failed to set up drag-drop listener:", e);
    }
  });

  onDestroy(() => {
    unlistenDragDrop?.();
  });

  $effect(() => {
    if (activeTab === "mods") loadMods();
    if (activeTab === "plugins") loadPlugins();
    if (activeTab === "worlds") loadWorlds();
    if (activeTab === "settings") loadSettings();
  });

  $effect(() => {
    if (activeTab === "logs") {
      loadLogs();
      // Poll while starting OR running so users can watch the boot sequence live
      if (server.status === "running" || server.status === "starting") {
        const interval = setInterval(loadLogs, 3000);
        return () => clearInterval(interval);
      }
    }
  });

  const tabs = [
    { id: "overview" as const, label: "Overview" },
    { id: "mods" as const, label: "Mods" },
    { id: "plugins" as const, label: "Plugins" },
    { id: "worlds" as const, label: "Worlds" },
    { id: "settings" as const, label: "Settings" },
    { id: "logs" as const, label: "Console" },
  ];
</script>

<!-- Apply Settings Modal -->
<Modal bind:open={showApplyModal} onclose={() => showApplyModal = false} title="Apply Settings">
  <p class="text-sm text-cubelit-muted mb-6">
    Applying settings will <span class="text-cubelit-warning font-medium">restart your server</span>. Players will be disconnected. Continue?
  </p>
  <div class="flex gap-3 justify-end">
    <Button variant="ghost" onclick={() => showApplyModal = false}>Cancel</Button>
    <Button variant="primary" onclick={applySettings}>Apply & Restart</Button>
  </div>
</Modal>

<!-- Delete Mod Modal -->
<Modal bind:open={showDeleteModModal} onclose={() => { deleteModName = null; showDeleteModModal = false; }} title="Delete Mod">
  <p class="text-sm text-cubelit-muted mb-6">Delete <span class="text-cubelit-text font-medium">{deleteModName}</span>? This cannot be undone.</p>
  <div class="flex gap-3 justify-end">
    <Button variant="ghost" onclick={() => { deleteModName = null; showDeleteModModal = false; }}>Cancel</Button>
    <Button variant="danger" onclick={confirmDeleteMod}>Delete</Button>
  </div>
</Modal>

<!-- Delete Plugin Modal -->
<Modal bind:open={showDeletePluginModal} onclose={() => { deletePluginName = null; showDeletePluginModal = false; }} title="Delete Plugin">
  <p class="text-sm text-cubelit-muted mb-6">Delete <span class="text-cubelit-text font-medium">{deletePluginName}</span>? This cannot be undone.</p>
  <div class="flex gap-3 justify-end">
    <Button variant="ghost" onclick={() => { deletePluginName = null; showDeletePluginModal = false; }}>Cancel</Button>
    <Button variant="danger" onclick={confirmDeletePlugin}>Delete</Button>
  </div>
</Modal>

<!-- Tabs -->
<div class="flex gap-1 mb-6 border-b border-cubelit-border">
  {#each tabs as tab}
    <button
      class="px-4 py-2.5 text-sm font-medium transition-colors relative {activeTab === tab.id
        ? 'text-cubelit-accent'
        : 'text-cubelit-muted hover:text-cubelit-text'}"
      onclick={() => activeTab = tab.id}
    >
      {tab.label}
      {#if activeTab === tab.id}
        <div class="absolute bottom-0 left-0 right-0 h-0.5 bg-cubelit-accent rounded-t" />
      {/if}
    </button>
  {/each}
</div>

{#if fileError}
  <div class="flex items-start justify-between gap-3 px-4 py-3 mb-4 bg-cubelit-error/5 border border-cubelit-error/30 rounded-xl">
    <p class="text-xs text-cubelit-error">{fileError}</p>
    <button class="shrink-0 text-cubelit-error/60 hover:text-cubelit-error transition-colors" onclick={() => fileError = null}>
      <svg class="w-3.5 h-3.5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
      </svg>
    </button>
  </div>
{/if}

{#if activeTab === "overview"}
  {#if server.status === "starting"}
    <div class="mb-4 flex items-center gap-3 bg-cubelit-warning/10 border border-cubelit-warning/30 rounded-xl px-4 py-3">
      <span class="relative flex h-2.5 w-2.5 shrink-0">
        <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-cubelit-warning opacity-75"></span>
        <span class="relative inline-flex rounded-full h-2.5 w-2.5 bg-cubelit-warning"></span>
      </span>
      <p class="text-sm text-cubelit-warning font-medium">Server is starting up —
        <button class="underline underline-offset-2 hover:opacity-80 transition-opacity" onclick={() => activeTab = "logs"}>
          open Console
        </button>
        to watch the boot sequence.
      </p>
    </div>
  {/if}

  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
    <div class="bg-cubelit-surface border border-cubelit-border rounded-xl p-5">
      <h3 class="text-sm font-medium text-cubelit-muted mb-3">Status</h3>
      <p class="text-lg font-semibold capitalize {STATUS_COLORS[server.status] ?? 'text-cubelit-muted'}">
        {server.status}
      </p>
    </div>

    <div class="bg-cubelit-surface border border-cubelit-border rounded-xl p-5">
      <h3 class="text-sm font-medium text-cubelit-muted mb-3">Connection</h3>
      <div class="space-y-2">
        <div class="flex items-center justify-between gap-2">
          <span class="text-xs text-cubelit-muted shrink-0">Local</span>
          <span class="text-cubelit-text font-mono text-sm">{getAddress()}</span>
        </div>
        <div class="flex items-center justify-between gap-2">
          <span class="text-xs text-cubelit-muted shrink-0">Public</span>
          {#if publicIp}
            <span class="text-cubelit-text font-mono text-sm">{publicIp}:{getPort()}</span>
          {:else}
            <span class="text-cubelit-muted font-mono text-sm text-xs">fetching…</span>
          {/if}
        </div>
        <p class="text-xs text-cubelit-muted/60 pt-1">Friends use the Public address to connect. Make sure port {getPort()} is forwarded on your router.</p>
      </div>
    </div>

    <StatsCards serverId={server.id} serverStatus={server.status} />

    <div class="bg-cubelit-surface border border-cubelit-border rounded-xl p-5">
      <h3 class="text-sm font-medium text-cubelit-muted mb-3">Image</h3>
      <p class="text-cubelit-text text-sm font-mono">{server.docker_image}</p>
    </div>

    <div class="bg-cubelit-surface border border-cubelit-border rounded-xl p-5">
      <div class="flex items-start justify-between gap-3">
        <div class="min-w-0">
          <h3 class="text-sm font-medium text-cubelit-muted mb-3">Server Files</h3>
          <p class="text-cubelit-text text-sm font-mono truncate">{server.volume_path}</p>
        </div>
        <button
          class="shrink-0 mt-0.5 text-xs text-cubelit-muted hover:text-cubelit-accent transition-colors flex items-center gap-1"
          onclick={() => openFolder()}
        >
          <svg class="w-3.5 h-3.5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 9.776c.112-.017.227-.026.344-.026h15.812c.117 0 .232.009.344.026m-16.5 0a2.25 2.25 0 0 0-1.883 2.542l.857 6a2.25 2.25 0 0 0 2.227 1.932H19.05a2.25 2.25 0 0 0 2.227-1.932l.857-6a2.25 2.25 0 0 0-1.883-2.542m-16.5 0V6A2.25 2.25 0 0 1 6 3.75h3.879a1.5 1.5 0 0 1 1.06.44l2.122 2.12a1.5 1.5 0 0 0 1.06.44H18A2.25 2.25 0 0 1 20.25 9v.776" />
          </svg>
          Open
        </button>
      </div>
    </div>

    <div class="bg-cubelit-surface border border-cubelit-border rounded-xl p-5">
      <h3 class="text-sm font-medium text-cubelit-muted mb-3">Container ID</h3>
      <p class="text-cubelit-text text-sm font-mono truncate">{server.container_id ?? "N/A"}</p>
    </div>

    <div class="bg-cubelit-surface border border-cubelit-border rounded-xl p-5">
      <h3 class="text-sm font-medium text-cubelit-muted mb-3">Ports</h3>
      {#each Object.entries(parsePorts(server.port_mappings)) as [container, host]}
        <p class="text-cubelit-text text-sm">{container} &rarr; {host}</p>
      {/each}
    </div>
  </div>

{:else if activeTab === "mods"}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="space-y-4 transition-all {modsDropDepth > 0 ? 'ring-2 ring-cubelit-accent ring-inset rounded-xl' : ''}"
    ondragenter={() => modsDropDepth++}
    ondragleave={() => modsDropDepth = Math.max(0, modsDropDepth - 1)}
    ondragover={(e) => e.preventDefault()}
    ondrop={() => modsDropDepth = 0}
  >
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-medium text-cubelit-text">Installed Mods</h3>
      <div class="flex items-center gap-2">
        <button
          class="text-xs text-cubelit-muted hover:text-cubelit-accent transition-colors flex items-center gap-1"
          onclick={() => openFolder("mods")}
        >
          <svg class="w-3.5 h-3.5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 9.776c.112-.017.227-.026.344-.026h15.812c.117 0 .232.009.344.026m-16.5 0a2.25 2.25 0 0 0-1.883 2.542l.857 6a2.25 2.25 0 0 0 2.227 1.932H19.05a2.25 2.25 0 0 0 2.227-1.932l.857-6a2.25 2.25 0 0 0-1.883-2.542m-16.5 0V6A2.25 2.25 0 0 1 6 3.75h3.879a1.5 1.5 0 0 1 1.06.44l2.122 2.12a1.5 1.5 0 0 0 1.06.44H18A2.25 2.25 0 0 1 20.25 9v.776" />
          </svg>
          Open Folder
        </button>
        <Button size="sm" onclick={uploadMod}>Upload Mod</Button>
      </div>
    </div>

    {#if modsDropDepth > 0}
      <div class="flex items-center justify-center py-6 border-2 border-dashed border-cubelit-accent rounded-xl bg-cubelit-accent/5">
        <p class="text-sm text-cubelit-accent font-medium">Drop .jar files to install</p>
      </div>
    {:else if modsError}
      <p class="text-xs text-cubelit-error py-4 text-center">{modsError}</p>
    {:else if modsLoading}
      <p class="text-cubelit-muted text-sm py-8 text-center">Loading mods...</p>
    {:else if mods.length === 0}
      <div class="text-center py-12 bg-cubelit-surface border border-dashed border-cubelit-border rounded-xl">
        <p class="text-cubelit-muted text-sm mb-2">No mods installed</p>
        <p class="text-cubelit-muted/70 text-xs">Upload .jar files or drag & drop them here</p>
      </div>
    {:else}
      <div class="space-y-2">
        {#each mods as mod}
          <div class="flex items-center justify-between bg-cubelit-surface border border-cubelit-border rounded-xl px-4 py-3">
            <div>
              <p class="text-sm text-cubelit-text">{mod.name}</p>
              <p class="text-xs text-cubelit-muted">{(mod.size / 1024 / 1024).toFixed(1)} MB</p>
            </div>
            <button
              class="text-xs text-cubelit-error hover:text-cubelit-error/80 transition-colors"
              onclick={() => { deleteModName = mod.name; showDeleteModModal = true; }}
            >
              Remove
            </button>
          </div>
        {/each}
      </div>
    {/if}
  </div>

{:else if activeTab === "plugins"}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="space-y-4 transition-all {pluginsDropDepth > 0 ? 'ring-2 ring-cubelit-accent ring-inset rounded-xl' : ''}"
    ondragenter={() => pluginsDropDepth++}
    ondragleave={() => pluginsDropDepth = Math.max(0, pluginsDropDepth - 1)}
    ondragover={(e) => e.preventDefault()}
    ondrop={() => pluginsDropDepth = 0}
  >
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-medium text-cubelit-text">Installed Plugins</h3>
      <div class="flex items-center gap-2">
        <button
          class="text-xs text-cubelit-muted hover:text-cubelit-accent transition-colors flex items-center gap-1"
          onclick={() => openFolder("plugins")}
        >
          <svg class="w-3.5 h-3.5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 9.776c.112-.017.227-.026.344-.026h15.812c.117 0 .232.009.344.026m-16.5 0a2.25 2.25 0 0 0-1.883 2.542l.857 6a2.25 2.25 0 0 0 2.227 1.932H19.05a2.25 2.25 0 0 0 2.227-1.932l.857-6a2.25 2.25 0 0 0-1.883-2.542m-16.5 0V6A2.25 2.25 0 0 1 6 3.75h3.879a1.5 1.5 0 0 1 1.06.44l2.122 2.12a1.5 1.5 0 0 0 1.06.44H18A2.25 2.25 0 0 1 20.25 9v.776" />
          </svg>
          Open Folder
        </button>
        <Button size="sm" onclick={uploadPlugin}>Upload Plugin</Button>
      </div>
    </div>

    {#if pluginsDropDepth > 0}
      <div class="flex items-center justify-center py-6 border-2 border-dashed border-cubelit-accent rounded-xl bg-cubelit-accent/5">
        <p class="text-sm text-cubelit-accent font-medium">Drop .jar files to install</p>
      </div>
    {:else if pluginsError}
      <p class="text-xs text-cubelit-error py-4 text-center">{pluginsError}</p>
    {:else if pluginsLoading}
      <p class="text-cubelit-muted text-sm py-8 text-center">Loading plugins...</p>
    {:else if plugins.length === 0}
      <div class="text-center py-12 bg-cubelit-surface border border-dashed border-cubelit-border rounded-xl">
        <p class="text-cubelit-muted text-sm mb-2">No plugins installed</p>
        <p class="text-cubelit-muted/70 text-xs">Upload .jar files or drag & drop them here</p>
      </div>
    {:else}
      <div class="space-y-2">
        {#each plugins as plugin}
          <div class="flex items-center justify-between bg-cubelit-surface border border-cubelit-border rounded-xl px-4 py-3">
            <div>
              <p class="text-sm text-cubelit-text">{plugin.name}</p>
              <p class="text-xs text-cubelit-muted">{(plugin.size / 1024 / 1024).toFixed(1)} MB</p>
            </div>
            <button
              class="text-xs text-cubelit-error hover:text-cubelit-error/80 transition-colors"
              onclick={() => { deletePluginName = plugin.name; showDeletePluginModal = true; }}
            >
              Remove
            </button>
          </div>
        {/each}
      </div>
    {/if}
  </div>

{:else if activeTab === "worlds"}
  <div class="space-y-4">
    <h3 class="text-sm font-medium text-cubelit-text">World Folders</h3>

    {#if worldsError}
      <p class="text-xs text-cubelit-error py-4 text-center">{worldsError}</p>
    {:else if worldsLoading}
      <p class="text-cubelit-muted text-sm py-8 text-center">Loading worlds...</p>
    {:else if worlds.length === 0}
      <div class="text-center py-12 bg-cubelit-surface border border-cubelit-border rounded-xl">
        <p class="text-cubelit-muted text-sm">No worlds found</p>
        <p class="text-cubelit-muted/70 text-xs mt-1">World folders will appear here after the server has run</p>
      </div>
    {:else}
      <div class="space-y-2">
        {#each worlds as world}
          <div class="flex items-center gap-3 bg-cubelit-surface border border-cubelit-border rounded-xl px-4 py-3">
            <svg class="w-5 h-5 text-cubelit-accent shrink-0" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" d="M12 21a9.004 9.004 0 0 0 8.716-6.747M12 21a9.004 9.004 0 0 1-8.716-6.747M12 21c2.485 0 4.5-4.03 4.5-9S14.485 3 12 3m0 18c-2.485 0-4.5-4.03-4.5-9S9.515 3 12 3m0 0a8.997 8.997 0 0 1 7.843 4.582M12 3a8.997 8.997 0 0 0-7.843 4.582m15.686 0A11.953 11.953 0 0 1 12 10.5c-2.998 0-5.74-1.1-7.843-2.918m15.686 0A8.959 8.959 0 0 1 21 12c0 .778-.099 1.533-.284 2.253m0 0A17.919 17.919 0 0 1 12 16.5c-3.162 0-6.133-.815-8.716-2.247m0 0A9.015 9.015 0 0 1 3 12c0-1.605.42-3.113 1.157-4.418" />
            </svg>
            <p class="text-sm text-cubelit-text">{world.name}</p>
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
        {#each recipe.environment as field}
          <div class="space-y-1">
            <label class="text-xs font-medium text-cubelit-muted">{field.label}</label>
            {#if field.type === "boolean"}
              <div class="flex items-center gap-2">
                <input
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
                type="text"
                class="w-full bg-cubelit-bg border border-cubelit-border rounded-lg px-3 py-1.5 text-sm text-cubelit-text focus:outline-none focus:border-cubelit-accent"
                bind:value={editEnv[field.key]}
              />
            {/if}
          </div>
        {/each}
      </div>
      <p class="text-xs text-cubelit-muted">Changes take effect after the server restarts.</p>
    {/if}
  </div>

{:else if activeTab === "logs"}
  <div class="space-y-5">

    <!-- Live log output -->
    <LogViewer lines={logs} loading={logsLoading} onRefresh={loadLogs} />

    <!-- ── Console ──────────────────────────────────────────────────────── -->
    <div class="border-t border-cubelit-border pt-5 space-y-4">
      <div class="flex items-center justify-between">
        <h3 class="text-sm font-medium text-cubelit-text">Server Console</h3>
        {#if !isRunning}
          <span class="text-xs text-cubelit-muted italic">Start the server to use the console</span>
        {/if}
      </div>

      <!-- Saved username -->
      <div class="flex items-center gap-3">
        <span class="text-xs text-cubelit-muted w-28 shrink-0">Your username</span>
        <input
          type="text"
          bind:value={mcUsername}
          placeholder="Steve"
          class="flex-1 bg-cubelit-bg border border-cubelit-border rounded-lg px-3 py-1.5 text-sm text-cubelit-text placeholder-cubelit-muted/40 focus:outline-none focus:border-cubelit-accent"
        />
      </div>

      <!-- Quick-action buttons -->
      <div class="flex flex-wrap gap-2">
        <button
          class="px-3 py-1.5 rounded-lg text-xs font-medium transition-colors bg-cubelit-surface border border-cubelit-border text-cubelit-text hover:border-cubelit-accent hover:text-cubelit-accent disabled:opacity-40 disabled:cursor-not-allowed"
          disabled={!isRunning || !mcUsername.trim() || commandLoading}
          onclick={() => runCommand(`op ${mcUsername.trim()}`)}
        >OP Self</button>

        <button
          class="px-3 py-1.5 rounded-lg text-xs font-medium transition-colors bg-cubelit-surface border border-cubelit-border text-cubelit-text hover:border-cubelit-accent hover:text-cubelit-accent disabled:opacity-40 disabled:cursor-not-allowed"
          disabled={!isRunning || !mcUsername.trim() || commandLoading}
          onclick={() => runCommand(`deop ${mcUsername.trim()}`)}
        >Deop Self</button>

        <button
          class="px-3 py-1.5 rounded-lg text-xs font-medium transition-colors bg-cubelit-surface border border-cubelit-border text-cubelit-text hover:border-cubelit-accent hover:text-cubelit-accent disabled:opacity-40 disabled:cursor-not-allowed"
          disabled={!isRunning || !mcUsername.trim() || commandLoading}
          onclick={() => runCommand(`whitelist add ${mcUsername.trim()}`)}
        >Whitelist Self</button>

        <button
          class="px-3 py-1.5 rounded-lg text-xs font-medium transition-colors bg-cubelit-surface border border-cubelit-border text-cubelit-text hover:border-cubelit-accent hover:text-cubelit-accent disabled:opacity-40 disabled:cursor-not-allowed"
          disabled={!isRunning || commandLoading}
          onclick={() => runCommand("list")}
        >List Players</button>

        <button
          class="px-3 py-1.5 rounded-lg text-xs font-medium transition-colors bg-cubelit-surface border border-cubelit-border text-cubelit-text hover:border-cubelit-accent hover:text-cubelit-accent disabled:opacity-40 disabled:cursor-not-allowed"
          disabled={!isRunning || commandLoading}
          onclick={() => runCommand("save-all")}
        >Save World</button>

        <button
          class="px-3 py-1.5 rounded-lg text-xs font-medium transition-colors bg-cubelit-surface border border-cubelit-border text-cubelit-text hover:border-cubelit-accent hover:text-cubelit-accent disabled:opacity-40 disabled:cursor-not-allowed"
          disabled={!isRunning || commandLoading}
          onclick={() => runCommand("weather clear")}
        >Clear Weather</button>

        <button
          class="px-3 py-1.5 rounded-lg text-xs font-medium transition-colors bg-cubelit-surface border border-cubelit-border text-cubelit-text hover:border-cubelit-warning hover:text-cubelit-warning disabled:opacity-40 disabled:cursor-not-allowed"
          disabled={backupLoading}
          onclick={handleBackup}
        >{backupLoading ? "Backing up…" : "Backup Server"}</button>
      </div>

      <!-- Backup result message -->
      {#if backupMessage}
        <p class="text-xs px-3 py-2 rounded-lg border {backupMessage.startsWith('Backup failed') ? 'border-cubelit-error/40 text-cubelit-error bg-cubelit-error/5' : 'border-cubelit-success/40 text-cubelit-success bg-cubelit-success/5'}">
          {backupMessage}
        </p>
      {/if}

      <!-- Custom command input -->
      <div class="flex gap-2">
        <input
          type="text"
          bind:value={commandInput}
          onkeydown={handleCommandKeydown}
          placeholder={isRunning ? "Enter command  (↑ ↓ for history)" : "Server must be running"}
          disabled={!isRunning || commandLoading}
          class="flex-1 bg-[#0d1117] border border-cubelit-border rounded-lg px-3 py-2 text-sm font-mono text-cubelit-text placeholder-cubelit-muted/40 focus:outline-none focus:border-cubelit-accent disabled:opacity-50"
        />
        <Button
          size="sm"
          disabled={!isRunning || !commandInput.trim() || commandLoading}
          onclick={() => runCommand(commandInput)}
        >{commandLoading ? "…" : "Send"}</Button>
      </div>

      <!-- Response history -->
      {#if consoleOutput.length > 0}
        <div class="bg-[#0d1117] border border-cubelit-border rounded-xl p-4 space-y-3 max-h-64 overflow-y-auto font-mono text-xs">
          {#each consoleOutput as entry}
            <div>
              <div class="text-cubelit-accent">❯ {entry.cmd}</div>
              <div class="mt-0.5 {entry.error ? 'text-cubelit-error' : 'text-gray-300'} whitespace-pre-wrap pl-3">
                {entry.response}
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

  </div>
{/if}
