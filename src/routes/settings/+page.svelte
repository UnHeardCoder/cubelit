<script lang="ts">
  import { onMount } from 'svelte';
  import { getThemeStore, type GridMode } from '$lib/stores/theme.svelte';
  import { getSettingsStore, isAbsolutePath } from '$lib/stores/settings.svelte';
  import Cube from '$lib/components/Cube.svelte';

  const themeStore = getThemeStore();
  const settingsStore = getSettingsStore();

  onMount(() => settingsStore.init());

  const installRootInvalid = $derived(
    settingsStore.installRoot.trim() !== '' && !isAbsolutePath(settingsStore.installRoot.trim())
  );

  const gridOptions: { value: GridMode; label: string; preview: string }[] = [
    { value: 'none',  label: 'None',  preview: 'No background pattern' },
    { value: 'dots',  label: 'Dots',  preview: 'Subtle dot grid' },
    { value: 'lines', label: 'Lines', preview: 'Fine line grid' },
    { value: 'cross', label: 'Cross', preview: 'Coarse cross grid' },
  ];
</script>

<div class="p-8 max-w-[720px] mx-auto">
  <!-- Header -->
  <div class="mb-8">
    <div class="text-[11px] text-cubelit-muted font-mono uppercase tracking-widest mb-1.5">Settings</div>
    <h1 class="text-2xl font-semibold tracking-tight text-cubelit-text">Preferences</h1>
    <p class="text-sm text-cubelit-text-dim mt-1">Customize your Cubelit experience.</p>
  </div>

  <!-- Theme -->
  <section class="bg-cubelit-surface border border-cubelit-border rounded-2xl p-5 mb-4">
    <div class="text-[11px] text-cubelit-muted font-mono uppercase tracking-widest mb-4">Appearance</div>

    <div class="flex items-center justify-between mb-5">
      <div>
        <div class="text-sm font-medium text-cubelit-text">Color theme</div>
        <div class="text-xs text-cubelit-text-dim mt-0.5">Choose how Cubelit looks</div>
      </div>
      <!-- Segmented control -->
      <div class="flex gap-1 p-1 bg-cubelit-bg-2 border border-cubelit-border rounded-lg">
        {#each (['dark', 'light'] as const) as t}
          <button
            type="button"
            onclick={() => { if (themeStore.theme !== t) themeStore.toggle(); }}
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-medium transition-all
              {themeStore.theme === t
                ? 'bg-cubelit-accent text-white shadow-sm'
                : 'text-cubelit-text-dim hover:text-cubelit-text'}"
          >
            {#if t === 'dark'}
              <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <path stroke-linecap="round" stroke-linejoin="round" d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z"/>
              </svg>
              Dark
            {:else}
              <svg class="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="4"/>
                <path stroke-linecap="round" d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/>
              </svg>
              Light
            {/if}
          </button>
        {/each}
      </div>
    </div>

    <!-- Grid background -->
    <div>
      <div class="text-sm font-medium text-cubelit-text mb-1">Grid background</div>
      <div class="text-xs text-cubelit-text-dim mb-3">Subtle texture on the main content area</div>
      <div class="grid grid-cols-4 gap-2">
        {#each gridOptions as opt}
          <button
            type="button"
            onclick={() => themeStore.setGrid(opt.value)}
            class="flex flex-col items-center gap-2 p-3 rounded-xl border-2 transition-all text-center
              {themeStore.gridMode === opt.value
                ? 'border-cubelit-accent bg-cubelit-accent/8 text-cubelit-accent'
                : 'border-cubelit-border bg-cubelit-bg-2 text-cubelit-text-dim hover:border-cubelit-border-2'}"
          >
            <!-- Mini preview -->
            <div
              class="w-10 h-8 rounded border border-cubelit-border-2 bg-cubelit-bg overflow-hidden flex items-center justify-center"
              style="
                {opt.value === 'dots' ? 'background-image: radial-gradient(var(--c-grid-line) 1px, transparent 1.4px); background-size: 6px 6px;' : ''}
                {opt.value === 'lines' ? 'background-image: linear-gradient(to right, var(--c-grid-line) 1px, transparent 1px), linear-gradient(to bottom, var(--c-grid-line) 1px, transparent 1px); background-size: 8px 8px;' : ''}
                {opt.value === 'cross' ? 'background-image: linear-gradient(to right, var(--c-grid-line) 1px, transparent 1px), linear-gradient(to bottom, var(--c-grid-line) 1px, transparent 1px); background-size: 16px 16px;' : ''}
              "
            >
              {#if opt.value === 'none'}
                <svg class="w-4 h-4 text-cubelit-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12"/>
                </svg>
              {/if}
            </div>
            <span class="text-[11px] font-medium">{opt.label}</span>
          </button>
        {/each}
      </div>
    </div>
  </section>

  <!-- Storage -->
  <section class="bg-cubelit-surface border border-cubelit-border rounded-2xl p-5 mb-4">
    <div class="text-[11px] text-cubelit-muted font-mono uppercase tracking-widest mb-4">Storage</div>

    <div class="text-sm font-medium text-cubelit-text mb-1">Default install location</div>
    <div class="text-xs text-cubelit-text-dim mb-3">
      Where new servers store their game files. The create wizard prefills
      <span class="font-mono">&lt;location&gt;/&lt;server name&gt;</span> — you can still change it per server.
      Applies to new servers only; existing servers keep their current folder.
    </div>
    <input
      type="text"
      value={settingsStore.installRoot}
      oninput={(e) => settingsStore.setInstallRoot(e.currentTarget.value)}
      placeholder="~/Cubelit (default)"
      spellcheck="false"
      class="w-full bg-cubelit-bg-2 border rounded-lg px-3 py-2 text-sm font-mono text-cubelit-text placeholder-cubelit-muted/40 focus:outline-none transition-colors
        {installRootInvalid ? 'border-cubelit-error' : 'border-cubelit-border focus:border-cubelit-accent'}"
    />
    {#if installRootInvalid}
      <div class="text-xs text-cubelit-error mt-1.5">
        Enter an absolute path (e.g. /mnt/games/Cubelit) — relative paths are ignored.
      </div>
    {/if}
  </section>

  <!-- About -->
  <section class="bg-cubelit-surface border border-cubelit-border rounded-2xl p-5">
    <div class="text-[11px] text-cubelit-muted font-mono uppercase tracking-widest mb-4">About</div>
    <div class="flex items-center gap-4">
      <Cube size={40} />
      <div>
        <div class="text-sm font-semibold text-cubelit-text">Cubelit</div>
        <div class="text-xs text-cubelit-muted font-mono mt-0.5">v0.2.0 · Self-hosted game server manager</div>
      </div>
    </div>
    <div class="mt-4 flex gap-2">
      <a
        href="https://github.com/UnHeardCoder/cubelit"
        target="_blank"
        rel="noopener noreferrer"
        class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium border border-cubelit-border bg-cubelit-bg-2 text-cubelit-text-dim hover:text-cubelit-text hover:border-cubelit-border-2 transition-colors"
      >
        <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 24 24">
          <path d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0 1 12 6.844a9.59 9.59 0 0 1 2.504.337c1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.02 10.02 0 0 0 22 12.017C22 6.484 17.522 2 12 2z"/>
        </svg>
        GitHub
      </a>
    </div>
  </section>
</div>
