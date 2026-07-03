<script lang="ts">
  import type { Cubelit } from "$lib/types/server";
  import type { RecipeCommand } from "$lib/types/recipe";
  import { sendServerCommand } from "$lib/api/minecraft";
  import Button from "$lib/components/Button.svelte";

  interface Props {
    server: Cubelit;
    /** The recipe's `dashboard.command` descriptor (null → no console). */
    command?: RecipeCommand | null;
  }

  let { server, command }: Props = $props();

  const mode = $derived(command?.mode ?? "none");
  const isInteractive = $derived(mode === "source_rcon" || mode === "docker_exec");
  const quickCommands = $derived(command?.quick_commands ?? []);

  // docker_exec helpers (e.g. Bedrock `send-command`) write the command to the
  // server's stdin and return nothing — the reply lands asynchronously in the
  // log pane above, not on the exec's stdout. RCON, by contrast, replies inline.
  const responsesInLog = $derived(mode === "docker_exec");

  // Some quick commands target a player by name (e.g. "op {user}"). Surface a
  // username field only when at least one command references the {user} token.
  const needsUser = $derived(
    quickCommands.some((q) => q.command.includes("{user}")),
  );

  const userKey = $derived(`console_user_${server.recipe_id}`);
  let username = $state("");
  let userLoaded = false;
  $effect(() => {
    if (typeof window === "undefined") return;
    if (!userLoaded) {
      username = localStorage.getItem(userKey) ?? "";
      userLoaded = true;
      return;
    }
    localStorage.setItem(userKey, username);
  });

  let commandInput = $state("");
  let commandHistory = $state<string[]>([]);
  let historyIndex = $state(-1);
  let output = $state<{ cmd: string; response: string; error?: boolean; note?: boolean }[]>([]);
  let loading = $state(false);

  const canSend = $derived(
    isInteractive && (server.status === "running" || server.status === "starting"),
  );

  function resolve(cmd: string): string {
    return cmd.replaceAll("{user}", username.trim());
  }

  async function run(raw: string) {
    const cmd = resolve(raw).trim();
    if (!cmd || loading || !canSend) return;

    loading = true;
    commandHistory = [cmd, ...commandHistory.filter((c) => c !== cmd)].slice(0, 50);
    historyIndex = -1;
    commandInput = "";

    try {
      const response = await sendServerCommand(server.id, cmd);
      if (response) {
        output = [{ cmd, response }, ...output].slice(0, 100);
      } else if (responsesInLog) {
        output = [
          { cmd, response: "Sent — the server's reply appears in the Console log above.", note: true },
          ...output,
        ].slice(0, 100);
      } else {
        output = [{ cmd, response: "(no output)" }, ...output].slice(0, 100);
      }
    } catch (e) {
      output = [{ cmd, response: String(e), error: true }, ...output].slice(0, 100);
    } finally {
      loading = false;
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      run(commandInput);
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      const next = Math.min(historyIndex + 1, commandHistory.length - 1);
      historyIndex = next;
      commandInput = commandHistory[next] ?? "";
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      const next = Math.max(historyIndex - 1, -1);
      historyIndex = next;
      commandInput = next === -1 ? "" : (commandHistory[next] ?? "");
    }
  }

  function needsUserFor(cmd: string): boolean {
    return cmd.includes("{user}") && !username.trim();
  }
</script>

<div class="space-y-4">
  <div class="flex items-center justify-between">
    <h3 class="text-sm font-medium text-cubelit-text">Server Console</h3>
    {#if isInteractive && !canSend}
      <span class="text-xs text-cubelit-muted italic">Start the server to use the console</span>
    {/if}
  </div>

  {#if !isInteractive}
    <!-- External / no-console games: explain the real workflow rather than a dead box. -->
    <div class="px-4 py-3 bg-cubelit-surface border border-cubelit-border rounded-xl">
      <p class="text-sm text-cubelit-muted">
        This server doesn't expose an in-app command console. Manage it through its own
        admin workflow (web panel or in-game admin commands).
      </p>
    </div>
  {:else}
    {#if needsUser}
      <div class="flex items-center gap-3">
        <span class="text-xs text-cubelit-muted w-28 shrink-0">Your username</span>
        <input
          type="text"
          bind:value={username}
          placeholder="PlayerName"
          class="flex-1 bg-cubelit-bg border border-cubelit-border rounded-lg px-3 py-1.5 text-sm text-cubelit-text placeholder-cubelit-muted/40 focus:outline-none focus:border-cubelit-accent"
        />
      </div>
    {/if}

    {#if quickCommands.length > 0}
      <div class="flex flex-wrap gap-2">
        {#each quickCommands as q}
          <button
            type="button"
            class="px-3 py-1.5 rounded-lg text-xs font-medium transition-colors bg-cubelit-surface border border-cubelit-border text-cubelit-text hover:border-cubelit-accent hover:text-cubelit-accent disabled:opacity-40 disabled:cursor-not-allowed"
            disabled={!canSend || loading || needsUserFor(q.command)}
            onclick={() => run(q.command)}
          >{q.label}</button>
        {/each}
      </div>
    {/if}

    <div class="flex gap-2">
      <input
        type="text"
        bind:value={commandInput}
        onkeydown={onKeydown}
        placeholder={canSend ? "Enter command  (↑ ↓ for history)" : "Server must be running"}
        disabled={!canSend || loading}
        class="flex-1 bg-[#0d1117] border border-cubelit-border rounded-lg px-3 py-2 text-sm font-mono text-cubelit-text placeholder-cubelit-muted/40 focus:outline-none focus:border-cubelit-accent disabled:opacity-50"
      />
      <Button
        size="sm"
        disabled={!canSend || !commandInput.trim() || loading}
        onclick={() => run(commandInput)}
      >{loading ? "…" : "Send"}</Button>
    </div>

    {#if responsesInLog}
      <p class="text-xs text-cubelit-muted">
        Type commands without a leading slash (e.g. <code class="text-cubelit-text">list</code>,
        not <code class="text-cubelit-text">/list</code>). Replies appear in the Console log above.
      </p>
    {/if}

    {#if output.length > 0}
      <div class="bg-[#0d1117] border border-cubelit-border rounded-xl p-4 space-y-3 max-h-64 overflow-y-auto font-mono text-xs">
        {#each output as entry}
          <div>
            <div class="text-cubelit-accent">❯ {entry.cmd}</div>
            <div class="mt-0.5 {entry.error ? 'text-cubelit-error' : entry.note ? 'text-cubelit-muted italic' : 'text-gray-300'} whitespace-pre-wrap pl-3">
              {entry.response}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>
