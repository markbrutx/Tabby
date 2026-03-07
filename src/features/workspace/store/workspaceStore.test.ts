import { describe, expect, it, vi } from "vitest";
import type {
  BootstrapSnapshot,
  PaneLifecycleEvent,
  SplitNode,
  WorkspaceSettings,
  WorkspaceSnapshot,
} from "@/features/workspace/domain";
import { createWorkspaceStore } from "@/features/workspace/store/workspaceStore";
import type { WorkspaceTransport } from "@/lib/bridge";

const singleLayout: SplitNode = { type: "pane", paneId: "pane-1" };

const settings: WorkspaceSettings = {
  defaultLayout: "1x1",
  defaultProfileId: "terminal",
  defaultWorkingDirectory: "/Users/mark/workspaces/tabby",
  defaultCustomCommand: "",
  fontSize: 13,
  theme: "system",
  launchFullscreen: true,
  hasCompletedOnboarding: true,
};

const workspace: WorkspaceSnapshot = {
  activeTabId: "tab-1",
  tabs: [
    {
      id: "tab-1",
      title: "Workspace 1",
      layout: singleLayout,
      activePaneId: "pane-1",
      panes: [
        {
          id: "pane-1",
          sessionId: "session-1",
          title: "Pane 1",
          cwd: settings.defaultWorkingDirectory,
          profileId: "terminal",
          profileLabel: "Terminal",
          startupCommand: null,
          status: "running",
        },
      ],
    },
  ],
};

const bootstrapPayload: BootstrapSnapshot = {
  workspace,
  settings,
  profiles: [
    {
      id: "terminal",
      label: "Terminal",
      description: "Pure login shell",
      startupCommand: null,
    },
  ],
};

function createTransportMock(overrides: Partial<WorkspaceTransport> = {}): WorkspaceTransport {
  return {
    bootstrapWorkspace: vi.fn().mockResolvedValue(bootstrapPayload),
    createTab: vi.fn().mockResolvedValue(workspace),
    closeTab: vi.fn().mockResolvedValue(workspace),
    setActiveTab: vi.fn().mockResolvedValue(workspace),
    focusPane: vi.fn().mockResolvedValue(workspace),
    updatePaneProfile: vi.fn().mockResolvedValue(workspace),
    updatePaneCwd: vi.fn().mockResolvedValue(workspace),
    restartPane: vi.fn().mockResolvedValue(workspace),
    splitPane: vi.fn().mockResolvedValue(workspace),
    closePane: vi.fn().mockResolvedValue(workspace),
    writePty: vi.fn().mockResolvedValue(undefined),
    resizePty: vi.fn().mockResolvedValue(undefined),
    getAppSettings: vi.fn().mockResolvedValue(settings),
    updateAppSettings: vi.fn().mockResolvedValue(settings),
    resetAppSettings: vi.fn().mockResolvedValue(settings),
    listenToPtyOutput: vi.fn().mockResolvedValue(() => undefined),
    listenToPaneLifecycle: vi.fn().mockResolvedValue(() => undefined),
    trackPaneCwd: vi.fn().mockResolvedValue(undefined),
    swapPanes: vi.fn().mockResolvedValue(workspace),
    createBrowserWebview: vi.fn().mockResolvedValue(undefined),
    navigateBrowser: vi.fn().mockResolvedValue(undefined),
    closeBrowserWebview: vi.fn().mockResolvedValue(undefined),
    setBrowserWebviewBounds: vi.fn().mockResolvedValue(undefined),
    setBrowserWebviewVisible: vi.fn().mockResolvedValue(undefined),
    listenToBrowserUrlChanged: vi.fn().mockResolvedValue(() => undefined),
    ...overrides,
  };
}

