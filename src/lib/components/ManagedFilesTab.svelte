<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { Cubelit } from "$lib/types/server";
  import type { FileEntry } from "$lib/types/files";
  import type { RecipeFileTab } from "$lib/types/recipe";
  import { listServerFiles, copyFileToServer, deleteServerFile } from "$lib/api/files";
  import Button from "$lib/components/Button.svelte";
  import Modal from "$lib/components/Modal.svelte";

  interface Props {
    server: Cubelit;
    tab: RecipeFileTab;
  }

  let { server, tab }: Props = $props();

  let files = $state<FileEntry[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let deleteName = $state<string | null>(null);
  let showDelete = $state(false);

  const allowUpload = $derived(tab.upload !== false);
  const exts = $derived((tab.file_types ?? []).map((e) => e.replace(/^\./, "")));

  function matches(name: string): boolean {
    if (exts.length === 0) return true;
    return exts.some((e) => name.toLowerCase().endsWith(`.${e.toLowerCase()}`));
  }

  async function load() {
    loading = true;
    error = null;
    try {
      const all = await listServerFiles(server.id, tab.path);
      files = all.filter((f) => f.is_dir || matches(f.name));
    } catch {
      error = `Could not list ${tab.path}. The folder may not exist until the server has run once.`;
      files = [];
    } finally {
      loading = false;
    }
  }

  async function upload() {
    error = null;
    try {
      const selected = await open({
        title: `Upload to ${tab.label}`,
        multiple: false,
        filters: exts.length > 0 ? [{ name: tab.label, extensions: exts }] : undefined,
      });
      if (!selected) return;
      const path = selected as string;
      const filename = path.split(/[/\\]/).pop() ?? "file";
      await copyFileToServer(server.id, path, `${tab.path}/${filename}`);
      await load();
    } catch (e) {
      error = String(e);
    }
  }

  async function confirmDelete() {
    if (!deleteName) return;
    try {
      await deleteServerFile(server.id, `${tab.path}/${deleteName}`);
      await load();
    } catch (e) {
      error = String(e);
    } finally {
      deleteName = null;
      showDelete = false;
    }
  }

  async function openFolder() {
    const sep = server.volume_path.includes("\\") ? "\\" : "/";
    await invoke("open_folder", { path: `${server.volume_path}${sep}${tab.path}` }).catch(() => {});
  }

  $effect(() => {
    // Re-run when the tab path changes.
    void tab.path;
    load();
  });
</script>

<Modal bind:open={showDelete} onclose={() => { deleteName = null; showDelete = false; }} title="Delete File">
  <p class="text-sm text-cubelit-muted mb-6">Delete <span class="text-cubelit-text font-medium">{deleteName}</span>? This cannot be undone.</p>
  <div class="flex gap-3 justify-end">
    <Button variant="ghost" onclick={() => { deleteName = null; showDelete = false; }}>Cancel</Button>
    <Button variant="danger" onclick={confirmDelete}>Delete</Button>
  </div>
</Modal>

<div class="space-y-4">
  <div class="flex items-center justify-between">
    <div>
      <h3 class="text-sm font-medium text-cubelit-text">{tab.label}</h3>
      {#if exts.length > 0}
        <p class="text-xs text-cubelit-muted">Accepts {tab.file_types!.join(", ")}</p>
      {/if}
    </div>
    <div class="flex items-center gap-2">
      <button
        type="button"
        class="text-xs text-cubelit-muted hover:text-cubelit-accent transition-colors"
        onclick={openFolder}
      >Open Folder</button>
      {#if allowUpload}
        <Button size="sm" onclick={upload}>Upload</Button>
      {/if}
    </div>
  </div>

  {#if error}
    <p class="text-xs text-cubelit-warning py-3 text-center">{error}</p>
  {:else if loading}
    <p class="text-cubelit-muted text-sm py-8 text-center">Loading…</p>
  {:else if files.length === 0}
    <div class="text-center py-12 bg-cubelit-surface border border-dashed border-cubelit-border rounded-xl">
      <p class="text-cubelit-muted text-sm">Nothing here yet</p>
    </div>
  {:else}
    <div class="space-y-2">
      {#each files as f}
        <div class="flex items-center justify-between bg-cubelit-surface border border-cubelit-border rounded-xl px-4 py-3">
          <div class="flex items-center gap-3 min-w-0">
            {#if f.is_dir}
              <svg class="w-4 h-4 text-cubelit-accent shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.75"><path stroke-linecap="round" stroke-linejoin="round" d="M3 7a2 2 0 0 1 2-2h4l2 3h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/></svg>
            {/if}
            <div class="min-w-0">
              <p class="text-sm text-cubelit-text truncate">{f.name}</p>
              {#if !f.is_dir}
                <p class="text-xs text-cubelit-muted">{(f.size / 1024).toFixed(1)} KB</p>
              {/if}
            </div>
          </div>
          <button
            type="button"
            class="text-xs text-cubelit-error hover:text-cubelit-error/80 transition-colors shrink-0"
            onclick={() => { deleteName = f.name; showDelete = true; }}
          >Remove</button>
        </div>
      {/each}
    </div>
  {/if}
</div>
