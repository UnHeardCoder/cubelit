<script lang="ts">
  import type { GameSetupProps } from "$lib/games/registry";
  import PortInput from "$lib/components/PortInput.svelte";

  let {
    recipe,
    serverName = $bindable(""),
    envValues,
    portValues,
    volumePath = $bindable(""),
    onenvchange,
    onportchange,
    onname,
    onvolumepath = () => {},
    ontagchange: _ontagchange = undefined,
  }: GameSetupProps = $props();

  async function browseFolder() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({ directory: true, title: "Choose Server Location" });
      if (selected) {
        volumePath = selected as string;
        onvolumepath(volumePath);
      }
    } catch (e) {
      console.error("Failed to open folder dialog:", e);
    }
  }
</script>

<div class="space-y-8">
  <h2 class="text-lg font-semibold text-cubelit-text">Configure FiveM Server</h2>

  <!-- txAdmin notice -->
  <div class="flex items-start gap-2 bg-cubelit-surface border border-cubelit-border rounded-lg px-4 py-3">
    <svg class="w-4 h-4 text-cubelit-accent shrink-0 mt-0.5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
      <path stroke-linecap="round" stroke-linejoin="round" d="M11.25 11.25l.041-.02a.75.75 0 0 1 1.063.852l-.708 2.836a.75.75 0 0 0 1.063.853l.041-.021M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9-3.75h.008v.008H12V8.25Z" />
    </svg>
    <p class="text-xs text-cubelit-muted">
      FiveM is managed via <span class="text-cubelit-text font-medium">txAdmin</span>. After creation, open the txAdmin web panel (port 40120) to enter your license key and deploy server resources.
    </p>
  </div>

  <!-- Server Name -->
  <div class="space-y-2">
    <label class="text-sm font-medium text-cubelit-text" for="fivem-server-name">Server Name</label>
    <input
      id="fivem-server-name"
      type="text"
      class="w-full px-3 py-2.5 bg-cubelit-bg border border-cubelit-border rounded-lg text-cubelit-text placeholder:text-cubelit-muted/50 focus:outline-none focus:border-cubelit-accent transition-colors"
      bind:value={serverName}
      placeholder="My FiveM Server"
      oninput={() => { onname(serverName); onenvchange('SERVER_NAME', serverName); }}
    />
  </div>

  <!-- Server Location -->
  <div class="space-y-2">
    <label class="text-sm font-medium text-cubelit-text" for="fivem-server-location">Server Location</label>
    <p class="text-xs text-cubelit-muted">Where your server files and resources will be stored.</p>
    <div class="flex gap-2">
      <input
        id="fivem-server-location"
        type="text"
        class="flex-1 px-3 py-2.5 bg-cubelit-bg border border-cubelit-border rounded-lg text-cubelit-text text-sm font-mono focus:outline-none focus:border-cubelit-accent transition-colors"
        bind:value={volumePath}
        oninput={() => onvolumepath(volumePath)}
      />
      <button
        type="button"
        class="px-4 py-2.5 bg-cubelit-surface border border-cubelit-border rounded-lg text-cubelit-text text-sm hover:bg-cubelit-border transition-colors shrink-0"
        onclick={browseFolder}
      >
        Browse
      </button>
    </div>
  </div>

  <!-- Max Players -->
  <div class="space-y-2">
    <label class="text-sm font-medium text-cubelit-text" for="fivem-max-players">Max Players</label>
    <input
      id="fivem-max-players"
      type="number"
      class="w-full px-3 py-2.5 bg-cubelit-bg border border-cubelit-border rounded-lg text-cubelit-text focus:outline-none focus:border-cubelit-accent transition-colors"
      value={envValues['SV_MAXCLIENTS'] ?? '32'}
      oninput={(e) => onenvchange('SV_MAXCLIENTS', (e.target as HTMLInputElement).value)}
      min="1"
      max="1024"
    />
  </div>

  <!-- Ports -->
  <div class="space-y-3">
    <p class="text-sm font-medium text-cubelit-text">Ports</p>
    {#each recipe.ports as port}
      <PortInput
        label={port.label}
        containerPort={port.container_port}
        value={portValues[`${port.container_port}/${port.protocol}`] ?? port.default_host_port}
        onchange={(v) => onportchange(`${port.container_port}/${port.protocol}`, v)}
      />
    {/each}
  </div>

  <!-- Database (MariaDB) -->
  <div class="space-y-3">
    <p class="text-sm font-medium text-cubelit-text">Database (MariaDB)</p>
    <p class="text-xs text-cubelit-muted">A MariaDB database will be automatically created alongside your FiveM server. Use these credentials in your framework config.</p>

    <div class="space-y-3">
      <div class="space-y-1.5">
        <label class="text-xs text-cubelit-muted" for="fivem-db-password">Root Password <span class="text-cubelit-muted/60">(leave empty for no password)</span></label>
        <input
          id="fivem-db-password"
          type="text"
          class="w-full px-3 py-2.5 bg-cubelit-bg border border-cubelit-border rounded-lg text-cubelit-text text-sm font-mono placeholder:text-cubelit-muted/40 focus:outline-none focus:border-cubelit-accent transition-colors"
          value={envValues['DB_PASSWORD'] ?? ''}
          placeholder="No password"
          oninput={(e) => onenvchange('DB_PASSWORD', (e.target as HTMLInputElement).value)}
        />
      </div>

      <div class="space-y-1.5">
        <label class="text-xs text-cubelit-muted" for="fivem-db-port">Host Port</label>
        <input
          id="fivem-db-port"
          type="number"
          class="w-full px-3 py-2.5 bg-cubelit-bg border border-cubelit-border rounded-lg text-cubelit-text text-sm focus:outline-none focus:border-cubelit-accent transition-colors"
          value={envValues['DB_HOST_PORT'] ?? '3306'}
          oninput={(e) => onenvchange('DB_HOST_PORT', (e.target as HTMLInputElement).value)}
          min="1"
          max="65535"
        />
      </div>

      <div class="bg-cubelit-surface border border-cubelit-border rounded-lg px-3 py-2.5 space-y-1">
        <div class="flex justify-between text-xs">
          <span class="text-cubelit-muted">Username</span>
          <span class="text-cubelit-text font-mono">root</span>
        </div>
        <div class="flex justify-between text-xs">
          <span class="text-cubelit-muted">Database</span>
          <span class="text-cubelit-text font-mono">fivem</span>
        </div>
      </div>
    </div>
  </div>
</div>
