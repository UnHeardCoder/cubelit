<script lang="ts">
  import type { Snippet } from "svelte";
  import { AnsiUp } from "ansi_up";

  interface Props {
    lines: string[];
    loading: boolean;
    onRefresh: () => void;
    helpContent?: Snippet;
  }

  let { lines, loading, onRefresh, helpContent }: Props = $props();

  const ansiUp = new AnsiUp();
  ansiUp.use_classes = false;

  let follow = $state(true);
  let scrollContainer: HTMLDivElement | undefined = $state();

  function toHtml(line: string): string {
    return ansiUp.ansi_to_html(line);
  }

  // Auto-scroll to bottom when new lines arrive and follow is enabled
  $effect(() => {
    // Track lines.length to re-run on new lines
    lines.length;
    if (follow && scrollContainer) {
      // Use tick to wait for DOM update
      requestAnimationFrame(() => {
        if (scrollContainer) {
          scrollContainer.scrollTop = scrollContainer.scrollHeight;
        }
      });
    }
  });

  function handleScroll() {
    if (!scrollContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
    // If user scrolled within 40px of the bottom, re-enable follow
    const atBottom = scrollHeight - scrollTop - clientHeight < 40;
    follow = atBottom;
  }
</script>

<div class="space-y-3">
  <div class="flex items-center justify-between">
    <h3 class="text-sm font-medium text-cubelit-text">Container Logs</h3>
    <div class="flex items-center gap-3">
      <button
        class="text-xs transition-colors {follow ? 'text-cubelit-accent' : 'text-cubelit-muted hover:text-cubelit-text'}"
        onclick={() => {
          follow = !follow;
          if (follow && scrollContainer) {
            scrollContainer.scrollTop = scrollContainer.scrollHeight;
          }
        }}
      >
        {follow ? "Following" : "Follow"}
      </button>
      <button
        class="text-xs text-cubelit-muted hover:text-cubelit-text transition-colors"
        onclick={onRefresh}
      >
        Refresh
      </button>
    </div>
  </div>

  {#if helpContent}
    {@render helpContent()}
  {/if}

  {#if loading && lines.length === 0}
    <p class="text-cubelit-muted text-sm py-8 text-center">Loading logs...</p>
  {:else if lines.length === 0}
    <div class="text-center py-12 bg-cubelit-surface border border-cubelit-border rounded-xl">
      <p class="text-cubelit-muted text-sm">No log output available</p>
    </div>
  {:else}
    <div
      bind:this={scrollContainer}
      onscroll={handleScroll}
      class="bg-[#0d1117] border border-cubelit-border rounded-xl p-4 overflow-auto max-h-[480px] font-mono text-xs leading-relaxed space-y-0.5"
    >
      {#each lines as line}
        <div class="text-gray-300 whitespace-pre-wrap break-all">{@html toHtml(line)}</div>
      {/each}
    </div>
  {/if}
</div>
