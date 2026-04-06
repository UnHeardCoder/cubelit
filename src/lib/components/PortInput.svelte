<script lang="ts">
  import { checkPort, suggestPort } from "$lib/api/system";

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
  let rangeError = $state<string | null>(null);
  let suggestion = $state<number | null>(null);
  let suggesting = $state(false);

  function inputId(): string {
    return `port-${containerPort}-${label.toLowerCase().replace(/[^a-z0-9]+/g, "-")}`;
  }

  async function doCheck(port: number) {
    checking = true;
    suggestion = null;
    try {
      available = await checkPort(port);
      if (!available) {
        suggesting = true;
        try {
          suggestion = await suggestPort(port);
        } finally {
          suggesting = false;
        }
      }
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

    if (newValue < 1 || newValue > 65535) {
      rangeError = "Port must be between 1 and 65535";
      available = null;
      suggestion = null;
      clearTimeout(checkTimeout);
      return;
    }

    rangeError = null;
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
  <label class="text-sm text-cubelit-muted" for={inputId()}>{label}</label>
  <div class="flex items-center gap-2">
    <span class="text-xs text-cubelit-muted whitespace-nowrap">{containerPort} &rarr;</span>
    <input
      id={inputId()}
      type="number"
      class="flex-1 px-3 py-2 bg-cubelit-bg border border-cubelit-border rounded-lg text-cubelit-text focus:outline-none focus:border-cubelit-accent transition-colors"
      {value}
      oninput={handleInput}
      min="1"
      max="65535"
    />
    <div class="w-5 shrink-0 flex justify-center">
      {#if checking || suggesting}
        <div class="w-3 h-3 rounded-full border-2 border-cubelit-muted border-t-transparent animate-spin"></div>
      {:else if available === true}
        <div class="w-3 h-3 rounded-full bg-cubelit-success" title="Port available"></div>
      {:else if available === false}
        <div class="w-3 h-3 rounded-full bg-cubelit-error" title="Port in use"></div>
      {/if}
    </div>
  </div>
  {#if rangeError}
    <p class="text-xs text-cubelit-error">{rangeError}</p>
  {:else if suggestion !== null}
    <p class="text-xs text-cubelit-muted">
      Port in use.
      <button
        type="button"
        class="text-cubelit-accent underline hover:opacity-80 transition-opacity"
        onclick={() => {
          if (suggestion !== null) {
            onchange(suggestion);
            suggestion = null;
          }
        }}
      >
        Use {suggestion} instead
      </button>
    </p>
  {/if}
</div>
