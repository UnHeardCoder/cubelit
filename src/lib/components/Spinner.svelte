<script lang="ts">
  interface Props {
    size?: 'sm' | 'md' | 'lg';
    class?: string;
  }
  let { size = 'md', class: className = '' }: Props = $props();

  const px  = $derived(({ sm: 28, md: 42, lg: 56 } as const)[size]);
  const gap = $derived(({ sm: 2,  md: 3,  lg: 4  } as const)[size]);
</script>

<div
  class="spin-voxel-stack {className}"
  style="width: {px}px; height: {px}px; gap: {gap}px;"
  aria-label="Loading"
>
  {#each Array(9) as _}
    <i></i>
  {/each}
</div>
