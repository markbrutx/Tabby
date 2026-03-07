import { describe, expect, it, vi } from "vitest";
import type {
  BootstrapSnapshot,
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
    listenToPtyOutput: vi.fn().mockResolvedValue(() => undefined),
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
