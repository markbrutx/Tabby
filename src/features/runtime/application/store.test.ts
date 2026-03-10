import { describe, expect, it, vi } from "vitest";
import type { PaneRuntimeView } from "@/contracts/tauri-bindings";
import type { RuntimeReadModel } from "@/features/runtime/domain/models";
import type { RuntimeClient } from "@/app-shell/clients";
import { createRuntimeStore } from "./store";
import type { RuntimeStoreDeps } from "./store";

function makeMockRuntimeClient(): RuntimeClient {
  return {
    dispatch: vi.fn().mockResolvedValue(undefined),
    dispatchBrowserSurface: vi.fn().mockResolvedValue(undefined),
    listenStatusChanged: vi.fn().mockResolvedValue(() => {}),
    listenTerminalOutput: vi.fn().mockResolvedValue(() => {}),
  };
}

function makeMockDeps(clientOverrides?: Partial<RuntimeClient>): RuntimeStoreDeps {
  return {
    runtimeClient: { ...makeMockRuntimeClient(), ...clientOverrides },
    initTerminalDispatcher: vi.fn().mockResolvedValue(undefined),
    teardownTerminalDispatcher: vi.fn(),
  };
}

function makeRuntimeReadModel(overrides?: Partial<RuntimeReadModel>): RuntimeReadModel {
  return {
    paneId: "p1",
    runtimeSessionId: "sess-1",
    kind: "terminal",
    status: "running",
    lastError: null,
    browserLocation: null,
    terminalCwd: "/home/user",
    gitRepoPath: null,
    ...overrides,
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
    gitRepoPath: null,
    ...overrides,
  };
}

describe("createRuntimeStore", () => {
  describe("loadBootstrap", () => {
    it("stores RuntimeReadModel entries keyed by paneId", () => {
      const deps = makeMockDeps();
      const store = createRuntimeStore(deps);

      const readModels: RuntimeReadModel[] = [
        makeRuntimeReadModel({ paneId: "p1", kind: "terminal", status: "running" }),
        makeRuntimeReadModel({
          paneId: "p2",
          kind: "browser",
          status: "starting",
          browserLocation: "https://example.com",
          terminalCwd: null,
        }),
      ];

      store.getState().loadBootstrap(readModels);

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
        gitRepoPath: null,
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
        gitRepoPath: null,
      });
    });

    it("stores an empty map when given an empty array", () => {
      const deps = makeMockDeps();
      const store = createRuntimeStore(deps);

      store.getState().loadBootstrap([]);

      expect(store.getState().runtimes).toEqual({});
    });

    it("accepts pre-mapped read models without further transformation", () => {
      const deps = makeMockDeps();
      const store = createRuntimeStore(deps);
      const readModel = makeRuntimeReadModel();

      store.getState().loadBootstrap([readModel]);

      const stored = store.getState().runtimes["p1"];
      expect(stored).toBeDefined();
      expect(stored.paneId).toBe(readModel.paneId);
      expect(stored.kind).toBe(readModel.kind);
    });
  });

  describe("runtime status listener", () => {
    it("maps incoming PaneRuntimeView DTO through mapper before updating store", () => {
      let statusHandler: ((runtime: PaneRuntimeView) => void) | null = null;
      const deps = makeMockDeps({
        listenStatusChanged: vi.fn((handler) => {
          statusHandler = handler;
          return Promise.resolve(() => {});
        }),
      });

      const store = createRuntimeStore(deps);
      store.getState().loadBootstrap([
        makeRuntimeReadModel({ paneId: "p1", status: "starting" }),
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

  describe("terminal dispatcher delegation", () => {
    it("delegates initTerminalOutputDispatcher to injected initTerminalDispatcher", async () => {
      const initTerminalDispatcher = vi.fn().mockResolvedValue(undefined);
      const deps = makeMockDeps();
      deps.initTerminalDispatcher = initTerminalDispatcher;
      const store = createRuntimeStore(deps);

      await store.getState().initTerminalOutputDispatcher();

      expect(initTerminalDispatcher).toHaveBeenCalledWith(deps.runtimeClient);
    });

    it("delegates teardownTerminalOutputDispatcher to injected teardownTerminalDispatcher", () => {
      const teardownTerminalDispatcher = vi.fn();
      const deps = makeMockDeps();
      deps.teardownTerminalDispatcher = teardownTerminalDispatcher;
      const store = createRuntimeStore(deps);

      store.getState().teardownTerminalOutputDispatcher();

      expect(teardownTerminalDispatcher).toHaveBeenCalledOnce();
    });
  });

  describe("browser location via unified RuntimeStatusChangedEvent", () => {
    it("updates browser_location from RuntimeStatusChangedEvent payload", () => {
      let statusHandler: ((runtime: PaneRuntimeView) => void) | null = null;
      const deps = makeMockDeps({
        listenStatusChanged: vi.fn((handler) => {
          statusHandler = handler;
          return Promise.resolve(() => {});
        }),
      });

      const store = createRuntimeStore(deps);
      store.getState().loadBootstrap([
        makeRuntimeReadModel({
          paneId: "pane-b",
          kind: "browser",
          status: "running",
          browserLocation: "https://example.com",
        }),
      ]);

      void store.getState().initializeListeners();

      // Simulate browser navigation arriving as RuntimeStatusChangedEvent
      statusHandler!(
        makeRuntimeDto({
          paneId: "pane-b",
          kind: "browser",
          status: "running",
          browserLocation: "https://docs.rs",
        }),
      );

      const runtime = store.getState().runtimes["pane-b"];
      expect(runtime.browserLocation).toBe("https://docs.rs");
    });
  });

  describe("isolation", () => {
    it("can be instantiated and tested with no cross-feature dependencies", () => {
      const deps = makeMockDeps();
      const store = createRuntimeStore(deps);

      expect(store.getState().runtimes).toEqual({});
      store.getState().loadBootstrap([makeRuntimeReadModel()]);
      expect(Object.keys(store.getState().runtimes)).toHaveLength(1);
    });
  });
});
