<script lang="ts">
  interface Props {
    id?: string;
    value?: string;
    placeholder?: string;
    label?: string;
    type?: string;
    disabled?: boolean;
    mono?: boolean;
    oninput?: (e: Event) => void;
    class?: string;
  }

  let {
    id = '',
    value = $bindable(''),
    placeholder = '',
    label = '',
    type = 'text',
    disabled = false,
    mono = false,
    oninput,
    class: className = '',
  }: Props = $props();

  const uid = $derived(id || `input-${Math.random().toString(36).slice(2, 9)}`);
</script>

<div class="flex flex-col gap-1.5 {className}">
  {#if label}
    <label class="text-[11px] text-cubelit-text-dim {mono ? 'font-mono' : ''}" for={uid}>
      {label}
    </label>
  {/if}
  <input
    id={uid}
    {type}
    bind:value
    {placeholder}
    {disabled}
    {oninput}
    class="
      w-full px-3 py-2.5 rounded-lg text-sm text-cubelit-text placeholder:text-cubelit-muted
      bg-cubelit-bg-2 border border-cubelit-border
      focus:outline-none focus:border-cubelit-accent transition-colors
      disabled:opacity-50 disabled:cursor-not-allowed
      {mono ? 'font-mono' : ''}
    "
  />
</div>
