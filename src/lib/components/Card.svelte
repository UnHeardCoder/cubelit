<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
    class?: string;
    onclick?: (e: MouseEvent) => void;
    hoverable?: boolean;
  }

  let {
    children,
    class: className = "",
    onclick,
    hoverable = false,
  }: Props = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="bg-cubelit-surface border border-cubelit-border rounded-xl {hoverable ? 'hover:border-cubelit-accent/50 cursor-pointer transition-colors' : ''} {className}"
  {onclick}
>
  {@render children()}
</div>
