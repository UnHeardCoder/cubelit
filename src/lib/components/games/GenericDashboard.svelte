<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { invoke } from '@tauri-apps/api/core';
  import { getServerStats, updateServerSettings } from '$lib/api/docker';
  import { getPublicIp } from '$lib/api/system';
  import { getServerLogs, listServerFiles, copyFileToServer, deleteServerFile, readServerFile, writeServerFile } from '$lib/api/files';
  import { backupServer } from '$lib/api/minecraft';
  import { getServersStore } from '$lib/stores/servers.svelte';
  import { goto } from '$app/navigation';
  import { GAME_HUE } from '$lib/games/art';
  import GaugeCard from '$lib/components/GaugeCard.svelte';
  import Sparkline from '$lib/components/Sparkline.svelte';
  import ConnRow from '$lib/components/ConnRow.svelte';
  import StatusPill from '$lib/components/StatusPill.svelte';
  import Modal from '$lib/components/Modal.svelte';
  import Button from '$lib/components/Button.svelte';
  import type { Cubelit } from '$lib/types/server';

  interface Props { server: Cubelit; }
  let { server }: Props = $props();

  let tab = $state<'overview' | 'console' | 'settings' | 'files'>('overview');

  // --- Overview ---
  let cpuPct = $state(0);
  let memUsed = $state(0);
  let memTotal = $state(1);
  let statsInterval: ReturnType<typeof setInterval> | null = null;
  let publicIp = $state<string | null>(null);

  async function loadStats() {
    try {
      const s = await getServerStats(server.id);
      cpuPct = s.cpu_percent;
      memUsed = s.memory_usage_mb / 1024;
      memTotal = s.memory_limit_mb / 1024;
    } catch { /* ignore */ }
  }

  function getAddress(): string {
    try {
      const ports: Record<string, number> = JSON.parse(server.port_mappings);
      const first = Object.values(ports)[0];
      if (first) return `localhost:${first}`;
    } catch { /* ignore */ }
    return '—';
  }

  function getPublicAddress(): string {
    try {
      const ports: Record<string, number> = JSON.parse(server.port_mappings);
      const first = Object.values(ports)[0];
      if (first) return `<public-ip>:${first}`;
    } catch { /* ignore */ }
    return '—';
  }

  // --- Console / Logs ---
  let logLines = $state<string[]>([]);
  let logLoading = $state(false);
  let logFollow = $state(true);
  let logContainer = $state<HTMLDivElement | null>(null);

  async function loadLogs() {
    logLoading = logLines.length === 0;
    try {
      const lines = await getServerLogs(server.id, 120);
      logLines = lines.map((line) => line.length > 4000 ? `${line.slice(0, 4000)}... [truncated]` : line);
      if (logFollow && logContainer) {
        requestAnimationFrame(() => {
          if (logContainer) logContainer.scrollTop = logContainer.scrollHeight;
        });
      }
    } catch { /* ignore */ }
    logLoading = false;
  }

  function handleLogScroll() {
    if (!logContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = logContainer;
    logFollow = scrollHeight - scrollTop - clientHeight < 40;
  }

  // --- Settings ---
  let envEntries = $state<[string, string][]>([]);
  let envEdited = $state<Record<string, string>>({});
  let showApplyModal = $state(false);
  let applyLoading = $state(false);
  let applyError = $state<string | null>(null);
  let showDeleteModal = $state(false);
  let deleteWithData = $state(false);
  let deleteLoading = $state(false);
  let deleteError = $state<string | null>(null);
  let deleteFileName = $state<string | null>(null);
  let showDeleteFileModal = $state(false);

  const ASA_GAME_USER_SETTINGS = 'ShooterGame/Saved/Config/WindowsServer/GameUserSettings.ini';
  const ASA_GAME_INI = 'ShooterGame/Saved/Config/WindowsServer/Game.ini';

  type AsaConfig = {
    sessionName: string;
    maxPlayers: string;
    serverPve: boolean;
    xpMultiplier: string;
    tamingSpeedMultiplier: string;
    harvestAmountMultiplier: string;
    playerWaterDrainMultiplier: string;
    playerFoodDrainMultiplier: string;
    oxygenPerLevel: string;
    allowSpeedLeveling: boolean;
    allowFlyerSpeedLeveling: boolean;
  };

  const defaultAsaConfig = (): AsaConfig => ({
    sessionName: 'Cubelit-ASA-Server',
    maxPlayers: '20',
    serverPve: false,
    xpMultiplier: '1',
    tamingSpeedMultiplier: '1',
    harvestAmountMultiplier: '1',
    playerWaterDrainMultiplier: '1',
    playerFoodDrainMultiplier: '1',
    oxygenPerLevel: '1',
    allowSpeedLeveling: false,
    allowFlyerSpeedLeveling: false,
  });

  let asaConfig = $state<AsaConfig>(defaultAsaConfig());
  let asaGameUserSettings = $state('');
  let asaGameIni = $state('');
  let asaConfigLoading = $state(false);
  let asaConfigSaving = $state(false);
  let asaConfigError = $state<string | null>(null);
  let asaConfigSaved = $state(false);

  function parseIniValue(content: string, section: string, key: string): string | null {
    let inSection = false;
    for (const rawLine of content.split(/\r?\n/)) {
      const line = rawLine.trim();
      if (!line || line.startsWith(';') || line.startsWith('#')) continue;
      if (line.startsWith('[') && line.endsWith(']')) {
        inSection = line === section;
        continue;
      }
      if (!inSection) continue;
      const index = line.indexOf('=');
      if (index === -1) continue;
      if (line.slice(0, index).trim() === key) return line.slice(index + 1).trim();
    }
    return null;
  }

  function boolIniValue(content: string, section: string, key: string, fallback: boolean): boolean {
    const value = parseIniValue(content, section, key);
    if (value === null) return fallback;
    return value.toLowerCase() === 'true';
  }

  function upsertIniValue(content: string, section: string, key: string, value: string): string {
    const lines = content.replace(/\r\n/g, '\n').split('\n');
    let sectionStart = -1;
    let sectionEnd = lines.length;

    for (let i = 0; i < lines.length; i += 1) {
      if (lines[i].trim() === section) {
        sectionStart = i;
        for (let j = i + 1; j < lines.length; j += 1) {
          const line = lines[j].trim();
          if (line.startsWith('[') && line.endsWith(']')) {
            sectionEnd = j;
            break;
          }
        }
        break;
      }
    }

    if (sectionStart === -1) {
      const prefix = content.trim().length > 0 ? `${content.replace(/\r\n/g, '\n').replace(/\n*$/, '\n\n')}` : '';
      return `${prefix}${section}\n${key}=${value}\n`;
    }

    for (let i = sectionStart + 1; i < sectionEnd; i += 1) {
      const line = lines[i];
      const index = line.indexOf('=');
      if (index !== -1 && line.slice(0, index).trim() === key) {
        lines[i] = `${key}=${value}`;
        return lines.join('\n');
      }
    }

    lines.splice(sectionEnd, 0, `${key}=${value}`);
    return lines.join('\n');
  }

  async function loadAsaConfig() {
    if (server.recipe_id !== 'ark-survival-ascended') return;

    asaConfigLoading = true;
    asaConfigError = null;
    asaConfigSaved = false;
    try {
      const [gus, game] = await Promise.all([
        readServerFile(server.id, ASA_GAME_USER_SETTINGS),
        readServerFile(server.id, ASA_GAME_INI),
      ]);

      asaGameUserSettings = gus;
      asaGameIni = game;
      asaConfig = {
        sessionName: parseIniValue(gus, '[SessionSettings]', 'SessionName') ?? 'Cubelit-ASA-Server',
        maxPlayers: parseIniValue(gus, '[/Script/Engine.GameSession]', 'MaxPlayers') ?? '20',
        serverPve: boolIniValue(gus, '[ServerSettings]', 'ServerPVE', false),
        xpMultiplier: parseIniValue(gus, '[ServerSettings]', 'XPMultiplier') ?? '1',
        tamingSpeedMultiplier: parseIniValue(gus, '[ServerSettings]', 'TamingSpeedMultiplier') ?? '1',
        harvestAmountMultiplier: parseIniValue(gus, '[ServerSettings]', 'HarvestAmountMultiplier') ?? '1',
        playerWaterDrainMultiplier: parseIniValue(gus, '[ServerSettings]', 'PlayerCharacterWaterDrainMultiplier') ?? '1',
        playerFoodDrainMultiplier: parseIniValue(gus, '[ServerSettings]', 'PlayerCharacterFoodDrainMultiplier') ?? '1',
        oxygenPerLevel: parseIniValue(game, '[/Script/ShooterGame.ShooterGameMode]', 'PerLevelStatsMultiplier_Player[3]') ?? '1',
        allowSpeedLeveling: boolIniValue(game, '[/Script/ShooterGame.ShooterGameMode]', 'bAllowSpeedLeveling', false),
        allowFlyerSpeedLeveling: boolIniValue(game, '[/Script/ShooterGame.ShooterGameMode]', 'bAllowFlyerSpeedLeveling', false),
      };
    } catch (e) {
      asaConfigError = `Failed to load ARK config: ${String(e)}`;
    } finally {
      asaConfigLoading = false;
    }
  }

  async function saveAsaConfig() {
    asaConfigSaving = true;
    asaConfigError = null;
    asaConfigSaved = false;
    try {
      let gus = asaGameUserSettings || ';METADATA=(Diff=true, UseCommands=true)\n';
      let game = asaGameIni || ';METADATA=(Diff=true, UseCommands=true)\n';

      gus = upsertIniValue(gus, '[SessionSettings]', 'SessionName', asaConfig.sessionName.trim() || 'Cubelit-ASA-Server');
      gus = upsertIniValue(gus, '[/Script/Engine.GameSession]', 'MaxPlayers', asaConfig.maxPlayers || '20');
      gus = upsertIniValue(gus, '[ServerSettings]', 'ServerPVE', String(asaConfig.serverPve));
      gus = upsertIniValue(gus, '[ServerSettings]', 'XPMultiplier', asaConfig.xpMultiplier || '1');
      gus = upsertIniValue(gus, '[ServerSettings]', 'TamingSpeedMultiplier', asaConfig.tamingSpeedMultiplier || '1');
      gus = upsertIniValue(gus, '[ServerSettings]', 'HarvestAmountMultiplier', asaConfig.harvestAmountMultiplier || '1');
      gus = upsertIniValue(gus, '[ServerSettings]', 'PlayerCharacterWaterDrainMultiplier', asaConfig.playerWaterDrainMultiplier || '1');
      gus = upsertIniValue(gus, '[ServerSettings]', 'PlayerCharacterFoodDrainMultiplier', asaConfig.playerFoodDrainMultiplier || '1');
      gus = upsertIniValue(gus, '[ServerSettings]', 'bAllowSpeedLeveling', String(asaConfig.allowSpeedLeveling));
      gus = upsertIniValue(gus, '[ServerSettings]', 'bAllowFlyerSpeedLeveling', String(asaConfig.allowFlyerSpeedLeveling));

      game = upsertIniValue(game, '[/Script/ShooterGame.ShooterGameMode]', 'PerLevelStatsMultiplier_Player[3]', asaConfig.oxygenPerLevel || '1');
      game = upsertIniValue(game, '[/Script/ShooterGame.ShooterGameMode]', 'bAllowSpeedLeveling', String(asaConfig.allowSpeedLeveling));
      game = upsertIniValue(game, '[/Script/ShooterGame.ShooterGameMode]', 'bAllowFlyerSpeedLeveling', String(asaConfig.allowFlyerSpeedLeveling));

      await Promise.all([
        writeServerFile(server.id, ASA_GAME_USER_SETTINGS, gus),
        writeServerFile(server.id, ASA_GAME_INI, game),
      ]);

      asaGameUserSettings = gus;
      asaGameIni = game;
      asaConfigSaved = true;
    } catch (e) {
      asaConfigError = `Failed to save ARK config: ${String(e)}`;
    } finally {
      asaConfigSaving = false;
    }
  }

  const serversStore = getServersStore();

  function loadEnv() {
    try {
      const env: Record<string, string> = JSON.parse(server.environment);
      envEntries = Object.entries(env);
      envEdited = { ...env };
    } catch {
      envEntries = [];
      envEdited = {};
    }
  }

  async function applySettings() {
    applyLoading = true;
    applyError = null;
    try {
      await updateServerSettings(server.id, envEdited);
      showApplyModal = false;
    } catch (e) {
      applyError = String(e);
    } finally {
      applyLoading = false;
    }
  }

  async function handleDelete() {
    deleteLoading = true;
    deleteError = null;
    try {
      await serversStore.remove(server.id, deleteWithData);
      await goto('/');
    } catch (e) {
      deleteError = String(e);
      deleteLoading = false;
    }
  }

  // --- Files ---
  let files = $state<{ name: string; size: number }[]>([]);
  let filesLoading = $state(false);

  async function loadFiles() {
    filesLoading = true;
    try {
      files = await listServerFiles(server.id);
    } catch { files = []; }
    finally { filesLoading = false; }
  }

  async function handleUpload() {
    const selected = await open({ multiple: false, title: 'Select file to upload' });
    if (!selected) return;
    const path = typeof selected === 'string' ? selected : selected;
    const filename = (path as string).split('/').pop() ?? 'file';
    await copyFileToServer(server.id, path as string, filename);
    await loadFiles();
  }

  async function confirmDeleteFile() {
    if (!deleteFileName) return;
    await deleteServerFile(server.id, deleteFileName);
    deleteFileName = null;
    showDeleteFileModal = false;
    await loadFiles();
  }

  onMount(async () => {
    if (server.status === 'running') {
      await loadStats();
      statsInterval = setInterval(loadStats, 5000);
    }
    getPublicIp().then(ip => { publicIp = ip; }).catch(() => { publicIp = null; });
    loadEnv();
  });

  onDestroy(() => {
    if (statsInterval) clearInterval(statsInterval);
  });

  $effect(() => {
    if (tab !== 'console') return;

    loadLogs();
    const interval = setInterval(() => {
      if (server.status === 'running') loadLogs();
    }, 3000);

    return () => clearInterval(interval);
  });

  $effect(() => {
    if (tab === 'settings' && server.recipe_id === 'ark-survival-ascended') {
      loadAsaConfig();
    }
  });

  const hue = $derived(GAME_HUE[server.recipe_id] ?? 30);
  const memPct = $derived(memTotal > 0 ? (memUsed / memTotal) * 100 : 0);
</script>

<!-- Apply settings modal -->
<Modal bind:open={showApplyModal} onclose={() => { if (!applyLoading) { showApplyModal = false; applyError = null; } }} title="Apply & Restart">
  <p class="text-sm text-cubelit-text-dim mb-4">Applying environment changes will recreate the container and restart the server. Continue?</p>
  {#if applyError}
    <p class="text-xs text-cubelit-error px-3 py-2 bg-cubelit-error/5 border border-cubelit-error/30 rounded-lg mb-3">{applyError}</p>
  {/if}
  <div class="flex gap-2 justify-end">
    <Button variant="ghost" onclick={() => { showApplyModal = false; applyError = null; }}>Cancel</Button>
    <Button onclick={applySettings} loading={applyLoading}>Apply & Restart</Button>
  </div>
</Modal>

<!-- Delete server modal -->
<Modal bind:open={showDeleteModal} onclose={() => { if (!deleteLoading) { showDeleteModal = false; deleteWithData = false; deleteError = null; } }} title="Delete Server">
  <p class="text-sm text-cubelit-text-dim mb-4">
    Delete <span class="text-cubelit-text font-medium">{server.name}</span>? This will stop and remove the container.
  </p>
  <label class="flex items-start gap-3 cursor-pointer mb-4">
    <input type="checkbox" class="mt-0.5 accent-cubelit-error" bind:checked={deleteWithData} />
    <div>
      <p class="text-sm text-cubelit-text">Also delete server files from disk</p>
      {#if deleteWithData}
        <p class="text-xs text-cubelit-error mt-0.5">Permanently deletes all world data. Cannot be undone.</p>
      {:else}
        <p class="text-xs text-cubelit-muted mt-0.5">Files remain at <span class="font-mono">{server.volume_path}</span></p>
      {/if}
    </div>
  </label>
  {#if deleteError}
    <p class="text-xs text-cubelit-error px-3 py-2 bg-cubelit-error/5 border border-cubelit-error/30 rounded-lg mb-3">{deleteError}</p>
  {/if}
  <div class="flex gap-2 justify-end">
    <Button variant="ghost" onclick={() => { showDeleteModal = false; deleteWithData = false; deleteError = null; }} disabled={deleteLoading}>Cancel</Button>
    <Button variant="danger" onclick={handleDelete} loading={deleteLoading}>Delete Server</Button>
  </div>
</Modal>

<!-- Delete file modal -->
<Modal bind:open={showDeleteFileModal} onclose={() => { deleteFileName = null; showDeleteFileModal = false; }} title="Delete File">
  <p class="text-sm text-cubelit-text-dim mb-4">Delete <span class="text-cubelit-text font-medium">{deleteFileName}</span>? This cannot be undone.</p>
  <div class="flex gap-2 justify-end">
    <Button variant="ghost" onclick={() => { deleteFileName = null; showDeleteFileModal = false; }}>Cancel</Button>
    <Button variant="danger" onclick={confirmDeleteFile}>Delete</Button>
  </div>
</Modal>

<!-- Tabs -->
<div class="flex gap-0.5 border-b border-cubelit-border mb-5" style="margin-bottom: 20px;">
  {#each (['overview', 'console', 'files', 'settings'] as const) as t}
    <button
      type="button"
      onclick={() => {
        tab = t;
        if (t === 'files') loadFiles();
      }}
      class="px-3.5 py-2.5 text-[13px] capitalize transition-colors border-b-2 -mb-px"
      style="
        color: {tab === t ? 'var(--c-text)' : 'var(--c-text-dim)'};
        border-bottom-color: {tab === t ? 'var(--c-accent)' : 'transparent'};
      "
    >{t}</button>
  {/each}
</div>

<!-- ── Overview tab ── -->
{#if tab === 'overview'}
  <div style="display: grid; grid-template-columns: 2fr 1fr; gap: 14px;">
    <!-- Left column -->
    <div class="flex flex-col gap-3.5">
      <!-- Gauge cards 2x2 -->
      <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px;">
        <GaugeCard label="CPU" value="{cpuPct.toFixed(1)}%" bar={cpuPct} />
        <GaugeCard
          label="Memory"
          value="{memUsed.toFixed(1)} / {memTotal.toFixed(1)} GB"
          bar={memPct}
        />
        <GaugeCard label="Status" value={server.status} />
        <GaugeCard label="Created" value={server.created_at?.slice(0, 10) ?? '—'} />
      </div>

      <!-- Connection -->
      <div class="bg-cubelit-surface border border-cubelit-border rounded-2xl p-4">
        <div class="text-[11px] text-cubelit-muted font-mono uppercase tracking-widest mb-3">Connect</div>
        <div class="flex flex-col gap-2">
          <ConnRow label="Local" value={getAddress()} />
          <ConnRow label="Public" value={publicIp ? `${publicIp}:${getAddress().split(':')[1] ?? ''}` : '—'} />
        </div>
        <p class="text-[11px] text-cubelit-muted mt-3">Share your public address with friends. Port forwarding may be required.</p>
      </div>

      <!-- CPU Sparkline -->
      {#if server.status === 'running'}
        <div class="bg-cubelit-surface border border-cubelit-border rounded-2xl p-4">
          <div class="flex justify-between items-center mb-2">
            <div class="text-[11px] text-cubelit-muted font-mono uppercase tracking-widest">CPU · last 60s</div>
            <div class="text-[11px] font-mono text-cubelit-text-dim">{cpuPct.toFixed(1)}%</div>
          </div>
          <Sparkline base={cpuPct} {hue} seed={1} height={72} />
        </div>
        <div class="bg-cubelit-surface border border-cubelit-border rounded-2xl p-4">
          <div class="flex justify-between items-center mb-2">
            <div class="text-[11px] text-cubelit-muted font-mono uppercase tracking-widest">Memory · last 60s</div>
            <div class="text-[11px] font-mono text-cubelit-text-dim">{memUsed.toFixed(1)} / {memTotal.toFixed(1)} GB</div>
          </div>
          <Sparkline base={memPct} hue={(hue + 200) % 360} seed={2} height={72} />
        </div>
      {/if}
    </div>

    <!-- Right column -->
    <div class="flex flex-col gap-3.5">
      <!-- Details -->
      <div class="bg-cubelit-surface border border-cubelit-border rounded-2xl p-4">
        <div class="text-[11px] text-cubelit-muted font-mono uppercase tracking-widest mb-3">Details</div>
        <div class="flex flex-col gap-2 text-xs">
          {#each [
            ['Image', server.docker_image],
            ['Volume', server.volume_path],
            ['Created', server.created_at?.slice(0, 10) ?? '—'],
            ['Container', server.container_id ? server.container_id.slice(0, 12) : '—'],
          ] as [k, v]}
            <div class="flex justify-between gap-2">
              <span class="text-cubelit-muted shrink-0">{k}</span>
              <span class="font-mono text-cubelit-text-dim truncate text-right">{v}</span>
            </div>
          {/each}
        </div>
      </div>

      <!-- Quick files -->
      <div class="bg-cubelit-surface border border-cubelit-border rounded-2xl p-4">
        <div class="text-[11px] text-cubelit-muted font-mono uppercase tracking-widest mb-3">Files</div>
        <div class="flex flex-col gap-1.5">
          <button
            type="button"
            class="w-full flex items-center gap-2 px-3 py-2 rounded-lg border border-cubelit-border bg-cubelit-bg-2 text-sm text-cubelit-text-dim hover:text-cubelit-text hover:border-cubelit-border-2 transition-colors"
            onclick={() => invoke('open_folder', { path: server.volume_path }).catch(() => {})}
          >
            <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75">
              <path stroke-linecap="round" stroke-linejoin="round" d="M3 7a2 2 0 0 1 2-2h4l2 3h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>
            </svg>
            Open server folder
          </button>
          <button
            type="button"
            class="w-full flex items-center gap-2 px-3 py-2 rounded-lg border border-cubelit-border bg-cubelit-bg-2 text-sm text-cubelit-text-dim hover:text-cubelit-text hover:border-cubelit-border-2 transition-colors"
            onclick={() => backupServer(server.id).catch(() => {})}
          >
            <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75">
              <path stroke-linecap="round" stroke-linejoin="round" d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/>
            </svg>
            Backup now
          </button>
        </div>
      </div>
    </div>
  </div>

<!-- ── Console / Logs tab ── -->
{:else if tab === 'console'}
  <div class="bg-cubelit-surface border border-cubelit-border rounded-2xl overflow-hidden flex flex-col" style="height: 440px;">
    <!-- Log output -->
    <div
      bind:this={logContainer}
      onscroll={handleLogScroll}
      class="flex-1 overflow-auto p-3.5 font-mono text-xs leading-relaxed"
      style="background: #0d1117;"
    >
      {#if logLoading && logLines.length === 0}
        <p class="text-cubelit-muted py-4 text-center">Loading logs…</p>
      {:else if logLines.length === 0}
        <p class="text-cubelit-muted py-4 text-center">No log output</p>
      {:else}
        {#each logLines as line}
          <div class="text-gray-300 whitespace-pre-wrap break-all leading-5">{line}</div>
        {/each}
      {/if}
    </div>
    <!-- Controls bar -->
    <div class="border-t border-cubelit-border px-3.5 py-2 flex items-center justify-between bg-cubelit-bg-2">
      <div class="flex gap-3">
        <button
          type="button"
          onclick={() => {
            logFollow = !logFollow;
            if (logFollow && logContainer) logContainer.scrollTop = logContainer.scrollHeight;
          }}
          class="text-xs transition-colors {logFollow ? 'text-cubelit-accent' : 'text-cubelit-muted hover:text-cubelit-text'}"
        >{logFollow ? 'Following' : 'Follow'}</button>
        <button
          type="button"
          class="text-xs text-cubelit-muted hover:text-cubelit-text transition-colors"
          onclick={loadLogs}
        >Refresh</button>
      </div>
      <span class="text-[11px] text-cubelit-muted font-mono">{logLines.length} lines</span>
    </div>
  </div>

<!-- ── Files tab ── -->
{:else if tab === 'files'}
  <div class="bg-cubelit-surface border border-cubelit-border rounded-2xl overflow-hidden">
    <div class="p-4 border-b border-cubelit-border flex justify-between items-center">
      <div>
        <div class="text-sm font-semibold text-cubelit-text">Files · {files.length}</div>
        <div class="text-xs text-cubelit-text-dim">Drop files to upload</div>
      </div>
      <div class="flex gap-2">
        <button type="button" onclick={() => invoke('open_folder', { path: server.volume_path }).catch(() => {})}
          class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium border border-cubelit-border bg-cubelit-bg-2 text-cubelit-text-dim hover:text-cubelit-text hover:border-cubelit-border-2 transition-colors">
          <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75"><path stroke-linecap="round" stroke-linejoin="round" d="M3 7a2 2 0 0 1 2-2h4l2 3h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/></svg>
          Open folder
        </button>
        <button type="button" onclick={handleUpload}
          class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-cubelit-accent text-white hover:brightness-110 transition-colors">
          <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12"/></svg>
          Upload
        </button>
      </div>
    </div>
    {#if filesLoading}
      <p class="text-cubelit-muted text-sm py-8 text-center">Loading files…</p>
    {:else if files.length === 0}
      <p class="text-cubelit-muted text-sm py-8 text-center">No files in this server's volume.</p>
    {:else}
      {#each files as file, i}
        <div class="flex items-center gap-3 px-4 py-2.5 {i < files.length - 1 ? 'border-b border-cubelit-border' : ''}">
          <div class="w-8 h-6 rounded flex items-center justify-center text-[9px] font-bold text-cubelit-accent bg-cubelit-accent-soft font-mono shrink-0">
            {file.name.split('.').pop()?.toUpperCase().slice(0, 4) ?? 'FILE'}
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-sm text-cubelit-text truncate">{file.name}</div>
            <div class="text-xs text-cubelit-muted font-mono">{(file.size / 1024).toFixed(1)} KB</div>
          </div>
          <button
            type="button"
            aria-label="Delete {file.name}"
            onclick={() => { deleteFileName = file.name; showDeleteFileModal = true; }}
            class="text-cubelit-muted hover:text-cubelit-error transition-colors p-1"
          >
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75">
              <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12"/>
            </svg>
          </button>
        </div>
      {/each}
    {/if}
  </div>

<!-- ── Settings tab ── -->
{:else if tab === 'settings'}
  <div class="flex flex-col gap-4">
    {#if server.recipe_id === 'ark-survival-ascended'}
      <div class="bg-cubelit-surface border border-cubelit-border rounded-2xl p-5">
        <div class="flex justify-between items-start gap-4 mb-4">
          <div>
            <div class="text-sm font-semibold text-cubelit-text">ARK server config</div>
            <div class="text-xs text-cubelit-text-dim">Edits Game.ini and GameUserSettings.ini. Changes apply after the server restarts.</div>
          </div>
          <Button onclick={saveAsaConfig} loading={asaConfigSaving} disabled={asaConfigLoading}>
            <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
              <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5"/>
            </svg>
            Save config
          </Button>
        </div>

        {#if asaConfigError}
          <p class="text-xs text-cubelit-error px-3 py-2 bg-cubelit-error/5 border border-cubelit-error/30 rounded-lg mb-3">{asaConfigError}</p>
        {:else if asaConfigSaved}
          <p class="text-xs text-cubelit-accent px-3 py-2 bg-cubelit-accent-soft border border-cubelit-accent/30 rounded-lg mb-3">Config saved. Restart the server when you want these changes to take effect.</p>
        {/if}

        {#if asaConfigLoading}
          <p class="text-cubelit-muted text-sm py-8 text-center">Loading ARK config…</p>
        {:else}
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <label class="flex flex-col gap-1.5">
              <span class="text-xs font-medium text-cubelit-text-dim">Server name</span>
              <input
                class="w-full px-3 py-2.5 rounded-lg text-sm text-cubelit-text bg-cubelit-bg-2 border border-cubelit-border focus:outline-none focus:border-cubelit-accent transition-colors"
                value={asaConfig.sessionName}
                oninput={(e) => { asaConfig.sessionName = (e.target as HTMLInputElement).value; asaConfigSaved = false; }}
              />
            </label>

            <label class="flex flex-col gap-1.5">
              <span class="text-xs font-medium text-cubelit-text-dim">Max players</span>
              <input
                class="w-full px-3 py-2.5 rounded-lg text-sm text-cubelit-text bg-cubelit-bg-2 border border-cubelit-border focus:outline-none focus:border-cubelit-accent transition-colors"
                type="number"
                min="1"
                max="200"
                value={asaConfig.maxPlayers}
                oninput={(e) => { asaConfig.maxPlayers = (e.target as HTMLInputElement).value; asaConfigSaved = false; }}
              />
            </label>

            <label class="flex items-center justify-between gap-3 rounded-lg border border-cubelit-border bg-cubelit-bg-2 px-3 py-2.5">
              <span class="text-sm text-cubelit-text">PVE server</span>
              <button
                type="button"
                aria-label="Toggle PVE server"
                class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {asaConfig.serverPve ? 'bg-cubelit-accent' : 'bg-cubelit-border'}"
                aria-pressed={asaConfig.serverPve}
                onclick={() => { asaConfig.serverPve = !asaConfig.serverPve; asaConfigSaved = false; }}
              >
                <span class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {asaConfig.serverPve ? 'translate-x-6' : 'translate-x-1'}"></span>
              </button>
            </label>

            <label class="flex items-center justify-between gap-3 rounded-lg border border-cubelit-border bg-cubelit-bg-2 px-3 py-2.5">
              <span class="text-sm text-cubelit-text">Player speed leveling</span>
              <button
                type="button"
                aria-label="Toggle player speed leveling"
                class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {asaConfig.allowSpeedLeveling ? 'bg-cubelit-accent' : 'bg-cubelit-border'}"
                aria-pressed={asaConfig.allowSpeedLeveling}
                onclick={() => { asaConfig.allowSpeedLeveling = !asaConfig.allowSpeedLeveling; asaConfigSaved = false; }}
              >
                <span class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {asaConfig.allowSpeedLeveling ? 'translate-x-6' : 'translate-x-1'}"></span>
              </button>
            </label>

            <label class="flex items-center justify-between gap-3 rounded-lg border border-cubelit-border bg-cubelit-bg-2 px-3 py-2.5">
              <span class="text-sm text-cubelit-text">Flyer speed leveling</span>
              <button
                type="button"
                aria-label="Toggle flyer speed leveling"
                class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {asaConfig.allowFlyerSpeedLeveling ? 'bg-cubelit-accent' : 'bg-cubelit-border'}"
                aria-pressed={asaConfig.allowFlyerSpeedLeveling}
                onclick={() => { asaConfig.allowFlyerSpeedLeveling = !asaConfig.allowFlyerSpeedLeveling; asaConfigSaved = false; }}
              >
                <span class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {asaConfig.allowFlyerSpeedLeveling ? 'translate-x-6' : 'translate-x-1'}"></span>
              </button>
            </label>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-4">
            {#each [
              ['XP multiplier', 'xpMultiplier', 0.1, 10, 0.1],
              ['Taming speed', 'tamingSpeedMultiplier', 1, 5000, 1],
              ['Harvest amount', 'harvestAmountMultiplier', 0.5, 20, 0.5],
              ['Water drain', 'playerWaterDrainMultiplier', 0, 5, 0.1],
              ['Food drain', 'playerFoodDrainMultiplier', 0, 5, 0.1],
              ['Oxygen per level', 'oxygenPerLevel', 1, 1000, 1],
            ] as [label, key, min, max, step]}
              <label class="flex flex-col gap-2 rounded-lg border border-cubelit-border bg-cubelit-bg-2 px-3 py-3">
                <div class="flex items-center justify-between gap-3">
                  <span class="text-xs font-medium text-cubelit-text-dim">{label}</span>
                  <input
                    class="w-24 px-2 py-1 rounded-md text-xs font-mono text-cubelit-text bg-cubelit-bg border border-cubelit-border focus:outline-none focus:border-cubelit-accent"
                    type="number"
                    {min}
                    {max}
                    {step}
                    value={asaConfig[key as keyof AsaConfig] as string}
                    oninput={(e) => { asaConfig[key as keyof AsaConfig] = (e.target as HTMLInputElement).value as never; asaConfigSaved = false; }}
                  />
                </div>
                <input
                  type="range"
                  {min}
                  {max}
                  {step}
                  value={asaConfig[key as keyof AsaConfig] as string}
                  oninput={(e) => { asaConfig[key as keyof AsaConfig] = (e.target as HTMLInputElement).value as never; asaConfigSaved = false; }}
                  class="w-full accent-cubelit-accent"
                />
              </label>
            {/each}
          </div>
        {/if}
      </div>
    {/if}

    <div class="bg-cubelit-surface border border-cubelit-border rounded-2xl p-5">
      <div class="flex justify-between items-start mb-4">
        <div>
          <div class="text-sm font-semibold text-cubelit-text">Environment variables</div>
          <div class="text-xs text-cubelit-text-dim">Applying changes recreates the container.</div>
        </div>
        <Button onclick={() => { showApplyModal = true; }}>
          <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
            <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5"/>
          </svg>
          Apply & restart
        </Button>
      </div>
      <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px;">
        {#each envEntries as [key]}
          <label class="flex flex-col gap-1.5">
            <span class="text-[11px] text-cubelit-text-dim font-mono">{key}</span>
            <input
              class="w-full px-3 py-2.5 rounded-lg text-sm font-mono text-cubelit-text bg-cubelit-bg-2 border border-cubelit-border focus:outline-none focus:border-cubelit-accent transition-colors"
              value={envEdited[key] ?? ''}
              oninput={(e) => { envEdited[key] = (e.target as HTMLInputElement).value; }}
            />
          </label>
        {/each}
      </div>
    </div>

    <!-- Danger zone -->
    <div class="rounded-2xl border p-4" style="border-color: color-mix(in oklab, var(--c-error) 30%, var(--c-border)); background: color-mix(in oklab, var(--c-error) 7%, transparent);">
      <div class="text-sm font-semibold text-cubelit-error mb-1">Danger zone</div>
      <div class="text-xs text-cubelit-text-dim mb-3">Deleting removes the container. World files can be kept.</div>
      <Button variant="danger" onclick={() => { showDeleteModal = true; }}>
        <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75">
          <path stroke-linecap="round" stroke-linejoin="round" d="M4 7h16M9 7V4h6v3M6 7l1 13h10l1-13"/>
        </svg>
        Delete server
      </Button>
    </div>
  </div>
{/if}
