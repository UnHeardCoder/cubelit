<script lang="ts">
  interface Props {
    id?: string;
    value?: string;
    placeholder?: string;
    label?: string;
    type?: string;
    disabled?: boolean;
    oninput?: (e: Event) => void;
    class?: string;
  }

  let {
    id = "",
    value = $bindable(""),
    placeholder = "",
    label = "",
    type = "text",
    disabled = false,
    oninput,
    class: className = "",
  }: Props = $props();

  const fallbackId = `input-${Math.random().toString(36).slice(2, 10)}`;

  function inputId(): string {
    return id || fallbackId;
  }
</script>

<div class="flex flex-col gap-1.5 {className}">
  {#if label}
    <label class="text-sm text-cubelit-muted" for={inputId()}>{label}</label>
  {/if}
  <input
    id={inputId()}
    {type}
    bind:value
    {placeholder}
    {disabled}
    {oninput}
    class="w-full px-3 py-2 bg-cubelit-bg border border-cubelit-border rounded-lg text-cubelit-text placeholder:text-cubelit-muted/50 focus:outline-none focus:border-cubelit-accent transition-colors"
  />
</div>
