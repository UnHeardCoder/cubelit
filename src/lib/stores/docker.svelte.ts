import { checkDockerStatus } from "$lib/api/docker";
import type { DockerStatus } from "$lib/types/docker";

let dockerStatus = $state<DockerStatus>({
  available: false,
  version: null,
  error: null,
});

let checking = $state(false);

export function getDockerStore() {
  async function check() {
    checking = true;
    try {
      dockerStatus = await checkDockerStatus();
    } catch (e) {
      dockerStatus = {
        available: false,
        version: null,
        error: String(e),
      };
    } finally {
      checking = false;
    }
  }

  return {
    get status() {
      return dockerStatus;
    },
    get checking() {
      return checking;
    },
    check,
  };
}
