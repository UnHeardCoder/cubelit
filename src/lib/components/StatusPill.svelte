<script lang="ts">
  import type { CubelitStatus } from '$lib/types/server';

  interface Props {
    status: CubelitStatus | string;
    /** Apply glass effect for use over hero images */
    glass?: boolean;
  }
  let { status, glass = false }: Props = $props();

  const labels: Record<string, string> = {
    running:  'Online',
    starting: 'Starting',
    stopped:  'Offline',
    created:  'Created',
    error:    'Error',
  };

  function pillClass(): string {
    switch (status) {
      case 'running':  return 'success';
      case 'starting': return 'warning';
      case 'error':    return 'error';
      default:         return '';
    }
  }

  function label(): string {
    return labels[status] ?? status;
  }
</script>

<span
  class="pill {pillClass()}"
  style={glass ? 'background: rgba(0,0,0,0.3); backdrop-filter: blur(8px); border-color: rgba(255,255,255,0.15);' : ''}
>
  <span class="status-dot {status}"></span>
  {label()}
</span>