describe("createWorkspaceStore", () => {
  it("hydrates the workspace from the transport bootstrap snapshot", async () => {
    const transport = createTransportMock();
    const store = createWorkspaceStore(transport);

    await store.getState().initialize();

    expect(transport.bootstrapWorkspace).toHaveBeenCalledTimes(1);
    expect(store.getState().workspace).toEqual(workspace);
    expect(store.getState().settings).toEqual(settings);
    expect(store.getState().profiles).toEqual(bootstrapPayload.profiles);
    expect(store.getState().isHydrating).toBe(false);
  });

  it("uses current settings defaults when creating a tab without overrides", async () => {
    const transport = createTransportMock();
    const store = createWorkspaceStore(transport);

    await store.getState().initialize();
    await store.getState().createTab("1x1");

    expect(transport.createTab).toHaveBeenCalledWith({
      preset: "1x1",
      cwd: settings.defaultWorkingDirectory,
      profileId: settings.defaultProfileId,
      startupCommand: null,
    });
  });

  it("createTabFromWizard expands groups into flat paneConfigs", async () => {
    const transport = createTransportMock();
    const store = createWorkspaceStore(transport);
    await store.getState().initialize();

    await store.getState().createTabFromWizard({
      groups: [
        { profileId: "terminal", workingDirectory: "/a", count: 2 },
        { profileId: "claude", workingDirectory: "/b", count: 1 },
      ],
    });

    expect(transport.createTab).toHaveBeenCalledWith({
      preset: "1x1",
      cwd: null,
      profileId: null,
      startupCommand: null,
      paneConfigs: [
        { profileId: "terminal", cwd: "/a", startupCommand: null },
        { profileId: "terminal", cwd: "/a", startupCommand: null },
        { profileId: "claude", cwd: "/b", startupCommand: null },
      ],
    });
  });

  it("createTabFromWizard sends customCommand for custom profile", async () => {
    const transport = createTransportMock();
    const store = createWorkspaceStore(transport);
    await store.getState().initialize();

    await store.getState().createTabFromWizard({
      groups: [
        { profileId: "custom", workingDirectory: "/c", customCommand: "npm dev", count: 1 },
      ],
    });

    expect(transport.createTab).toHaveBeenCalledWith({
      preset: "1x1",
      cwd: null,
      profileId: null,
      startupCommand: null,
      paneConfigs: [
        { profileId: "custom", cwd: "/c", startupCommand: "npm dev" },
      ],
    });
  });

  it("lifecycle event updates pane status to exited", async () => {
    const handler: { current: ((event: PaneLifecycleEvent) => void) | null } = { current: null };
    const transport = createTransportMock({
      listenToPaneLifecycle: vi.fn(async (h: (event: PaneLifecycleEvent) => void) => {
        handler.current = h;
        return () => { handler.current = null; };
      }),
    });
    const store = createWorkspaceStore(transport);

    await store.getState().initialize();

    handler.current?.({
      paneId: "pane-1",
      sessionId: "session-1",
      status: "exited",
      errorMessage: null,
    });

    const pane = store.getState().workspace?.tabs[0].panes[0];
    expect(pane?.status).toBe("exited");
  });

  it("lifecycle event updates pane status to failed with error", async () => {
    const handler: { current: ((event: PaneLifecycleEvent) => void) | null } = { current: null };
    const transport = createTransportMock({
      listenToPaneLifecycle: vi.fn(async (h: (event: PaneLifecycleEvent) => void) => {
        handler.current = h;
        return () => { handler.current = null; };
      }),
    });
    const store = createWorkspaceStore(transport);

    await store.getState().initialize();

    handler.current?.({
      paneId: "pane-1",
      sessionId: "session-1",
      status: "failed",
      errorMessage: "Process exited with code 1",
    });

    const pane = store.getState().workspace?.tabs[0].panes[0];
    expect(pane?.status).toBe("failed");
  });

  it("lifecycle event with wrong session is ignored", async () => {
    const handler: { current: ((event: PaneLifecycleEvent) => void) | null } = { current: null };
    const transport = createTransportMock({
      listenToPaneLifecycle: vi.fn(async (h: (event: PaneLifecycleEvent) => void) => {
        handler.current = h;
        return () => { handler.current = null; };
      }),
    });
    const store = createWorkspaceStore(transport);

    await store.getState().initialize();

    handler.current?.({
      paneId: "pane-1",
      sessionId: "stale-session",
      status: "exited",
      errorMessage: null,
    });

    const pane = store.getState().workspace?.tabs[0].panes[0];
    expect(pane?.status).toBe("running");
  });

  it("lifecycle event for unknown pane does not crash", async () => {
    const handler: { current: ((event: PaneLifecycleEvent) => void) | null } = { current: null };
    const transport = createTransportMock({
      listenToPaneLifecycle: vi.fn(async (h: (event: PaneLifecycleEvent) => void) => {
        handler.current = h;
        return () => { handler.current = null; };
      }),
    });
    const store = createWorkspaceStore(transport);

    await store.getState().initialize();

    handler.current?.({
      paneId: "nonexistent-pane",
      sessionId: null,
      status: "exited",
      errorMessage: null,
    });

    expect(store.getState().workspace?.tabs).toHaveLength(1);
  });

  it("swapPanes calls transport with correct args", async () => {
    const transport = createTransportMock();
    const store = createWorkspaceStore(transport);

    await store.getState().initialize();
    await store.getState().swapPanes("pane-a", "pane-b");

    expect(transport.swapPanes).toHaveBeenCalledWith("pane-a", "pane-b");
  });

  it("applies focus and close transitions using transport responses", async () => {
    const singleLayout2: SplitNode = { type: "pane", paneId: "pane-2" };

    const focusedWorkspace: WorkspaceSnapshot = {
      ...workspace,
      tabs: [
        {
          ...workspace.tabs[0],
          activePaneId: "pane-1",
        },
      ],
    };

    const transport = createTransportMock({
      focusPane: vi.fn().mockResolvedValue(focusedWorkspace),
      closeTab: vi.fn().mockResolvedValue({
        activeTabId: "tab-2",
        tabs: [
          {
            id: "tab-2",
            title: "Workspace 2",
            layout: singleLayout2,
            activePaneId: "pane-2",
            panes: [
              {
                id: "pane-2",
                sessionId: "session-2",
                title: "Pane 1",
                cwd: "/tmp/next",
                profileId: "terminal",
                profileLabel: "Terminal",
                startupCommand: null,
                status: "running",
              },
            ],
          },
        ],
      }),
    });
    const store = createWorkspaceStore(transport);

    await store.getState().initialize();
    await store.getState().focusPane("tab-1", "pane-1");
    await store.getState().closeTab("tab-1");

    expect(transport.focusPane).toHaveBeenCalledWith("tab-1", "pane-1");
    expect(transport.closeTab).toHaveBeenCalledWith("tab-1");
    expect(store.getState().workspace?.activeTabId).toBe("tab-2");
  });
});
