import { describe, expect, it, vi } from "vitest";
import type { PaneRuntimeView } from "@/contracts/tauri-bindings";
import type { RuntimeClient } from "@/app-shell/clients";
import { createRuntimeStore } from "./store";

function makeMockRuntimeClient(): RuntimeClient {
  return {
    dispatch: vi.fn().mockResolvedValue(undefined),
    dispatchBrowserSurface: vi.fn().mockResolvedValue(undefined),
    listenStatusChanged: vi.fn().mockResolvedValue(() => {}),
    listenTerminalOutput: vi.fn().mockResolvedValue(() => {}),
    listenBrowserLocationObserved: vi.fn().mockResolvedValue(() => {}),
  };
}

function makeRuntimeDto(overrides?: Partial<PaneRuntimeView>): PaneRuntimeView {
  return {
    paneId: "p1",
    runtimeSessionId: "sess-1",
    kind: "terminal",
    status: "running",
    lastError: null,
    browserLocation: null,
    terminalCwd: "/home/user",
    ...overrides,
  };
}

describe("createRuntimeStore", () => {
  describe("loadBootstrap", () => {
    it("maps PaneRuntimeView DTOs to RuntimeReadModel before storing", () => {
      const client = makeMockRuntimeClient();
      const store = createRuntimeStore(client);

      const dtos: PaneRuntimeView[] = [
        makeRuntimeDto({ paneId: "p1", kind: "terminal", status: "running" }),
        makeRuntimeDto({
          paneId: "p2",
          kind: "browser",
          status: "starting",
          browserLocation: "https://example.com",
          terminalCwd: null,
        }),
      ];

      store.getState().loadBootstrap(dtos);

      const runtimes = store.getState().runtimes;
      expect(Object.keys(runtimes)).toHaveLength(2);

      const p1 = runtimes["p1"];
      expect(p1).toEqual({
        paneId: "p1",
        runtimeSessionId: "sess-1",
        kind: "terminal",
        status: "running",
        lastError: null,
        browserLocation: null,
        terminalCwd: "/home/user",
      });

      const p2 = runtimes["p2"];
      expect(p2).toEqual({
        paneId: "p2",
        runtimeSessionId: "sess-1",
        kind: "browser",
        status: "starting",
        lastError: null,
        browserLocation: "https://example.com",
        terminalCwd: null,
      });
    });

    it("stores an empty map when given an empty array", () => {
      const client = makeMockRuntimeClient();
      const store = createRuntimeStore(client);

      store.getState().loadBootstrap([]);

      expect(store.getState().runtimes).toEqual({});
    });

    it("stored models are frozen snapshots independent of the input DTO", () => {
      const client = makeMockRuntimeClient();
      const store = createRuntimeStore(client);
      const dto = makeRuntimeDto();

      store.getState().loadBootstrap([dto]);

      const stored = store.getState().runtimes["p1"];
      expect(stored).toBeDefined();
      expect(stored).not.toBe(dto);
      expect(stored.paneId).toBe(dto.paneId);
    });
  });

  describe("runtime status listener", () => {
    it("maps incoming PaneRuntimeView DTO through mapper before updating store", () => {
      const client = makeMockRuntimeClient();
      let statusHandler: ((runtime: PaneRuntimeView) => void) | null = null;
      client.listenStatusChanged = vi.fn((handler) => {
        statusHandler = handler;
        return Promise.resolve(() => {});
      });

      const store = createRuntimeStore(client);
      store.getState().loadBootstrap([
        makeRuntimeDto({ paneId: "p1", status: "starting" }),
      ]);

      void store.getState().initializeListeners();

      expect(statusHandler).not.toBeNull();

      const updatedDto = makeRuntimeDto({ paneId: "p1", status: "running" });
      statusHandler!(updatedDto);

      const runtime = store.getState().runtimes["p1"];
      expect(runtime.status).toBe("running");
      expect(runtime).not.toBe(updatedDto);
    });
  });
});
