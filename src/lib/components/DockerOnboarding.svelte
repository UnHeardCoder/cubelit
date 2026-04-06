<script lang="ts">
  import { onDestroy } from "svelte";
  import Button from "./Button.svelte";
  import { getOnboardingStatus, enableWsl2, setWslDefaultVersion } from "$lib/api/system";
  import type { OnboardingStatus } from "$lib/types/docker";

  interface Props {
    status: OnboardingStatus | null;
    checking: boolean;
    oncheck: () => void;
  }

  let { status = null, checking, oncheck }: Props = $props();

  type Step =
    | "checking"
    | "enable_wsl2"
    | "set_default_wsl2"
    | "enabling_wsl2"
    | "setting_default_wsl2"
    | "reboot_required"
    | "install_docker"
    | "start_docker"
    | "diagnostic_error";

  // Keep a local copy so polling can replace the prop value between parent refreshes.
  let currentStatus = $state<OnboardingStatus | null>(null);
  let actionError = $state<string | null>(null);
  let actionLoading = $state(false);
  let actionStep = $state<"enabling_wsl2" | "setting_default_wsl2" | null>(null);
  let pollInterval: ReturnType<typeof setInterval> | null = null;
  let mounted = true;

  const windowsStoreUrl = "https://apps.microsoft.com/detail/xp8cbj40xlbwkx";
  const dockerDesktopUrl = "https://www.docker.com/products/docker-desktop/";

  $effect(() => {
    currentStatus = status;
  });

  function isWindows(): boolean {
    return currentStatus?.platform === "windows";
  }

  function statusStep(): Step {
    if (!currentStatus) return "checking";

    if (currentStatus.platform === "windows") {
      switch (currentStatus.wsl.state) {
        case "reboot_required":
          return "reboot_required";
        case "needs_install":
          return "enable_wsl2";
        case "needs_default_v2":
          return "set_default_wsl2";
        case "check_failed":
          return "diagnostic_error";
      }
    }

    switch (currentStatus.docker.state) {
      case "ready":
        return "checking";
      case "not_installed":
        return "install_docker";
      case "not_running":
        return "start_docker";
      case "permission_denied":
      case "unknown":
        return "diagnostic_error";
    }
  }

  function currentStep(): Step {
    return actionStep ?? statusStep();
  }

  function diagnosticsMessage(): string {
    if (!currentStatus) return "Cubelit couldn't verify your Docker setup yet.";
    const wslError = currentStatus.wsl.error?.trim();
    if (currentStatus.platform === "windows" && currentStatus.wsl.state === "check_failed") {
      return wslError || "Cubelit couldn't verify your WSL setup.";
    }
    return currentStatus.docker.error?.trim() || wslError || "Cubelit couldn't verify your Docker setup.";
  }

  async function refreshStatus() {
    if (!mounted) return;
    const nextStatus = await getOnboardingStatus();
    if (!mounted) return;
    currentStatus = nextStatus;
  }

  function stopPolling() {
    if (pollInterval !== null) clearInterval(pollInterval);
    pollInterval = null;
  }

  onDestroy(() => {
    mounted = false;
    stopPolling();
  });

  function startPolling(waitForStep: "enable_wsl2" | "set_default_wsl2") {
    stopPolling();
    let attempts = 0;
    const maxAttempts = 24;

    pollInterval = setInterval(async () => {
      if (!mounted || pollInterval === null) return;
      attempts++;
      try {
        await refreshStatus();
        if (!mounted || pollInterval === null) return;
        const step = statusStep();
        if (step !== waitForStep) {
          stopPolling();
          if (!mounted) return;
          actionStep = null;
          oncheck();
          return;
        }
      } catch (e) {
        if (!mounted) return;
        actionError = String(e);
      }

      if (attempts >= maxAttempts) {
        stopPolling();
        if (!mounted) return;
        actionStep = null;
        actionError = "Setup is taking longer than expected. Click Check Again after the command window closes.";
        oncheck();
      }
    }, 5000);
  }

  async function handleEnableWsl2() {
    actionLoading = true;
    actionError = null;
    try {
      await enableWsl2();
      if (!mounted) return;
      actionStep = "enabling_wsl2";
      startPolling("enable_wsl2");
    } catch (e) {
      if (!mounted) return;
      actionError = String(e);
    } finally {
      if (!mounted) return;
      actionLoading = false;
    }
  }

  async function handleSetDefaultWsl2() {
    actionLoading = true;
    actionError = null;
    try {
      await setWslDefaultVersion();
      if (!mounted) return;
      actionStep = "setting_default_wsl2";
      await refreshStatus();
      if (!mounted) return;
      if (statusStep() === "set_default_wsl2") {
        startPolling("set_default_wsl2");
      } else {
        actionStep = null;
        oncheck();
      }
    } catch (e) {
      if (!mounted) return;
      actionError = String(e);
      actionStep = null;
    } finally {
      if (!mounted) return;
      actionLoading = false;
    }
  }

  function handleCheckAgain() {
    stopPolling();
    if (!mounted) return;
    actionStep = null;
    actionError = null;
    oncheck();
  }
