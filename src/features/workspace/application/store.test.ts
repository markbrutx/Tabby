import { describe, expect, it, vi } from "vitest";
import type { WorkspaceView } from "@/contracts/tauri-bindings";
import type { WorkspaceClient } from "@/app-shell/clients";
import type { WorkspaceReadModel } from "@/features/workspace/domain/models";
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

function makeWorkspaceReadModel(
  overrides?: Partial<WorkspaceReadModel>,
): WorkspaceReadModel {
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
              launchProfileId: "default",
              workingDirectory: "/home",
              commandOverride: null,
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
    bootstrap: vi.fn().mockResolvedValue(makeWorkspaceReadModel()),
    dispatch: vi.fn().mockResolvedValue(makeWorkspaceView()),
    listenProjectionUpdated: vi.fn().mockResolvedValue(() => {}),
    ...overrides,
  };
}

function makeMockDeps(
  clientOverrides?: Partial<WorkspaceClient>,
  depsOverrides?: Partial<WorkspaceStoreDeps>,
): WorkspaceStoreDeps {
  return {
    workspaceClient: makeMockWorkspaceClient(clientOverrides),
    onWizardComplete: vi.fn(),
    ...depsOverrides,
  };
}

describe("createWorkspaceStore", () => {
  it("loadBootstrap sets workspace and clears hydrating state", async () => {
    const deps = makeMockDeps();
    const store = createWorkspaceStore(deps);

    expect(store.getState().isHydrating).toBe(true);
    expect(store.getState().workspace).toBeNull();

    await store.getState().loadBootstrap(makeWorkspaceReadModel());

    expect(store.getState().isHydrating).toBe(false);
    expect(store.getState().workspace).not.toBeNull();
    expect(store.getState().workspace?.activeTabId).toBe("t1");
    expect(store.getState().workspace?.tabs).toHaveLength(1);
  });

  it("beginBootstrap sets hydrating true and clears error", () => {
    const deps = makeMockDeps();
    const store = createWorkspaceStore(deps);

    store.getState().setBootstrapError("some error");
    expect(store.getState().isHydrating).toBe(false);
    expect(store.getState().error).toBe("some error");

    store.getState().beginBootstrap();

    expect(store.getState().isHydrating).toBe(true);
    expect(store.getState().error).toBeNull();
  });

  it("setBootstrapError sets error and clears hydrating", () => {
    const deps = makeMockDeps();
    const store = createWorkspaceStore(deps);

    store.getState().setBootstrapError("connection refused");

    expect(store.getState().isHydrating).toBe(false);
    expect(store.getState().error).toBe("connection refused");
    expect(store.getState().workspace).toBeNull();
  });

  it("dispatches closeTab command through injected client", async () => {
    const updatedView = makeWorkspaceView({ tabs: [], activeTabId: "" });
    const deps = makeMockDeps({
      dispatch: vi.fn().mockResolvedValue(updatedView),
    });
    const store = createWorkspaceStore(deps);

    await store.getState().loadBootstrap(makeWorkspaceReadModel());
    await store.getState().closeTab("t1");

    expect(deps.workspaceClient.dispatch).toHaveBeenCalledWith({
      kind: "closeTab",
      tab_id: "t1",
    });
    expect(store.getState().workspace?.tabs).toHaveLength(0);
  });

  it("sets error state when dispatch fails", async () => {
    const deps = makeMockDeps({
      dispatch: vi.fn().mockRejectedValue(new Error("dispatch error")),
    });
    const store = createWorkspaceStore(deps);

    await store.getState().loadBootstrap(makeWorkspaceReadModel());
    await store.getState().closeTab("t1");

    expect(store.getState().error).toBe("dispatch error");
  });

  it("clears error via clearError action", async () => {
    const deps = makeMockDeps({
      dispatch: vi.fn().mockRejectedValue(new Error("fail")),
    });
    const store = createWorkspaceStore(deps);

    await store.getState().loadBootstrap(makeWorkspaceReadModel());
    await store.getState().closeTab("t1");

    expect(store.getState().error).toBe("fail");

    store.getState().clearError();

    expect(store.getState().error).toBeNull();
  });

  it("shows wizard tab when workspace has no tabs", async () => {
    const deps = makeMockDeps();
    const store = createWorkspaceStore(deps);

    await store.getState().loadBootstrap(
      makeWorkspaceReadModel({ tabs: [], activeTabId: "" }),
    );

    expect(store.getState().wizardTab).not.toBeNull();
    expect(store.getState().wizardTab?.title).toBe("Workspace 1");
  });

  it("dispatches setActiveTab command and updates workspace state", async () => {
    const updatedView = makeWorkspaceView({ activeTabId: "t2" });
    const deps = makeMockDeps({
      dispatch: vi.fn().mockResolvedValue(updatedView),
    });
    const store = createWorkspaceStore(deps);

    await store.getState().loadBootstrap(makeWorkspaceReadModel());
    await store.getState().setActiveTab("t2");

    expect(deps.workspaceClient.dispatch).toHaveBeenCalledWith({
      kind: "setActiveTab",
      tab_id: "t2",
    });
    expect(store.getState().workspace?.activeTabId).toBe("t2");
  });

  it("dispatches splitPane command with direction and pane spec", async () => {
    const splitView = makeWorkspaceView({
      tabs: [
        {
          tabId: "t1",
          title: "Tab 1",
          layout: {
            type: "split",
            direction: "horizontal",
            ratio: 0.5,
            first: { type: "pane", paneId: "p1" },
            second: { type: "pane", paneId: "p2" },
          },
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
            {
              paneId: "p2",
              title: "Terminal 2",
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
    });
    const deps = makeMockDeps({
      dispatch: vi.fn().mockResolvedValue(splitView),
    });
    const store = createWorkspaceStore(deps);

    await store.getState().loadBootstrap(makeWorkspaceReadModel());
    const paneSpec = {
      kind: "terminal" as const,
      launchProfileId: "default",
      workingDirectory: "/home",
      commandOverride: null,
    };
    await store.getState().splitPane("p1", "horizontal", paneSpec);

    expect(deps.workspaceClient.dispatch).toHaveBeenCalledWith({
      kind: "splitPane",
      pane_id: "p1",
      direction: "horizontal",
      pane_spec: {
        kind: "terminal",
        launch_profile_id: "default",
        working_directory: "/home",
        command_override: null,
      },
    });
    expect(store.getState().workspace?.tabs[0].panes).toHaveLength(2);
  });

  it("transitions isWorking state during mutations", async () => {
    let resolveDispatch: (value: WorkspaceView) => void;
    const dispatchPromise = new Promise<WorkspaceView>((resolve) => {
      resolveDispatch = resolve;
    });
    const deps = makeMockDeps({
      dispatch: vi.fn().mockReturnValue(dispatchPromise),
    });
    const store = createWorkspaceStore(deps);

    await store.getState().loadBootstrap(makeWorkspaceReadModel());

    expect(store.getState().isWorking).toBe(false);

    const closePromise = store.getState().closeTab("t1");

    expect(store.getState().isWorking).toBe(true);

    resolveDispatch!(makeWorkspaceView({ tabs: [] }));
    await closePromise;

    expect(store.getState().isWorking).toBe(false);
  });

  it("openSetupWizard creates a wizard tab", async () => {
    const deps = makeMockDeps();
    const store = createWorkspaceStore(deps);

    await store.getState().loadBootstrap(makeWorkspaceReadModel());

    expect(store.getState().wizardTab).toBeNull();

    store.getState().openSetupWizard();

    expect(store.getState().wizardTab).not.toBeNull();
    expect(store.getState().wizardTab?.title).toBe("Workspace 2");
  });

  it("closeSetupWizard clears wizard when tabs exist", async () => {
    const deps = makeMockDeps();
    const store = createWorkspaceStore(deps);

    await store.getState().loadBootstrap(makeWorkspaceReadModel());

    store.getState().openSetupWizard();
    expect(store.getState().wizardTab).not.toBeNull();

    store.getState().closeSetupWizard();
    expect(store.getState().wizardTab).toBeNull();
  });

  it("closeSetupWizard keeps wizard when no tabs exist", async () => {
    const deps = makeMockDeps();
    const store = createWorkspaceStore(deps);

    await store.getState().loadBootstrap(
      makeWorkspaceReadModel({ tabs: [], activeTabId: "" }),
    );
    expect(store.getState().wizardTab).not.toBeNull();

    store.getState().closeSetupWizard();
    expect(store.getState().wizardTab).not.toBeNull();
  });

  it("createTabFromWizard dispatches openTab and calls onWizardComplete", async () => {
    const tabView = makeWorkspaceView();
    const onWizardComplete = vi.fn();
    const deps = makeMockDeps(
      { dispatch: vi.fn().mockResolvedValue(tabView) },
      { onWizardComplete },
    );
    const store = createWorkspaceStore(deps);

    await store.getState().loadBootstrap(makeWorkspaceReadModel());
    await store.getState().createTabFromWizard({
      groups: [
        {
          mode: "terminal",
          profileId: "terminal",
          workingDirectory: "~",
          count: 1,
        },
      ],
    });

    expect(deps.workspaceClient.dispatch).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: "openTab",
        auto_layout: true,
      }),
    );
    expect(onWizardComplete).toHaveBeenCalledOnce();
  });

  it("dispatches closePane command through injected client", async () => {
    const updatedView = makeWorkspaceView();
    const deps = makeMockDeps({
      dispatch: vi.fn().mockResolvedValue(updatedView),
    });
    const store = createWorkspaceStore(deps);

    await store.getState().loadBootstrap(makeWorkspaceReadModel());
    await store.getState().closePane("p1");

    expect(deps.workspaceClient.dispatch).toHaveBeenCalledWith({
      kind: "closePane",
      pane_id: "p1",
    });
  });

  it("dispatches swapPaneSlots command through injected client", async () => {
    const updatedView = makeWorkspaceView();
    const deps = makeMockDeps({
      dispatch: vi.fn().mockResolvedValue(updatedView),
    });
    const store = createWorkspaceStore(deps);

    await store.getState().loadBootstrap(makeWorkspaceReadModel());
    await store.getState().swapPaneSlots("p1", "p2");

    expect(deps.workspaceClient.dispatch).toHaveBeenCalledWith({
      kind: "swapPaneSlots",
      pane_id_a: "p1",
      pane_id_b: "p2",
    });
  });

  describe("isolation", () => {
    it("can be instantiated and tested with no cross-feature dependencies", async () => {
      const deps = makeMockDeps();
      const store = createWorkspaceStore(deps);

      expect(store.getState().workspace).toBeNull();
      expect(store.getState().isHydrating).toBe(true);

      await store.getState().loadBootstrap(makeWorkspaceReadModel());

      expect(store.getState().workspace).not.toBeNull();
      expect(store.getState().isHydrating).toBe(false);
    });
  });
});
