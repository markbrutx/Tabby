import { describe, expect, it, vi } from "vitest";
import type { WorkspaceBootstrapView } from "@/contracts/tauri-bindings";
import type { WorkspaceClient } from "@/app-shell/clients";
import {
  createAppBootstrapCoordinator,
  type AppBootstrapCoordinatorDeps,
  type BootstrapableWorkspaceStore,
  type BootstrapableSettingsStore,
  type BootstrapableRuntimeStore,
} from "./AppBootstrapCoordinator";

function makeBootstrapPayload(): WorkspaceBootstrapView {
  return {
    workspace: {
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
    },
    settings: {
      defaultLayout: "1x1",
      defaultTerminalProfileId: "terminal",
      defaultWorkingDirectory: "~",
      defaultCustomCommand: "",
      fontSize: 14,
      theme: "midnight",
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
    runtimeProjections: [
      {
        paneId: "p1",
        runtimeSessionId: "rs1",
        kind: "terminal",
        status: "running",
        lastError: null,
        browserLocation: null,
        terminalCwd: null,
      },
    ],
  };
}

function makeMockDeps(
  clientOverrides?: Partial<WorkspaceClient>,
): {
  deps: AppBootstrapCoordinatorDeps;
  workspaceStore: BootstrapableWorkspaceStore;
  settingsStore: BootstrapableSettingsStore;
  runtimeStore: BootstrapableRuntimeStore;
  workspaceClient: WorkspaceClient;
} {
  const workspaceClient: WorkspaceClient = {
    bootstrap: vi.fn().mockResolvedValue(makeBootstrapPayload()),
    dispatch: vi.fn(),
    listenProjectionUpdated: vi.fn(),
    ...clientOverrides,
  };

  const workspaceStore: BootstrapableWorkspaceStore = {
    getState: () => ({
      beginBootstrap: vi.fn(),
      loadBootstrap: vi.fn().mockResolvedValue(undefined),
      setBootstrapError: vi.fn(),
    }),
  };

  const settingsStore: BootstrapableSettingsStore = {
    getState: () => ({
      settings: null,
      loadBootstrap: vi.fn(),
      markOnboardingComplete: vi.fn().mockResolvedValue(undefined),
    }),
  };

  const runtimeStore: BootstrapableRuntimeStore = {
    getState: () => ({
      loadBootstrap: vi.fn(),
    }),
  };

  return {
    deps: {
      workspaceClient,
      workspaceStore,
      settingsStore,
      runtimeStore,
    },
    workspaceStore,
    settingsStore,
    runtimeStore,
    workspaceClient,
  };
}

describe("AppBootstrapCoordinator", () => {
  it("calls bootstrap and distributes data to all three stores", async () => {
    const beginBootstrap = vi.fn();
    const loadWorkspace = vi.fn().mockResolvedValue(undefined);
    const loadSettings = vi.fn();
    const loadRuntime = vi.fn();

    const payload = makeBootstrapPayload();
    const workspaceClient: WorkspaceClient = {
      bootstrap: vi.fn().mockResolvedValue(payload),
      dispatch: vi.fn(),
      listenProjectionUpdated: vi.fn(),
    };

    const coordinator = createAppBootstrapCoordinator({
      workspaceClient,
      workspaceStore: {
        getState: () => ({
          beginBootstrap,
          loadBootstrap: loadWorkspace,
          setBootstrapError: vi.fn(),
        }),
      },
      settingsStore: {
        getState: () => ({
          settings: null,
          loadBootstrap: loadSettings,
          markOnboardingComplete: vi.fn().mockResolvedValue(undefined),
        }),
      },
      runtimeStore: {
        getState: () => ({
          loadBootstrap: loadRuntime,
        }),
      },
    });

    await coordinator.initialize();

    expect(workspaceClient.bootstrap).toHaveBeenCalledOnce();
    expect(beginBootstrap).toHaveBeenCalledOnce();
    expect(loadSettings).toHaveBeenCalledWith(
      payload.settings,
      payload.profileCatalog.terminalProfiles,
    );
    expect(loadRuntime).toHaveBeenCalledWith(payload.runtimeProjections);
    expect(loadWorkspace).toHaveBeenCalledWith(payload);
  });

  it("calls setBootstrapError when bootstrap fails", async () => {
    const beginBootstrap = vi.fn();
    const setBootstrapError = vi.fn();
    const loadSettings = vi.fn();
    const loadRuntime = vi.fn();

    const coordinator = createAppBootstrapCoordinator({
      workspaceClient: {
        bootstrap: vi.fn().mockRejectedValue(new Error("network failure")),
        dispatch: vi.fn(),
        listenProjectionUpdated: vi.fn(),
      },
      workspaceStore: {
        getState: () => ({
          beginBootstrap,
          loadBootstrap: vi.fn(),
          setBootstrapError,
        }),
      },
      settingsStore: {
        getState: () => ({
          settings: null,
          loadBootstrap: loadSettings,
          markOnboardingComplete: vi.fn().mockResolvedValue(undefined),
        }),
      },
      runtimeStore: {
        getState: () => ({
          loadBootstrap: loadRuntime,
        }),
      },
    });

    await coordinator.initialize();

    expect(beginBootstrap).toHaveBeenCalledOnce();
    expect(setBootstrapError).toHaveBeenCalledWith("network failure");
    expect(loadSettings).not.toHaveBeenCalled();
    expect(loadRuntime).not.toHaveBeenCalled();
  });

  it("completeOnboarding updates settings through coordinator, not through workspace store", async () => {
    const markOnboardingComplete = vi.fn().mockResolvedValue(undefined);
    const settingsStore: BootstrapableSettingsStore = {
      getState: () => ({
        settings: { hasCompletedOnboarding: false },
        loadBootstrap: vi.fn(),
        markOnboardingComplete,
      }),
    };

    const { deps } = makeMockDeps();
    const coordinator = createAppBootstrapCoordinator({
      ...deps,
      settingsStore,
    });

    await coordinator.completeOnboarding();

    expect(markOnboardingComplete).toHaveBeenCalledOnce();
  });

  it("completeOnboarding is a no-op when onboarding is already complete", async () => {
    const markOnboardingComplete = vi.fn().mockResolvedValue(undefined);
    const settingsStore: BootstrapableSettingsStore = {
      getState: () => ({
        settings: { hasCompletedOnboarding: true },
        loadBootstrap: vi.fn(),
        markOnboardingComplete,
      }),
    };

    const { deps } = makeMockDeps();
    const coordinator = createAppBootstrapCoordinator({
      ...deps,
      settingsStore,
    });

    await coordinator.completeOnboarding();

    expect(markOnboardingComplete).not.toHaveBeenCalled();
  });

  it("completeOnboarding is a no-op when settings are null", async () => {
    const markOnboardingComplete = vi.fn().mockResolvedValue(undefined);
    const settingsStore: BootstrapableSettingsStore = {
      getState: () => ({
        settings: null,
        loadBootstrap: vi.fn(),
        markOnboardingComplete,
      }),
    };

    const { deps } = makeMockDeps();
    const coordinator = createAppBootstrapCoordinator({
      ...deps,
      settingsStore,
    });

    await coordinator.completeOnboarding();

    expect(markOnboardingComplete).not.toHaveBeenCalled();
  });

  it("distributes settings before runtime before workspace", async () => {
    const callOrder: string[] = [];
    const payload = makeBootstrapPayload();

    const coordinator = createAppBootstrapCoordinator({
      workspaceClient: {
        bootstrap: vi.fn().mockResolvedValue(payload),
        dispatch: vi.fn(),
        listenProjectionUpdated: vi.fn(),
      },
      workspaceStore: {
        getState: () => ({
          beginBootstrap: vi.fn(),
          loadBootstrap: vi.fn().mockImplementation(() => {
            callOrder.push("workspace");
            return Promise.resolve();
          }),
          setBootstrapError: vi.fn(),
        }),
      },
      settingsStore: {
        getState: () => ({
          settings: null,
          loadBootstrap: vi.fn().mockImplementation(() => {
            callOrder.push("settings");
          }),
          markOnboardingComplete: vi.fn().mockResolvedValue(undefined),
        }),
      },
      runtimeStore: {
        getState: () => ({
          loadBootstrap: vi.fn().mockImplementation(() => {
            callOrder.push("runtime");
          }),
        }),
      },
    });

    await coordinator.initialize();

    expect(callOrder).toEqual(["settings", "runtime", "workspace"]);
  });
});
