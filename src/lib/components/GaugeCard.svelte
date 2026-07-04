<script lang="ts">
  interface Props {
    label: string;
    value: string;
    sub?: string;
    bar?: number; // 0–100
    class?: string;
  }
  let { label, value, sub, bar, class: className = '' }: Props = $props();

  function barColor(): string {
    if (bar === undefined) return 'var(--c-accent)';
    if (bar > 80) return 'var(--c-error)';
    if (bar > 50) return 'var(--c-warning)';
    return 'var(--c-accent)';
  }
</script>

<div
  class="bg-cubelit-surface border border-cubelit-border rounded-xl p-4 {className}"
  style="border-radius: 12px;"
>
  <div class="text-[11px] text-cubelit-muted font-mono uppercase tracking-widest">{label}</div>
  <div class="text-xl font-semibold mt-1 tracking-tight text-cubelit-text">{value}</div>
  {#if sub}
    <div class="text-xs text-cubelit-text-dim mt-0.5">{sub}</div>
  {/if}
  {#if bar !== undefined}
    <div class="mt-2.5 h-1 bg-cubelit-bg-2 rounded-full overflow-hidden">
      <div
        class="h-full rounded-full transition-all"
        style="width: {Math.min(100, bar)}%; background: {barColor()};"
      ></div>
    </div>
  {/if}
</div>
