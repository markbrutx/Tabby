import { describe, expect, it, vi } from "vitest";
import type { WorkspaceView } from "@/contracts/tauri-bindings";
import type { WorkspaceClient } from "@/app-shell/clients";
import { createWorkspaceStore } from "./store";
import type { WorkspaceStoreDeps } from "./store";

function makeWorkspaceView(overrides?: Partial<WorkspaceView>): WorkspaceView {
  return {
    activeTabId: "t1",
    tabs: [
      {
        tabId: "t1",
        title: "Tab 1",
        layout: { type: "pane", paneId: "p1" },
        panes: [
          {
            paneId: "p1",
            title: "Terminal",
            spec: {
              kind: "terminal",
              launch_profile_id: "default",
              working_directory: "/home",
              command_override: null,
            },
          },
        ],
        activePaneId: "p1",
      },
    ],
    ...overrides,
  };
}

function makeMockWorkspaceClient(
  overrides?: Partial<WorkspaceClient>,
): WorkspaceClient {
  return {
    bootstrap: vi.fn().mockResolvedValue({
      workspace: makeWorkspaceView(),
      settings: {
        defaultLayout: "single",
        defaultTerminalProfileId: "terminal",
        defaultWorkingDirectory: "~",
        defaultCustomCommand: "",
        fontSize: 14,
        theme: "dark",
        launchFullscreen: false,
        hasCompletedOnboarding: true,
        lastWorkingDirectory: null,
      },
      profileCatalog: {
        terminalProfiles: [
          {
            id: "terminal",
            label: "Terminal",
            description: "Default terminal",
            startupCommandTemplate: null,
          },
        ],
      },
      runtimeProjections: [],
    }),
    dispatch: vi.fn().mockResolvedValue(makeWorkspaceView()),
    listenProjectionUpdated: vi.fn().mockResolvedValue(() => {}),
    ...overrides,
  };
}

function makeMockDeps(
  clientOverrides?: Partial<WorkspaceClient>,
): WorkspaceStoreDeps {
  const settingsStore = {
    getState: () => ({
      settings: null as { hasCompletedOnboarding: boolean } | null,
      loadBootstrap: vi.fn(),
      updateSettings: vi.fn().mockResolvedValue(undefined),
    }),
  };

  const runtimeStore = {
    getState: () => ({
      loadBootstrap: vi.fn(),
    }),
  };

  return {
    workspaceClient: makeMockWorkspaceClient(clientOverrides),
    getSettingsStore: () => settingsStore,
    getRuntimeStore: () => runtimeStore,
  };
}

describe("createWorkspaceStore", () => {
  it("initializes workspace from bootstrap and clears hydrating state", async () => {
    const deps = makeMockDeps();
    const store = createWorkspaceStore(deps);

    expect(store.getState().isHydrating).toBe(true);
    expect(store.getState().workspace).toBeNull();

    await store.getState().initialize();

    expect(store.getState().isHydrating).toBe(false);
    expect(store.getState().workspace).not.toBeNull();
    expect(store.getState().workspace?.activeTabId).toBe("t1");
    expect(store.getState().workspace?.tabs).toHaveLength(1);
    expect(deps.workspaceClient.bootstrap).toHaveBeenCalledOnce();
  });

  it("dispatches closeTab command through injected client", async () => {
    const updatedView = makeWorkspaceView({ tabs: [], activeTabId: "" });
    const deps = makeMockDeps({
      dispatch: vi.fn().mockResolvedValue(updatedView),
    });
    const store = createWorkspaceStore(deps);

    await store.getState().initialize();
    await store.getState().closeTab("t1");

    expect(deps.workspaceClient.dispatch).toHaveBeenCalledWith({
      kind: "closeTab",
      tab_id: "t1",
    });
    expect(store.getState().workspace?.tabs).toHaveLength(0);
  });

  it("sets error state when bootstrap fails", async () => {
    const deps = makeMockDeps({
      bootstrap: vi.fn().mockRejectedValue(new Error("connection refused")),
    });
    const store = createWorkspaceStore(deps);

    await store.getState().initialize();

    expect(store.getState().isHydrating).toBe(false);
    expect(store.getState().error).toBe("connection refused");
    expect(store.getState().workspace).toBeNull();
  });

  it("sets error state when dispatch fails", async () => {
    const deps = makeMockDeps({
      dispatch: vi.fn().mockRejectedValue(new Error("dispatch error")),
    });
    const store = createWorkspaceStore(deps);

    await store.getState().initialize();
    await store.getState().closeTab("t1");

    expect(store.getState().error).toBe("dispatch error");
  });

  it("clears error via clearError action", async () => {
    const deps = makeMockDeps({
      dispatch: vi.fn().mockRejectedValue(new Error("fail")),
    });
    const store = createWorkspaceStore(deps);

    await store.getState().initialize();
    await store.getState().closeTab("t1");

    expect(store.getState().error).toBe("fail");

    store.getState().clearError();

    expect(store.getState().error).toBeNull();
  });

  it("shows wizard tab when workspace has no tabs", async () => {
    const emptyView = makeWorkspaceView({ tabs: [], activeTabId: "" });
    const deps = makeMockDeps({
      bootstrap: vi.fn().mockResolvedValue({
        workspace: emptyView,
        settings: {
          defaultLayout: "single",
          defaultTerminalProfileId: "terminal",
          defaultWorkingDirectory: "~",
          defaultCustomCommand: "",
          fontSize: 14,
          theme: "dark",
          launchFullscreen: false,
          hasCompletedOnboarding: false,
          lastWorkingDirectory: null,
        },
        profileCatalog: { terminalProfiles: [] },
        runtimeProjections: [],
      }),
    });
    const store = createWorkspaceStore(deps);

    await store.getState().initialize();

    expect(store.getState().wizardTab).not.toBeNull();
    expect(store.getState().wizardTab?.title).toBe("Workspace 1");
  });
});
