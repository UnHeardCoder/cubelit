<script lang="ts">
  import type { Recipe } from "$lib/types/recipe";
  import Input from "./Input.svelte";
  import PortInput from "./PortInput.svelte";

  interface Props {
    recipe: Recipe;
    serverName: string;
    envValues: Record<string, string>;
    portValues: Record<string, number>;
    onenvchange: (key: string, value: string) => void;
    onportchange: (containerPort: string, hostPort: number) => void;
    onname: (name: string) => void;
  }

  let {
    recipe,
    serverName = $bindable(""),
    envValues,
    portValues,
    onenvchange,
    onportchange,
    onname,
  }: Props = $props();

  function handleEnvInput(key: string, e: Event) {
    const target = e.target as HTMLInputElement | HTMLSelectElement;
    onenvchange(key, target.value);
  }
</script>

<div class="space-y-6">
  <h2 class="text-lg font-semibold text-cubelit-text">Configure {recipe.name}</h2>

  <Input
    label="Server Name"
    bind:value={serverName}
    placeholder="My {recipe.name} Server"
    oninput={() => onname(serverName)}
  />

  <!-- Ports -->
  {#if recipe.ports.length > 0}
    <div>
      <h3 class="text-sm font-medium text-cubelit-muted mb-3">Ports</h3>
      <div class="space-y-3">
        {#each recipe.ports as port}
          <PortInput
            label="{port.label} ({port.protocol.toUpperCase()})"
            containerPort={port.container_port}
            value={portValues[`${port.container_port}/${port.protocol}`] ?? port.default_host_port}
            onchange={(v) => onportchange(`${port.container_port}/${port.protocol}`, v)}
          />
        {/each}
      </div>
    </div>
  {/if}

  <!-- Environment variables -->
  {#if recipe.environment.length > 0}
    <div>
      <h3 class="text-sm font-medium text-cubelit-muted mb-3">Settings</h3>
      <div class="space-y-3">
        {#each recipe.environment as env}
          {#if env.type === "select"}
            <div class="flex flex-col gap-1.5">
              <label class="text-sm text-cubelit-muted">{env.label}</label>
              <select
                class="w-full px-3 py-2 bg-cubelit-bg border border-cubelit-border rounded-lg text-cubelit-text focus:outline-none focus:border-cubelit-accent transition-colors"
                value={envValues[env.key] ?? env.default_value}
                onchange={(e) => handleEnvInput(env.key, e)}
              >
                {#each env.options as option}
                  <option value={option}>{option}</option>
                {/each}
              </select>
            </div>
          {:else if env.type === "boolean"}
            <div class="flex items-center justify-between">
              <label class="text-sm text-cubelit-muted">{env.label}</label>
              <button
                class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {(envValues[env.key] ?? env.default_value).toLowerCase() === 'true' ? 'bg-cubelit-accent' : 'bg-cubelit-border'}"
                onclick={() => {
                  const current = (envValues[env.key] ?? env.default_value).toLowerCase();
                  onenvchange(env.key, current === "true" ? "FALSE" : "TRUE");
                }}
              >
                <span
                  class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {(envValues[env.key] ?? env.default_value).toLowerCase() === 'true' ? 'translate-x-6' : 'translate-x-1'}"
                />
              </button>
            </div>
          {:else if env.type === "ram"}
            <div class="flex flex-col gap-1.5">
              <label class="text-sm text-cubelit-muted">{env.label}</label>
              <select
                class="w-full px-3 py-2 bg-cubelit-bg border border-cubelit-border rounded-lg text-cubelit-text focus:outline-none focus:border-cubelit-accent transition-colors"
                value={envValues[env.key] ?? env.default_value}
                onchange={(e) => handleEnvInput(env.key, e)}
              >
                {#each env.options as option}
                  <option value={option}>{option}</option>
                {/each}
              </select>
            </div>
          {:else}
            <div class="flex flex-col gap-1.5">
              <label class="text-sm text-cubelit-muted">{env.label}</label>
              <input
                type={env.type === "number" ? "number" : "text"}
                class="w-full px-3 py-2 bg-cubelit-bg border border-cubelit-border rounded-lg text-cubelit-text placeholder:text-cubelit-muted/50 focus:outline-none focus:border-cubelit-accent transition-colors"
                value={envValues[env.key] ?? env.default_value}
                oninput={(e) => handleEnvInput(env.key, e)}
              />
            </div>
          {/if}
        {/each}
      </div>
    </div>
  {/if}
</div>
