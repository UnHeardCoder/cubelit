<script lang="ts">
  import type { Cubelit } from "$lib/types/server";
  import type { RecipeConfigFile } from "$lib/types/recipe";
  import { readServerFile, writeServerFile } from "$lib/api/files";
  import { getField, setField, supportsFields } from "$lib/config/parsers";
  import Button from "$lib/components/Button.svelte";

  interface Props {
    server: Cubelit;
    configFiles: RecipeConfigFile[];
  }

  let { server, configFiles }: Props = $props();

  // Per-file state keyed by path.
  let raw = $state<Record<string, string>>({});
  let fields = $state<Record<string, Record<string, string>>>({});
  let loading = $state<Record<string, boolean>>({});
  let saving = $state<Record<string, boolean>>({});
  let error = $state<Record<string, string | null>>({});
  let saved = $state<Record<string, boolean>>({});

  function usesFields(cf: RecipeConfigFile): boolean {
    return !!cf.fields && cf.fields.length > 0 && supportsFields(cf.format);
  }

  async function load(cf: RecipeConfigFile) {
    loading[cf.path] = true;
    error[cf.path] = null;
    saved[cf.path] = false;
    try {
      const content = await readServerFile(server.id, cf.path);
      raw[cf.path] = content;
      if (usesFields(cf)) {
        const fv: Record<string, string> = {};
        for (const f of cf.fields!) {
          fv[f.key] = getField(content, cf.format, f.key, f.section) ?? f.default ?? "";
        }
        fields[cf.path] = fv;
      }
    } catch (e) {
      // A missing file is expected before first boot — start from blank.
      raw[cf.path] = "";
      if (usesFields(cf)) {
        const fv: Record<string, string> = {};
        for (const f of cf.fields!) fv[f.key] = f.default ?? "";
        fields[cf.path] = fv;
      }
      error[cf.path] = `Could not read ${cf.path} yet — it may not exist until the server has run once. (${String(e)})`;
    } finally {
      loading[cf.path] = false;
    }
  }

  async function save(cf: RecipeConfigFile) {
    saving[cf.path] = true;
    error[cf.path] = null;
    saved[cf.path] = false;
    try {
      let content = raw[cf.path] ?? "";
      if (usesFields(cf)) {
        const fv = fields[cf.path] ?? {};
        for (const f of cf.fields!) {
          content = setField(content, cf.format, f.key, fv[f.key] ?? "", f.section);
        }
      }
      await writeServerFile(server.id, cf.path, content);
      raw[cf.path] = content;
      saved[cf.path] = true;
    } catch (e) {
      error[cf.path] = `Failed to save: ${String(e)}`;
    } finally {
      saving[cf.path] = false;
    }
  }

  $effect(() => {
    for (const cf of configFiles) load(cf);
  });
</script>

<div class="space-y-4">
  {#each configFiles as cf}
    <div class="bg-cubelit-surface border border-cubelit-border rounded-2xl p-5">
      <div class="flex justify-between items-start gap-4 mb-4">
        <div class="min-w-0">
          <div class="text-sm font-semibold text-cubelit-text">{cf.label}</div>
          <div class="text-xs text-cubelit-text-dim font-mono truncate">{cf.path}</div>
        </div>
        <Button onclick={() => save(cf)} loading={saving[cf.path]} disabled={loading[cf.path]}>
          Save config
        </Button>
      </div>

      {#if error[cf.path]}
        <p class="text-xs text-cubelit-warning px-3 py-2 bg-cubelit-warning/5 border border-cubelit-warning/30 rounded-lg mb-3">{error[cf.path]}</p>
      {:else if saved[cf.path]}
        <p class="text-xs text-cubelit-accent px-3 py-2 bg-cubelit-accent-soft border border-cubelit-accent/30 rounded-lg mb-3">Saved. Restart the server for changes to take effect.</p>
      {/if}

      {#if loading[cf.path]}
        <p class="text-cubelit-muted text-sm py-6 text-center">Loading…</p>
      {:else if usesFields(cf)}
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          {#each cf.fields! as f}
            <label class="flex flex-col gap-1.5">
              <span class="text-xs font-medium text-cubelit-text-dim">{f.label}</span>
              {#if f.type === "boolean"}
                <button
                  type="button"
                  aria-label={`Toggle ${f.label}`}
                  class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {fields[cf.path]?.[f.key]?.toLowerCase() === 'true' ? 'bg-cubelit-accent' : 'bg-cubelit-border'}"
                  onclick={() => { const fv = fields[cf.path] ?? {}; fv[f.key] = fv[f.key]?.toLowerCase() === 'true' ? 'False' : 'True'; fields[cf.path] = { ...fv }; saved[cf.path] = false; }}
                >
                  <span class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {fields[cf.path]?.[f.key]?.toLowerCase() === 'true' ? 'translate-x-6' : 'translate-x-1'}"></span>
                </button>
              {:else if f.type === "select"}
                <select
                  class="w-full px-3 py-2.5 rounded-lg text-sm text-cubelit-text bg-cubelit-bg-2 border border-cubelit-border focus:outline-none focus:border-cubelit-accent"
                  value={fields[cf.path]?.[f.key] ?? ''}
                  onchange={(e) => { const fv = fields[cf.path] ?? {}; fv[f.key] = (e.currentTarget as HTMLSelectElement).value; fields[cf.path] = { ...fv }; saved[cf.path] = false; }}
                >
                  {#each f.options ?? [] as opt}
                    <option value={opt} style="background-color:#23272f;color:#f5f5f6;">{opt}</option>
                  {/each}
                </select>
              {:else}
                <input
                  class="w-full px-3 py-2.5 rounded-lg text-sm text-cubelit-text bg-cubelit-bg-2 border border-cubelit-border focus:outline-none focus:border-cubelit-accent"
                  type={f.type === "number" ? "number" : "text"}
                  min={f.min ?? undefined}
                  max={f.max ?? undefined}
                  step={f.step ?? undefined}
                  value={fields[cf.path]?.[f.key] ?? ''}
                  oninput={(e) => { const fv = fields[cf.path] ?? {}; fv[f.key] = (e.currentTarget as HTMLInputElement).value; fields[cf.path] = { ...fv }; saved[cf.path] = false; }}
                />
              {/if}
            </label>
          {/each}
        </div>
      {:else}
        <textarea
          class="w-full h-72 px-3 py-2.5 rounded-lg text-xs font-mono text-cubelit-text bg-[#0d1117] border border-cubelit-border focus:outline-none focus:border-cubelit-accent resize-y"
          spellcheck="false"
          value={raw[cf.path] ?? ''}
          oninput={(e) => { raw[cf.path] = (e.currentTarget as HTMLTextAreaElement).value; saved[cf.path] = false; }}
        ></textarea>
      {/if}
    </div>
  {/each}
</div>
