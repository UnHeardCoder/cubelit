<script lang="ts">
  import type { CubelitStatus } from "$lib/types/server";

  interface Props {
    status: CubelitStatus;
  }

  let { status }: Props = $props();

  const labels: Record<string, string> = {
    starting: "Starting",
    running: "Online",
    stopped: "Offline",
    created: "Created",
    error: "Error",
  };

  const colors: Record<string, string> = {
    starting: "bg-cubelit-warning",
    running: "bg-cubelit-accent",
    stopped: "bg-cubelit-muted/60",
    created: "bg-cubelit-warning",
    error: "bg-cubelit-error",
  };
</script>

<div class="absolute top-0 right-0 overflow-hidden w-24 h-24 pointer-events-none">
  <div
    class="absolute top-3 -right-6 w-28 text-center text-xs font-bold text-white py-1 rotate-45 {colors[status] ?? 'bg-cubelit-muted'}"
  >
    {labels[status] ?? status}
  </div>
</div>
