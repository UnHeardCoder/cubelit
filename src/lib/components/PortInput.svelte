<script lang="ts">
  import { checkPort } from "$lib/api/system";

  interface Props {
    label: string;
    containerPort: number;
    value: number;
    onchange: (value: number) => void;
  }

  let { label, containerPort, value, onchange }: Props = $props();

  let available = $state<boolean | null>(null);
  let checking = $state(false);
  let checkTimeout: ReturnType<typeof setTimeout>;

  async function doCheck(port: number) {
    checking = true;
    try {
      available = await checkPort(port);
    } catch {
      available = null;
    } finally {
      checking = false;
    }
  }

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    const newValue = parseInt(target.value);
    if (isNaN(newValue)) return;
    onchange(newValue);
    available = null;
    clearTimeout(checkTimeout);
    checkTimeout = setTimeout(() => doCheck(newValue), 500);
  }

  // Check initial value
  $effect(() => {
    doCheck(value);
  });
</script>

<div class="flex flex-col gap-1.5">
  <label class="text-sm text-cubelit-muted">{label}</label>
  <div class="flex items-center gap-2">
    <span class="text-xs text-cubelit-muted whitespace-nowrap">{containerPort} &rarr;</span>
    <input
      type="number"
      class="flex-1 px-3 py-2 bg-cubelit-bg border border-cubelit-border rounded-lg text-cubelit-text focus:outline-none focus:border-cubelit-accent transition-colors"
      {value}
      oninput={handleInput}
      min="1024"
      max="65535"
    />
    <div class="w-5 shrink-0 flex justify-center">
      {#if checking}
        <div class="w-3 h-3 rounded-full border-2 border-cubelit-muted border-t-transparent animate-spin" />
      {:else if available === true}
        <div class="w-3 h-3 rounded-full bg-cubelit-success" title="Port available" />
      {:else if available === false}
        <div class="w-3 h-3 rounded-full bg-cubelit-error" title="Port in use" />
      {/if}
    </div>
  </div>
</div>
