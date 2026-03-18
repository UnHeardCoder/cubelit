<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import Button from "./Button.svelte";
  import { checkWslStatus, enableWsl2, setWslDefaultVersion } from "$lib/api/system";
  import type { WslStatus } from "$lib/types/docker";

  interface Props {
    error: string | null;
    checking: boolean;
    oncheck: () => void;
  }

  let { error, checking, oncheck }: Props = $props();

  const isWindows = navigator.userAgent.includes("Windows");
  const dockerUrl = isWindows
    ? "https://apps.microsoft.com/detail/xp8cbj40xlbwkx"
    : "https://www.docker.com/products/docker-desktop/";

  type Step =
    | "checking"
    | "enable_wsl2"
    | "enabling_wsl2"
    | "reboot_required"
    | "install_docker"
    | "start_docker";

  let step = $state<Step>("checking");
  let wslStatus = $state<WslStatus | null>(null);
  let actionError = $state<string | null>(null);
  let actionLoading = $state(false);
  let pollInterval: ReturnType<typeof setInterval> | null = null;

  onMount(async () => {
    await runChecks();
  });

  onDestroy(() => {
    if (pollInterval !== null) clearInterval(pollInterval);
  });

  async function runChecks() {
    step = "checking";
    actionError = null;

    if (isWindows) {
      try {
        wslStatus = await checkWslStatus();
        if (wslStatus.reboot_required) {
          step = "reboot_required";
          return;
        }
        if (!wslStatus.wsl_installed || !wslStatus.wsl2_enabled) {
          // If WSL is installed but not version 2, set default version first
          if (wslStatus.wsl_installed && !wslStatus.wsl2_enabled) {
            try {
              await setWslDefaultVersion();
              wslStatus = await checkWslStatus();
              if (wslStatus.wsl2_enabled) {
                // Fall through to Docker check
              } else {
                step = "enable_wsl2";
                return;
              }
            } catch {
              step = "enable_wsl2";
              return;
            }
          } else {
            step = "enable_wsl2";
            return;
          }
        }
      } catch {
        step = "enable_wsl2";
        return;
      }
    }

    // WSL2 ok (or non-Windows) — determine Docker state from error prop
    if (!error) {
      // Should not be shown, but guard anyway
      step = "install_docker";
    } else if (
      error.toLowerCase().includes("not found") ||
      error.toLowerCase().includes("no such file") ||
      error.toLowerCase().includes("cannot connect") ||
      error.toLowerCase().includes("connection refused")
    ) {
      step = "install_docker";
    } else {
      // Docker is installed but not responding / not running
      step = "start_docker";
    }
  }

  async function handleEnableWsl2() {
    actionLoading = true;
    actionError = null;
    try {
      await enableWsl2();
      step = "enabling_wsl2";
      startPolling();
    } catch (e) {
      actionError = String(e);
    } finally {
      actionLoading = false;
    }
  }

  function startPolling() {
    if (pollInterval !== null) clearInterval(pollInterval);
    let attempts = 0;
    const MAX_ATTEMPTS = 24; // 24 × 5s = 2 minutes
    pollInterval = setInterval(async () => {
      attempts++;
      try {
        const status = await checkWslStatus();
        wslStatus = status;
        if (status.reboot_required) {
          clearInterval(pollInterval!);
          pollInterval = null;
          step = "reboot_required";
        } else if (status.wsl2_enabled) {
          clearInterval(pollInterval!);
          pollInterval = null;
          oncheck();
        }
      } catch { /* keep polling */ }

      if (attempts >= MAX_ATTEMPTS) {
        clearInterval(pollInterval!);
        pollInterval = null;
        step = "enable_wsl2";
        actionError =
          "Setup is taking longer than expected. Please click Check Again after the PowerShell window closes.";
      }
    }, 5000);
  }

  function handleCheckAgain() {
    if (pollInterval !== null) {
      clearInterval(pollInterval);
      pollInterval = null;
    }
    runChecks();
  }
</script>

