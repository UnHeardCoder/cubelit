import {
  syncAllStatuses,
  startServer,
  stopServer,
  restartServer,
  deleteServer,
} from "$lib/api/servers";
import type { Cubelit } from "$lib/types/server";

let servers = $state<Cubelit[]>([]);
let loading = $state(false);

export function getServersStore() {
  async function load() {
    loading = true;
    try {
      servers = await syncAllStatuses();
    } catch (e) {
      console.error("Failed to load servers:", e);
    } finally {
      loading = false;
    }
  }

  async function start(id: string) {
    try {
      await startServer(id);
      await load();
    } catch (e) {
      console.error("Failed to start server:", e);
      throw e;
    }
  }

  async function stop(id: string) {
    try {
      await stopServer(id);
      await load();
    } catch (e) {
      console.error("Failed to stop server:", e);
      throw e;
    }
  }

  async function restart(id: string) {
    try {
      await restartServer(id);
      await load();
    } catch (e) {
      console.error("Failed to restart server:", e);
      throw e;
    }
  }

  async function remove(id: string, deleteData: boolean) {
    try {
      await deleteServer(id, deleteData);
      await load();
    } catch (e) {
      console.error("Failed to delete server:", e);
      throw e;
    }
  }

  return {
    get servers() {
      return servers;
    },
    get loading() {
      return loading;
    },
    load,
    start,
    stop,
    restart,
    remove,
  };
}
