<script lang="ts">
  import type { Recipe } from "$lib/types/recipe";
  import PortInput from "$lib/components/PortInput.svelte";

  interface Props {
    recipe: Recipe;
    serverName: string;
    envValues: Record<string, string>;
    portValues: Record<string, number>;
    volumePath: string;
    onenvchange: (key: string, value: string) => void;
    onportchange: (containerPort: string, hostPort: number) => void;
    onname: (name: string) => void;
    onvolumepath: (path: string) => void;
    ontagchange: (tag: string) => void;
  }

  let {
    recipe,
    serverName = $bindable(""),
    envValues,
    portValues,
    volumePath = $bindable(""),
    onenvchange,
    onportchange,
    onname,
    onvolumepath,
    ontagchange,
  }: Props = $props();

  let showAdvancedPorts = $state(false);

  const serverTypes = ["VANILLA", "FORGE", "FABRIC", "PAPER", "SPIGOT", "BUKKIT"];
  const ramOptions = ["1G", "2G", "4G", "6G", "8G"];
  const difficulties = ["peaceful", "easy", "normal", "hard"];

  // ─── Modpack browser ────────────────────────────────────────────────────────

  interface FtbPack {
    id: number;
    name: string;
    description: string;
    iconUrl: string;
    plays: number;
    versions: { id: string; name: string; specs?: { minecraft?: string } }[];
  }

  interface ModrinthPack {
    slug: string;
    title: string;
    description: string;
    icon_url?: string;
    downloads: number;
    versions: string[];
  }

  interface PackVersion {
    id: string;
    name: string;
    minecraftVersion: string;
  }

  let useModpack = $state(false);
  let modpackSource = $state<"ftb" | "modrinth">("modrinth");
  let searchQuery = $state("");
  let ftbPacks = $state<FtbPack[]>([]);
  let modrinthResults = $state<ModrinthPack[]>([]);
  let selectedPackId = $state<number | string | null>(null);
  let selectedPackName = $state("");
  let packVersions = $state<PackVersion[]>([]);
  let selectedVersionId = $state("");
  let browserLoading = $state(false);
  let versionsLoading = $state(false);
  let browserError = $state<string | null>(null);
  let searchDebounce: ReturnType<typeof setTimeout> | null = null;

  const MODRINTH_HEADERS = { "User-Agent": "Cubelit/0.1 (github.com/cubelit)" };
  const FTB_BASE = "https://api.modpacks.ch";
  const MODRINTH_BASE = "https://api.modrinth.com/v2";

  // ─── Java version selection ───────────────────────────────────────────────
  const javaTags = [
    { tag: "latest", label: "Java 21+", mc: "MC 1.21+" },
    { tag: "java17", label: "Java 17",  mc: "MC 1.17 – 1.20.x" },
    { tag: "java8",  label: "Java 8",   mc: "MC 1.16 and older" },
  ];
  let javaTag = $state("latest");
  let javaAutoDetected = $state<string | null>(null); // label of the auto-detected reason

  function setJavaTag(tag: string) {
    javaTag = tag;
    ontagchange(tag);
  }

  function suggestJavaTag(mcVersion: string): string {
    const m = mcVersion.match(/^1\.(\d+)/);
    if (!m) return "latest";
    const minor = parseInt(m[1]);
    if (minor >= 21) return "latest";
    if (minor >= 17) return "java17";
    return "java8";
  }

  function autoDetectJava(mcVersion: string) {
    if (!mcVersion) return;
    const suggested = suggestJavaTag(mcVersion);
    setJavaTag(suggested);
    javaAutoDetected = `Auto-detected for MC ${mcVersion}`;
  }

  function applyModpackEnv() {
    if (selectedPackId === null || !selectedVersionId) return;
    if (modpackSource === "ftb") {
      onenvchange("TYPE", "FTBA");
      onenvchange("FTB_MODPACK_ID", String(selectedPackId));
      onenvchange("FTB_MODPACK_VERSION_ID", selectedVersionId);
      // Clear Modrinth vars if any
      onenvchange("MODRINTH_MODPACK", "");
      onenvchange("MODRINTH_VERSION", "");
    } else {
      onenvchange("TYPE", "MODRINTH");
      onenvchange("MODRINTH_MODPACK", String(selectedPackId));
      onenvchange("MODRINTH_VERSION", selectedVersionId);
      // Clear FTB vars if any
      onenvchange("FTB_MODPACK_ID", "");
      onenvchange("FTB_MODPACK_VERSION_ID", "");
    }
  }

  function clearModpackEnv() {
    // Restore vanilla defaults
    onenvchange("TYPE", "VANILLA");
    onenvchange("VERSION", "LATEST");
    onenvchange("MEMORY", "2G");
    javaAutoDetected = null;
    setJavaTag("latest");
    onenvchange("FTB_MODPACK_ID", "");
    onenvchange("FTB_MODPACK_VERSION_ID", "");
    onenvchange("MODRINTH_MODPACK", "");
    onenvchange("MODRINTH_VERSION", "");
    selectedPackId = null;
    selectedPackName = "";
    packVersions = [];
    selectedVersionId = "";
  }

  function handleModpackToggle() {
    useModpack = !useModpack;
    if (useModpack) {
      // Modpacks need more RAM — bump to 4G if still at the vanilla default
      if ((envValues["MEMORY"] ?? "2G") === "2G") onenvchange("MEMORY", "4G");
      loadSource();
    } else {
      clearModpackEnv();
    }
  }

  function switchSource(source: "ftb" | "modrinth") {
    modpackSource = source;
    selectedPackId = null;
    selectedPackName = "";
    packVersions = [];
    selectedVersionId = "";
    searchQuery = "";
    loadSource();
  }

  function loadSource() {
    if (modpackSource === "ftb") {
      loadFtbPopular();
    } else {
      searchModrinth("");
    }
  }

  async function loadFtbPopular() {
    browserLoading = true;
    browserError = null;
    ftbPacks = [];
    try {
      const res = await fetch(`${FTB_BASE}/public/modpack/popular/installs/20/0`);
      if (!res.ok) throw new Error(`FTB popular: HTTP ${res.status}`);
      const data = await res.json() as { packs: number[]; status: string };
      if (!Array.isArray(data.packs)) throw new Error(`Unexpected FTB response: ${JSON.stringify(data).slice(0, 200)}`);
      const ids = data.packs.slice(0, 12);
      const results = await Promise.allSettled(
        ids.map(async (id): Promise<FtbPack> => {
          const r = await fetch(`${FTB_BASE}/public/modpack/${id}`);
          if (!r.ok) throw new Error(`Pack ${id}: HTTP ${r.status}`);
          const p = await r.json();
          // FTB API uses p.art[] array, not p.artwork
          const artArr = p.art as Array<{ type: string; url: string }> | undefined;
          const iconUrl = artArr?.find((a) => a.type === "square")?.url ?? artArr?.[0]?.url ?? "";
          // FTB versions: id is a number, minecraft version lives in targets[]
          const versions = (p.versions as Array<{
            id: number; name: string;
            targets?: Array<{ type: string; version: string }>;
          }> | undefined) ?? [];
          return {
            id: p.id as number,
            name: p.name as string,
            description: ((p.synopsis ?? p.description) as string | undefined)?.slice(0, 120) ?? "",
            iconUrl,
            plays: (p.installs ?? p.plays ?? 0) as number,
            versions: versions.map((v) => {
              const mc = v.targets?.find((t) => t.type === "game")?.version;
              return { id: String(v.id), name: v.name, ...(mc ? { specs: { minecraft: mc } } : {}) };
            }),
          };
        })
      );
      ftbPacks = (results.filter((r) => r.status === "fulfilled") as PromiseFulfilledResult<FtbPack>[])
        .map((r) => r.value)
        .filter((p) => p.name);
      if (ftbPacks.length === 0) {
        const firstRej = results.find((r) => r.status === "rejected") as PromiseRejectedResult | undefined;
        if (firstRej) console.error("FTB pack detail failed:", firstRej.reason);
        browserError = "Failed to load FTB packs. Check your internet connection.";
      }
    } catch (e) {
      console.error("FTB load error:", e);
      browserError = "Failed to load FTB packs. Check your internet connection.";
    } finally {
      browserLoading = false;
    }
  }

  async function searchModrinth(query: string) {
    browserLoading = true;
    browserError = null;
    modrinthResults = [];
    try {
      const params = new URLSearchParams({
        facets: JSON.stringify([["project_type:modpack"]]),
        index: "downloads",
        limit: "20",
        query,
      });
      const res = await fetch(`${MODRINTH_BASE}/search?${params}`, { headers: MODRINTH_HEADERS });
      const data: { hits: ModrinthPack[] } = await res.json();
      modrinthResults = data.hits ?? [];
    } catch {
      browserError = "Failed to search Modrinth. Check your internet connection.";
    } finally {
      browserLoading = false;
    }
  }

  function onSearchInput() {
    if (searchDebounce) clearTimeout(searchDebounce);
    searchDebounce = setTimeout(() => {
      if (modpackSource === "modrinth") searchModrinth(searchQuery);
    }, 400);
  }

  async function selectFtbPack(pack: FtbPack) {
    selectedPackId = pack.id;
    selectedPackName = pack.name;
    packVersions = [...pack.versions].reverse().map((v) => ({
      id: v.id,
      name: v.name,
      minecraftVersion: v.specs?.minecraft ?? "",
    }));
    selectedVersionId = packVersions[0]?.id ?? "";
    autoDetectJava(packVersions[0]?.minecraftVersion ?? "");
    applyModpackEnv();
  }

  async function selectModrinthPack(pack: ModrinthPack) {
    selectedPackId = pack.slug;
    selectedPackName = pack.title;
    versionsLoading = true;
    packVersions = [];
    selectedVersionId = "";
    try {
      const res = await fetch(
        `${MODRINTH_BASE}/project/${pack.slug}/version?include_changelog=false`,
        { headers: MODRINTH_HEADERS }
      );
      const versions: {
        id: string;
        name: string;
        version_number: string;
        game_versions: string[];
      }[] = await res.json();
      packVersions = versions.map((v) => ({
        id: v.id,
        name: v.name || v.version_number,
        minecraftVersion: v.game_versions?.[0] ?? "",
      }));
      selectedVersionId = packVersions[0]?.id ?? "";
      autoDetectJava(packVersions[0]?.minecraftVersion ?? "");
      applyModpackEnv();
    } catch {
      browserError = "Failed to load versions for this pack.";
    } finally {
      versionsLoading = false;
    }
  }

  function selectVersion(id: string) {
    selectedVersionId = id;
    applyModpackEnv();
  }

  // Client-side filter for FTB (no server search)
  let filteredFtbPacks = $derived(
    searchQuery.trim()
      ? ftbPacks.filter((p) => p.name.toLowerCase().includes(searchQuery.toLowerCase()))
      : ftbPacks
  );

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
  <h2 class="text-lg font-semibold text-cubelit-text">Configure Minecraft Server</h2>

  <!-- Server Name -->
  <div class="space-y-2">
    <label class="text-sm font-medium text-cubelit-text">Server Name</label>
    <input
      type="text"
      class="w-full px-3 py-2.5 bg-cubelit-bg border border-cubelit-border rounded-lg text-cubelit-text placeholder:text-cubelit-muted/50 focus:outline-none focus:border-cubelit-accent transition-colors"
      bind:value={serverName}
      placeholder="My Minecraft Server"
      oninput={() => onname(serverName)}
    />
  </div>

  <!-- Server Location -->
  <div class="space-y-2">
    <label class="text-sm font-medium text-cubelit-text">Server Location</label>
    <p class="text-xs text-cubelit-muted">Where your server files will be stored.</p>
    <div class="flex gap-2">
      <input
        type="text"
        class="flex-1 px-3 py-2.5 bg-cubelit-bg border border-cubelit-border rounded-lg text-cubelit-text text-sm font-mono focus:outline-none focus:border-cubelit-accent transition-colors"
        bind:value={volumePath}
        oninput={() => onvolumepath(volumePath)}
      />
      <button
        class="px-4 py-2.5 bg-cubelit-surface border border-cubelit-border rounded-lg text-cubelit-text text-sm hover:bg-cubelit-border transition-colors shrink-0"
        onclick={browseFolder}
      >
        Browse
      </button>
    </div>
  </div>

  <!-- Modpack Toggle -->
  <div class="flex items-center justify-between bg-cubelit-surface border border-cubelit-border rounded-xl px-4 py-3">
    <div>
      <p class="text-sm font-medium text-cubelit-text">Use a modpack?</p>
      <p class="text-xs text-cubelit-muted mt-0.5">Browse FTB and Modrinth modpacks</p>
    </div>
    <button
      class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {useModpack ? 'bg-cubelit-accent' : 'bg-cubelit-border'}"
      onclick={handleModpackToggle}
    >
      <span class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {useModpack ? 'translate-x-6' : 'translate-x-1'}" />
    </button>
  </div>

  {#if useModpack}
    <!-- Modpack Browser -->
    <div class="space-y-4">
      <!-- Source Tabs -->
      <div class="flex gap-1 border-b border-cubelit-border">
        {#each (["modrinth", "ftb"] as const) as source}
          <button
            class="px-4 py-2 text-sm font-medium transition-colors relative capitalize {modpackSource === source ? 'text-cubelit-accent' : 'text-cubelit-muted hover:text-cubelit-text'}"
            onclick={() => switchSource(source)}
          >
            {source === "ftb" ? "FTB" : "Modrinth"}
            {#if modpackSource === source}
              <div class="absolute bottom-0 left-0 right-0 h-0.5 bg-cubelit-accent rounded-t" />
            {/if}
          </button>
        {/each}
      </div>

      <!-- Search -->
      <input
        type="text"
        class="w-full px-3 py-2.5 bg-cubelit-bg border border-cubelit-border rounded-lg text-cubelit-text text-sm placeholder:text-cubelit-muted/50 focus:outline-none focus:border-cubelit-accent transition-colors"
        placeholder={modpackSource === "ftb" ? "Filter packs..." : "Search Modrinth modpacks..."}
        bind:value={searchQuery}
        oninput={onSearchInput}
      />

      <!-- Selected Pack Info -->
      {#if selectedPackId !== null}
        <div class="flex items-center gap-2 px-3 py-2 bg-cubelit-accent/10 border border-cubelit-accent/30 rounded-lg">
          <svg class="w-4 h-4 text-cubelit-accent shrink-0" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5" />
          </svg>
          <span class="text-sm text-cubelit-accent font-medium">{selectedPackName}</span>
          <button class="ml-auto text-xs text-cubelit-muted hover:text-cubelit-text" onclick={() => { selectedPackId = null; selectedPackName = ""; packVersions = []; selectedVersionId = ""; clearModpackEnv(); useModpack = false; useModpack = true; loadSource(); }}>
            Change
          </button>
        </div>

        <!-- Version Picker -->
        {#if versionsLoading}
          <p class="text-xs text-cubelit-muted text-center py-3">Loading versions...</p>
        {:else if packVersions.length > 0}
          <div class="space-y-1.5">
            <label class="text-xs text-cubelit-muted">Version</label>
            <div class="relative">
              <select
                class="w-full appearance-none px-3 py-2.5 pr-8 bg-cubelit-bg border border-cubelit-border rounded-lg text-cubelit-text text-sm focus:outline-none focus:border-cubelit-accent transition-colors"
                value={selectedVersionId}
                onchange={(e) => selectVersion((e.target as HTMLSelectElement).value)}
              >
                {#each packVersions as v}
                  <option value={v.id} style="background-color:#23272f;color:#f5f5f6;">{v.name}{v.minecraftVersion ? ` (MC ${v.minecraftVersion})` : ""}</option>
                {/each}
              </select>
              <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-2.5">
                <svg class="w-4 h-4 text-cubelit-muted" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" d="m19.5 8.25-7.5 7.5-7.5-7.5" />
                </svg>
              </div>
            </div>
            <p class="text-xs text-cubelit-muted">Latest version selected by default. The modpack will be downloaded on first server start.</p>
          </div>
        {/if}
      {:else}
        <!-- Pack List -->
        {#if browserError}
          <p class="text-xs text-cubelit-error py-4 text-center">{browserError}</p>
        {:else if browserLoading}
          <p class="text-xs text-cubelit-muted py-6 text-center">Loading packs...</p>
        {:else}
          {@const packs = modpackSource === "ftb" ? filteredFtbPacks : modrinthResults}
          {#if packs.length === 0}
            <p class="text-xs text-cubelit-muted py-6 text-center">No packs found.</p>
          {:else}
            <div class="grid grid-cols-1 gap-2 max-h-72 overflow-y-auto pr-1">
              {#each packs as pack}
                <button
                  class="flex items-center gap-3 px-3 py-2.5 bg-cubelit-surface hover:bg-cubelit-border border border-cubelit-border rounded-lg text-left transition-colors"
                  onclick={() => modpackSource === "ftb" ? selectFtbPack(pack as FtbPack) : selectModrinthPack(pack as ModrinthPack)}
                >
                  {#if modpackSource === "ftb"}
                    {@const ftb = pack as FtbPack}
                    {#if ftb.iconUrl}
                      <img src={ftb.iconUrl} alt={ftb.name} class="w-8 h-8 rounded object-cover shrink-0" />
                    {:else}
                      <div class="w-8 h-8 rounded bg-cubelit-border shrink-0" />
                    {/if}
                    <div class="min-w-0 flex-1">
                      <p class="text-sm font-medium text-cubelit-text truncate">{ftb.name}</p>
                      <p class="text-xs text-cubelit-muted truncate">{ftb.description}</p>
                    </div>
                    <span class="text-xs text-cubelit-muted shrink-0">{ftb.plays.toLocaleString()} plays</span>
                  {:else}
                    {@const mr = pack as ModrinthPack}
                    {#if mr.icon_url}
                      <img src={mr.icon_url} alt={mr.title} class="w-8 h-8 rounded object-cover shrink-0" />
                    {:else}
                      <div class="w-8 h-8 rounded bg-cubelit-border shrink-0" />
                    {/if}
                    <div class="min-w-0 flex-1">
                      <p class="text-sm font-medium text-cubelit-text truncate">{mr.title}</p>
                      <p class="text-xs text-cubelit-muted truncate">{mr.description}</p>
                    </div>
                    <span class="text-xs text-cubelit-muted shrink-0">{mr.downloads.toLocaleString()} DL</span>
                  {/if}
                </button>
              {/each}
            </div>
          {/if}
        {/if}
      {/if}
    </div>
  {:else}
    <!-- Server Type -->
    <div class="space-y-2">
      <label class="text-sm font-medium text-cubelit-text">Server Type</label>
      <div class="grid grid-cols-3 gap-2">
        {#each serverTypes as type}
          <button
            class="px-3 py-2.5 rounded-lg border-2 text-sm font-medium transition-colors {(envValues['TYPE'] ?? 'VANILLA') === type
              ? 'border-cubelit-accent bg-cubelit-accent/10 text-cubelit-accent'
              : 'border-cubelit-border bg-cubelit-surface text-cubelit-muted hover:border-cubelit-accent/40'}"
            onclick={() => onenvchange('TYPE', type)}
          >
            {type}
          </button>
        {/each}
      </div>
    </div>

    <!-- Minecraft Version -->
    <div class="space-y-2">
      <label class="text-sm font-medium text-cubelit-text">Minecraft Version</label>
      <input
        type="text"
        class="w-full px-3 py-2.5 bg-cubelit-bg border border-cubelit-border rounded-lg text-cubelit-text placeholder:text-cubelit-muted/50 focus:outline-none focus:border-cubelit-accent transition-colors"
        value={envValues['VERSION'] ?? 'LATEST'}
        oninput={(e) => onenvchange('VERSION', (e.target as HTMLInputElement).value)}
        placeholder="LATEST"
      />
      <p class="text-xs text-cubelit-muted">Use "LATEST" for the newest version or specify like "1.20.4"</p>
    </div>

  {/if}

  <!-- RAM (always shown) -->
  <div class="space-y-2">
    <label class="text-sm font-medium text-cubelit-text">Server RAM</label>
    <div class="flex gap-2">
      {#each ramOptions as ram}
        <button
          class="flex-1 px-3 py-2.5 rounded-lg border-2 text-sm font-medium transition-colors {(envValues['MEMORY'] ?? '2G') === ram
            ? 'border-cubelit-accent bg-cubelit-accent/10 text-cubelit-accent'
            : 'border-cubelit-border bg-cubelit-surface text-cubelit-muted hover:border-cubelit-accent/40'}"
          onclick={() => onenvchange('MEMORY', ram)}
        >
          {ram}
        </button>
      {/each}
    </div>
  </div>

  <!-- Java Version (always shown) -->
  <div class="space-y-2">
    <div class="flex items-center gap-2">
      <label class="text-sm font-medium text-cubelit-text">Java Version</label>
      {#if javaAutoDetected}
        <span class="text-xs text-cubelit-accent bg-cubelit-accent/10 px-2 py-0.5 rounded-full">{javaAutoDetected}</span>
      {/if}
    </div>
    <div class="flex gap-2">
      {#each javaTags as jt}
        <button
          class="flex-1 px-3 py-2.5 rounded-lg border-2 text-sm font-medium transition-colors {javaTag === jt.tag
            ? 'border-cubelit-accent bg-cubelit-accent/10 text-cubelit-accent'
            : 'border-cubelit-border bg-cubelit-surface text-cubelit-muted hover:border-cubelit-accent/40'}"
          onclick={() => { setJavaTag(jt.tag); javaAutoDetected = null; }}
        >
          <span class="block">{jt.label}</span>
          <span class="block text-xs font-normal opacity-70">{jt.mc}</span>
        </button>
      {/each}
    </div>
    <p class="text-xs text-cubelit-muted">Match this to your modpack's Minecraft version. Mismatches cause crash loops.</p>
  </div>

  <!-- Game Settings (always shown) -->
  <div class="space-y-4">
    <label class="text-sm font-medium text-cubelit-text">Game Settings</label>

    <!-- Difficulty -->
    <div class="space-y-2">
      <label class="text-xs text-cubelit-muted">Difficulty</label>
      <div class="flex gap-2">
        {#each difficulties as diff}
          <button
            class="flex-1 px-3 py-2 rounded-lg border text-xs font-medium capitalize transition-colors {(envValues['DIFFICULTY'] ?? 'normal') === diff
              ? 'border-cubelit-accent bg-cubelit-accent/10 text-cubelit-accent'
              : 'border-cubelit-border bg-cubelit-surface text-cubelit-muted hover:border-cubelit-accent/40'}"
            onclick={() => onenvchange('DIFFICULTY', diff)}
          >
            {diff}
          </button>
        {/each}
      </div>
    </div>

    <!-- Max Players -->
    <div class="flex items-center justify-between">
      <label class="text-xs text-cubelit-muted">Max Players</label>
      <input
        type="number"
        class="w-20 px-3 py-2 bg-cubelit-bg border border-cubelit-border rounded-lg text-cubelit-text text-sm text-center focus:outline-none focus:border-cubelit-accent transition-colors"
        value={envValues['MAX_PLAYERS'] ?? '20'}
        oninput={(e) => onenvchange('MAX_PLAYERS', (e.target as HTMLInputElement).value)}
        min="1"
        max="1000"
      />
    </div>

    <!-- MOTD -->
    <div class="space-y-1">
      <label class="text-xs text-cubelit-muted">Server Message (MOTD)</label>
      <input
        type="text"
        class="w-full px-3 py-2 bg-cubelit-bg border border-cubelit-border rounded-lg text-cubelit-text text-sm focus:outline-none focus:border-cubelit-accent transition-colors"
        value={envValues['MOTD'] ?? 'A Cubelit Minecraft Server'}
        oninput={(e) => onenvchange('MOTD', (e.target as HTMLInputElement).value)}
      />
    </div>

    <!-- Online Mode -->
    <div class="flex items-center justify-between">
      <div>
        <label class="text-xs text-cubelit-muted">Online Mode</label>
        <p class="text-[10px] text-cubelit-muted/70">Verify players with Mojang</p>
      </div>
      <button
        class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {(envValues['ONLINE_MODE'] ?? 'TRUE').toUpperCase() === 'TRUE' ? 'bg-cubelit-accent' : 'bg-cubelit-border'}"
        onclick={() => {
          const current = (envValues['ONLINE_MODE'] ?? 'TRUE').toUpperCase();
          onenvchange('ONLINE_MODE', current === 'TRUE' ? 'FALSE' : 'TRUE');
        }}
      >
        <span class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {(envValues['ONLINE_MODE'] ?? 'TRUE').toUpperCase() === 'TRUE' ? 'translate-x-6' : 'translate-x-1'}" />
      </button>
    </div>
  </div>

  <!-- Advanced: Ports (collapsible) -->
  <div>
    <button
      class="flex items-center gap-2 text-sm text-cubelit-muted hover:text-cubelit-text transition-colors"
      onclick={() => showAdvancedPorts = !showAdvancedPorts}
    >
      <svg class="w-4 h-4 transition-transform {showAdvancedPorts ? 'rotate-90' : ''}" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 4.5l7.5 7.5-7.5 7.5" />
      </svg>
      Advanced: Ports
    </button>

    {#if showAdvancedPorts}
      <div class="mt-3 space-y-3">
        {#each recipe.ports as port}
          <PortInput
            label="{port.label} ({port.protocol.toUpperCase()})"
            containerPort={port.container_port}
            value={portValues[`${port.container_port}/${port.protocol}`] ?? port.default_host_port}
            onchange={(v) => onportchange(`${port.container_port}/${port.protocol}`, v)}
          />
        {/each}
      </div>
    {/if}
  </div>
</div>