<div class="min-h-screen bg-cubelit-bg flex items-center justify-center p-8">
  <div class="max-w-md text-center">

    {#if step === "checking"}
      <div class="mb-6">
        <svg class="w-20 h-20 mx-auto text-cubelit-muted animate-spin" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
        </svg>
      </div>
      <h2 class="text-2xl font-bold text-cubelit-text mb-3">Checking your setup...</h2>

    {:else if step === "enable_wsl2"}
      <div class="mb-6">
        <svg class="w-20 h-20 mx-auto text-cubelit-warning" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126zM12 15.75h.007v.008H12v-.008z" />
        </svg>
      </div>
      <h2 class="text-2xl font-bold text-cubelit-text mb-3">WSL2 Required</h2>
      <p class="text-cubelit-muted mb-2">
        Docker Desktop on Windows requires WSL2 (Windows Subsystem for Linux).
      </p>
      <p class="text-cubelit-muted mb-6">
        Click below to enable it automatically. You'll see a UAC prompt — click <strong class="text-cubelit-text">Yes</strong> to allow the change.
      </p>
      {#if actionError}
        <p class="text-sm text-cubelit-error/80 mb-4 bg-cubelit-error/10 rounded-lg p-3">{actionError}</p>
      {/if}
      <div class="flex flex-col gap-3">
        <Button onclick={handleEnableWsl2} loading={actionLoading}>
          Enable WSL2
        </Button>
        <button
          onclick={handleCheckAgain}
          class="text-sm text-cubelit-accent hover:text-cubelit-accent-hover transition-colors"
        >
          Check Again
        </button>
      </div>

    {:else if step === "enabling_wsl2"}
      <div class="mb-6">
        <svg class="w-20 h-20 mx-auto text-cubelit-accent animate-spin" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
        </svg>
      </div>
      <h2 class="text-2xl font-bold text-cubelit-text mb-3">Enabling WSL2...</h2>
      <p class="text-cubelit-muted mb-2">
        A PowerShell window may have opened. Please accept the UAC prompt if shown.
      </p>
      <p class="text-cubelit-muted mb-6 text-sm">
        Checking automatically every 5 seconds...
      </p>
      <div class="flex flex-col gap-3">
        <button
          onclick={handleCheckAgain}
          class="text-sm text-cubelit-accent hover:text-cubelit-accent-hover transition-colors"
        >
          Check Again
        </button>
      </div>

    {:else if step === "reboot_required"}
      <div class="mb-6">
        <svg class="w-20 h-20 mx-auto text-cubelit-warning" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182m0-4.991v4.99" />
        </svg>
      </div>
      <h2 class="text-2xl font-bold text-cubelit-text mb-3">Restart Required</h2>
      <p class="text-cubelit-muted mb-2">
        WSL2 features have been enabled successfully.
      </p>
      <p class="text-cubelit-muted mb-6">
        Your computer needs to restart to finish the setup. After restarting, open Cubelit again to continue.
      </p>
      <div class="flex flex-col gap-3">
        <p class="text-sm text-cubelit-muted bg-cubelit-surface rounded-lg p-3">
          Restart from the <strong class="text-cubelit-text">Start menu → Power → Restart</strong>
        </p>
        <button
          onclick={handleCheckAgain}
          class="text-sm text-cubelit-accent hover:text-cubelit-accent-hover transition-colors"
        >
          Check Again
        </button>
      </div>

    {:else if step === "install_docker"}
      <div class="mb-6">
        <svg class="w-20 h-20 mx-auto text-cubelit-muted" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" d="M5.25 14.25h13.5m-13.5 0a3 3 0 01-3-3m3 3a3 3 0 100 6h13.5a3 3 0 100-6m-16.5-3a3 3 0 013-3h13.5a3 3 0 013 3m-19.5 0a4.5 4.5 0 01.9-2.7L5.737 5.1a3.375 3.375 0 012.7-1.35h7.126c1.062 0 2.062.5 2.7 1.35l2.587 3.45a4.5 4.5 0 01.9 2.7m0 0a3 3 0 01-3 3m0 3h.008v.008h-.008v-.008zm0-6h.008v.008h-.008v-.008zm-3 6h.008v.008h-.008v-.008zm0-6h.008v.008h-.008v-.008z" />
        </svg>
      </div>
      <h2 class="text-2xl font-bold text-cubelit-text mb-3">Docker Not Found</h2>
      <p class="text-cubelit-muted mb-6">
        {isWindows
          ? "WSL2 is ready. Now install Docker Desktop to continue."
          : "Cubelit needs Docker to create and manage game servers. Please install Docker Desktop and make sure it's running."}
      </p>
      <div class="flex flex-col gap-3">
        <Button onclick={oncheck} loading={checking}>
          Check Again
        </Button>
        <a
          href={dockerUrl}
          target="_blank"
          rel="noopener noreferrer"
          class="text-sm text-cubelit-accent hover:text-cubelit-accent-hover transition-colors"
        >
          {isWindows ? "Get Docker Desktop from Microsoft Store" : "Download Docker Desktop"}
        </a>
      </div>

    {:else if step === "start_docker"}
      <div class="mb-6">
        <svg class="w-20 h-20 mx-auto text-cubelit-muted" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" d="M5.25 14.25h13.5m-13.5 0a3 3 0 01-3-3m3 3a3 3 0 100 6h13.5a3 3 0 100-6m-16.5-3a3 3 0 013-3h13.5a3 3 0 013 3m-19.5 0a4.5 4.5 0 01.9-2.7L5.737 5.1a3.375 3.375 0 012.7-1.35h7.126c1.062 0 2.062.5 2.7 1.35l2.587 3.45a4.5 4.5 0 01.9 2.7m0 0a3 3 0 01-3 3m0 3h.008v.008h-.008v-.008zm0-6h.008v.008h-.008v-.008zm-3 6h.008v.008h-.008v-.008zm0-6h.008v.008h-.008v-.008z" />
        </svg>
      </div>
      <h2 class="text-2xl font-bold text-cubelit-text mb-3">Docker Not Running</h2>
      <p class="text-cubelit-muted mb-2">
        Docker Desktop is installed but not running.
      </p>
      {#if error}
        <p class="text-sm text-cubelit-error/80 mb-4 bg-cubelit-error/10 rounded-lg p-3">{error}</p>
      {:else}
        <p class="text-cubelit-muted mb-6">Please open Docker Desktop and wait for it to finish starting.</p>
      {/if}
      <div class="flex flex-col gap-3">
        <Button onclick={oncheck} loading={checking}>
          Check Again
        </Button>
      </div>
    {/if}

  </div>
</div>
