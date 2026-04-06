import { getOnboardingStatus } from "$lib/api/system";
import type { DockerStatus, OnboardingStatus } from "$lib/types/docker";

let dockerStatus = $state<DockerStatus>({
  available: false,
  version: null,
  error: null,
});

let checking = $state(false);
let onboardingStatus = $state<OnboardingStatus | null>(null);

export function getDockerStore() {
  async function check() {
    checking = true;
    try {
      onboardingStatus = await getOnboardingStatus();
      dockerStatus = {
        available: onboardingStatus.docker.state === "ready",
        version: onboardingStatus.docker.version,
        error: onboardingStatus.docker.error,
      };
    } catch (e) {
      onboardingStatus = null;
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
    get onboarding() {
      return onboardingStatus;
    },
    check,
  };
}
