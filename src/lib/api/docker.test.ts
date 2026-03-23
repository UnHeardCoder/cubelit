import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { checkDockerStatus, getServerStats, updateServerSettings } from "./docker";

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
  mockInvoke.mockReset();
});

describe("checkDockerStatus", () => {
  it("calls check_docker_status", async () => {
    mockInvoke.mockResolvedValue({ available: true, version: "24.0.0" });
    await checkDockerStatus();
    expect(mockInvoke).toHaveBeenCalledWith("check_docker_status");
  });
});

describe("getServerStats", () => {
  it("calls get_server_stats with id", async () => {
    mockInvoke.mockResolvedValue({ cpu_percent: 5, memory_mb: 512 });
    await getServerStats("s1");
    expect(mockInvoke).toHaveBeenCalledWith("get_server_stats", { id: "s1" });
  });
});

describe("updateServerSettings", () => {
  it("calls update_server_settings with id and environment", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await updateServerSettings("s1", { MEMORY: "4G" });
    expect(mockInvoke).toHaveBeenCalledWith("update_server_settings", {
      id: "s1",
      environment: { MEMORY: "4G" },
    });
  });
});
