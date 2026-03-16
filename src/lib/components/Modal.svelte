<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    open: boolean;
    onclose: () => void;
    title?: string;
    children: Snippet;
  }

  let { open = $bindable(false), onclose, title = "", children }: Props = $props();

  function handleBackdrop(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onclose();
    }
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
    onclick={handleBackdrop}
  >
    <div class="bg-cubelit-surface border border-cubelit-border rounded-xl max-w-lg w-full mx-4 p-6">
      {#if title}
        <h2 class="text-lg font-semibold text-cubelit-text mb-4">{title}</h2>
      {/if}
      {@render children()}
    </div>
  </div>
{/if}
