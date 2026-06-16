<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    open: boolean;
    onclose: () => void;
    title?: string;
    children: Snippet;
  }

  let { open = $bindable(false), onclose, title = '', children }: Props = $props();

  function handleBackdrop(e: MouseEvent) {
    if (e.target === e.currentTarget) onclose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onclose();
  }
</script>

<svelte:window onkeydown={open ? handleKeydown : undefined} />

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="cubelit-modal-root fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
    onclick={handleBackdrop}
  >
    <div class="bg-cubelit-surface border border-cubelit-border-2 rounded-2xl max-w-lg w-full mx-4 p-6 shadow-2xl">
      {#if title}
        <h2 class="text-sm font-semibold text-cubelit-text mb-4 tracking-tight">{title}</h2>
      {/if}
      {@render children()}
    </div>
  </div>
{/if}
