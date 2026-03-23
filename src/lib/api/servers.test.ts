import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import {
  listCubelits,
  getCubelit,
  createServer,
  startServer,
  stopServer,
  restartServer,
  deleteServer,
  syncServerStatus,
  syncAllStatuses,
  renameServer,
} from "./servers";

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
  mockInvoke.mockReset();
});

describe("listCubelits", () => {
  it("calls list_cubelits with no args", async () => {
    mockInvoke.mockResolvedValue([]);
    await listCubelits();
    expect(mockInvoke).toHaveBeenCalledWith("list_cubelits");
  });
});

describe("getCubelit", () => {
  it("calls get_cubelit with the id", async () => {
    mockInvoke.mockResolvedValue({ id: "abc" });
    await getCubelit("abc");
    expect(mockInvoke).toHaveBeenCalledWith("get_cubelit", { id: "abc" });
  });
});

describe("createServer", () => {
  it("calls create_server with config", async () => {
    const config = { name: "My Server", recipe_id: "minecraft-java" } as any;
    mockInvoke.mockResolvedValue({ id: "new-id" });
    await createServer(config);
    expect(mockInvoke).toHaveBeenCalledWith("create_server", { config });
  });
});

describe("startServer", () => {
  it("calls start_server with id", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await startServer("s1");
    expect(mockInvoke).toHaveBeenCalledWith("start_server", { id: "s1" });
  });
});

describe("stopServer", () => {
  it("calls stop_server with id", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await stopServer("s1");
    expect(mockInvoke).toHaveBeenCalledWith("stop_server", { id: "s1" });
  });
});

describe("restartServer", () => {
  it("calls restart_server with id", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await restartServer("s1");
    expect(mockInvoke).toHaveBeenCalledWith("restart_server", { id: "s1" });
  });
});

describe("deleteServer", () => {
  it("calls delete_server with id and deleteData flag", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await deleteServer("s1", true);
    expect(mockInvoke).toHaveBeenCalledWith("delete_server", { id: "s1", deleteData: true });
  });
});

describe("syncServerStatus", () => {
  it("calls sync_server_status with id", async () => {
    mockInvoke.mockResolvedValue({ id: "s1", status: "running" });
    await syncServerStatus("s1");
    expect(mockInvoke).toHaveBeenCalledWith("sync_server_status", { id: "s1" });
  });
});

describe("syncAllStatuses", () => {
  it("calls sync_all_statuses", async () => {
    mockInvoke.mockResolvedValue([]);
    await syncAllStatuses();
    expect(mockInvoke).toHaveBeenCalledWith("sync_all_statuses");
  });
});

describe("renameServer", () => {
  it("calls rename_server with id and name", async () => {
    mockInvoke.mockResolvedValue({ id: "s1", name: "New Name" });
    await renameServer("s1", "New Name");
    expect(mockInvoke).toHaveBeenCalledWith("rename_server", { id: "s1", name: "New Name" });
  });
});