</script>

<div class="min-h-screen bg-cubelit-bg flex items-center justify-center p-8">
  <div class="max-w-md text-center">
    {#if currentStep() === "checking"}
      <div class="mb-6">
        <svg class="w-20 h-20 mx-auto text-cubelit-muted animate-spin" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
        </svg>
      </div>
      <h2 class="text-2xl font-bold text-cubelit-text mb-3">Checking your setup...</h2>

    {:else if currentStep() === "enable_wsl2"}
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
        Click below to enable it automatically. You may see a UAC prompt. Accept it to continue.
      </p>
      {#if actionError}
        <p class="text-sm text-cubelit-error/80 mb-4 bg-cubelit-error/10 rounded-lg p-3">{actionError}</p>
      {/if}
      <div class="flex flex-col gap-3">
        <Button onclick={handleEnableWsl2} loading={actionLoading}>
          Enable WSL2
        </Button>
        <button
          type="button"
          onclick={handleCheckAgain}
          class="text-sm text-cubelit-accent hover:text-cubelit-accent-hover transition-colors"
        >
          Check Again
        </button>
      </div>

    {:else if currentStep() === "set_default_wsl2"}
      <div class="mb-6">
        <svg class="w-20 h-20 mx-auto text-cubelit-warning" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" d="M11.25 4.5v5.25m0 0 3-3m-3 3-3-3M5.25 19.5h13.5a1.5 1.5 0 0 0 1.5-1.5V9.75a1.5 1.5 0 0 0-1.5-1.5h-13.5a1.5 1.5 0 0 0-1.5 1.5V18a1.5 1.5 0 0 0 1.5 1.5Z" />
        </svg>
      </div>
      <h2 class="text-2xl font-bold text-cubelit-text mb-3">Set WSL Default to Version 2</h2>
      <p class="text-cubelit-muted mb-6">
        WSL is installed, but version 2 is not configured as the default. Cubelit can set that for you now.
      </p>
      {#if actionError}
        <p class="text-sm text-cubelit-error/80 mb-4 bg-cubelit-error/10 rounded-lg p-3">{actionError}</p>
      {/if}
      <div class="flex flex-col gap-3">
        <Button onclick={handleSetDefaultWsl2} loading={actionLoading}>
          Set Default Version to 2
        </Button>
        <button
          type="button"
          onclick={handleCheckAgain}
          class="text-sm text-cubelit-accent hover:text-cubelit-accent-hover transition-colors"
        >
          Check Again
        </button>
      </div>

    {:else if currentStep() === "enabling_wsl2" || currentStep() === "setting_default_wsl2"}
      <div class="mb-6">
        <svg class="w-20 h-20 mx-auto text-cubelit-accent animate-spin" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="1.5"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path>
        </svg>
      </div>
      <h2 class="text-2xl font-bold text-cubelit-text mb-3">
        {currentStep() === "enabling_wsl2" ? "Enabling WSL2..." : "Updating WSL default version..."}
      </h2>
      <p class="text-cubelit-muted mb-2">
        A system command window may have opened. Accept any Windows prompt if shown.
      </p>
      <p class="text-cubelit-muted mb-6 text-sm">
        Checking automatically every 5 seconds...
      </p>
      {#if actionError}
        <p class="text-sm text-cubelit-error/80 mb-4 bg-cubelit-error/10 rounded-lg p-3">{actionError}</p>
      {/if}
      <button
        type="button"
        onclick={handleCheckAgain}
        class="text-sm text-cubelit-accent hover:text-cubelit-accent-hover transition-colors"
      >
        Check Again
      </button>

    {:else if currentStep() === "reboot_required"}
      <div class="mb-6">
        <svg class="w-20 h-20 mx-auto text-cubelit-warning" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0l3.181 3.183a8.25 8.25 0 0013.803-3.7M4.031 9.865a8.25 8.25 0 0113.803-3.7l3.181 3.182m0-4.991v4.99" />
        </svg>
      </div>
      <h2 class="text-2xl font-bold text-cubelit-text mb-3">Restart Required</h2>
      <p class="text-cubelit-muted mb-2">
        WSL-related Windows features are enabled, but your computer needs to restart before Docker can use them.
      </p>
      <p class="text-cubelit-muted mb-6">
        Restart your PC, then open Cubelit again.
      </p>
      <div class="flex flex-col gap-3">
        <p class="text-sm text-cubelit-muted bg-cubelit-surface rounded-lg p-3">
          Restart from <strong class="text-cubelit-text">Start menu → Power → Restart</strong>
        </p>
        <button
          type="button"
          onclick={handleCheckAgain}
          class="text-sm text-cubelit-accent hover:text-cubelit-accent-hover transition-colors"
        >
          Check Again
        </button>
      </div>

    {:else if currentStep() === "install_docker"}
      <div class="mb-6">
        <svg class="w-20 h-20 mx-auto text-cubelit-muted" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" d="M5.25 14.25h13.5m-13.5 0a3 3 0 01-3-3m3 3a3 3 0 100 6h13.5a3 3 0 100-6m-16.5-3a3 3 0 013-3h13.5a3 3 0 013 3m-19.5 0a4.5 4.5 0 01.9-2.7L5.737 5.1a3.375 3.375 0 012.7-1.35h7.126c1.062 0 2.062.5 2.7 1.35l2.587 3.45a4.5 4.5 0 01.9 2.7m0 0a3 3 0 01-3 3m0 3h.008v.008h-.008v-.008zm0-6h.008v.008h-.008v-.008zm-3 6h.008v.008h-.008v-.008zm0-6h.008v.008h-.008v-.008z" />
        </svg>
      </div>
      <h2 class="text-2xl font-bold text-cubelit-text mb-3">Docker Not Found</h2>
      <p class="text-cubelit-muted mb-6">
        {isWindows()
          ? "WSL2 is ready. Install Docker Desktop, then return here."
          : "Cubelit needs Docker to create and manage game servers. Install Docker Desktop or Docker Engine, then return here."}
      </p>
      <div class="flex flex-col gap-3">
        <Button onclick={oncheck} loading={checking}>
          Check Again
        </Button>
        <a
          href={isWindows() ? windowsStoreUrl : dockerDesktopUrl}
          target="_blank"
          rel="noopener noreferrer"
          class="text-sm text-cubelit-accent hover:text-cubelit-accent-hover transition-colors"
        >
          {isWindows() ? "Get Docker Desktop from Microsoft Store" : "Download Docker Desktop"}
        </a>
      </div>

    {:else if currentStep() === "start_docker"}
      <div class="mb-6">
        <svg class="w-20 h-20 mx-auto text-cubelit-muted" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" d="M5.25 14.25h13.5m-13.5 0a3 3 0 01-3-3m3 3a3 3 0 100 6h13.5a3 3 0 100-6m-16.5-3a3 3 0 013-3h13.5a3 3 0 013 3m-19.5 0a4.5 4.5 0 01.9-2.7L5.737 5.1a3.375 3.375 0 012.7-1.35h7.126c1.062 0 2.062.5 2.7 1.35l2.587 3.45a4.5 4.5 0 01.9 2.7m0 0a3 3 0 01-3 3m0 3h.008v.008h-.008v-.008zm0-6h.008v.008h-.008v-.008zm-3 6h.008v.008h-.008v-.008zm0-6h.008v.008h-.008v-.008z" />
        </svg>
      </div>
      <h2 class="text-2xl font-bold text-cubelit-text mb-3">Docker Not Running</h2>
      <p class="text-cubelit-muted mb-2">
        Docker is installed, but Cubelit can't connect to it yet.
      </p>
      <p class="text-cubelit-muted mb-6">
        Open Docker Desktop and wait for it to finish starting, then check again.
      </p>
      {#if currentStatus?.docker.error}
        <p class="text-sm text-cubelit-error/80 mb-4 bg-cubelit-error/10 rounded-lg p-3">{currentStatus.docker.error}</p>
      {/if}
      <div class="flex flex-col gap-3">
        <Button onclick={oncheck} loading={checking}>
          Check Again
        </Button>
      </div>

    {:else if currentStep() === "diagnostic_error"}
      <div class="mb-6">
        <svg class="w-20 h-20 mx-auto text-cubelit-warning" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m0 3.75h.008v.008H12v-.008zm8.25-3.758c0 4.556-3.694 8.25-8.25 8.25S3.75 16.553 3.75 11.997 7.444 3.747 12 3.747s8.25 3.694 8.25 8.25Z" />
        </svg>
      </div>
      <h2 class="text-2xl font-bold text-cubelit-text mb-3">Setup Check Failed</h2>
      <p class="text-cubelit-muted mb-6">
        Cubelit couldn't determine exactly what's wrong with your Docker or WSL setup. The diagnostic output is below.
      </p>
      <p class="text-sm text-cubelit-error/80 mb-4 bg-cubelit-error/10 rounded-lg p-3 text-left whitespace-pre-wrap break-words">{diagnosticsMessage()}</p>
      <div class="flex flex-col gap-3">
        <Button onclick={oncheck} loading={checking}>
          Check Again
        </Button>
      </div>
    {/if}
  </div>
</div>
