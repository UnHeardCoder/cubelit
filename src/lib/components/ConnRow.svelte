<script lang="ts">
  interface Props {
    label: string;
    value: string;
  }
  let { label, value }: Props = $props();

  let copied = $state(false);

  async function copy() {
    try {
      await navigator.clipboard.writeText(value);
      copied = true;
      setTimeout(() => { copied = false; }, 1400);
    } catch { /* ignore */ }
  }
</script>

<div class="flex items-center gap-2.5 px-3 py-2 bg-cubelit-bg-2 rounded-lg border border-cubelit-border">
  <span class="text-[11px] text-cubelit-muted w-12 shrink-0">{label}</span>
  <span class="font-mono text-sm text-cubelit-text flex-1 truncate">{value}</span>
  <button
    type="button"
    onclick={copy}
    class="shrink-0 text-cubelit-muted hover:text-cubelit-text transition-colors p-1"
    title="Copy"
  >
    {#if copied}
      <!-- Check icon -->
      <svg class="w-3.5 h-3.5 text-cubelit-success" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
        <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5" />
      </svg>
    {:else}
      <!-- Copy icon -->
      <svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
        <rect x="9" y="9" width="11" height="11" rx="2" stroke="currentColor" />
        <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" stroke="currentColor" />
      </svg>
    {/if}
  </button>
</div>
