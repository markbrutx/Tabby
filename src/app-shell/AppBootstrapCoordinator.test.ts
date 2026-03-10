import { describe, expect, it, vi } from "vitest";
import type { SettingsView, WorkspaceBootstrapView } from "@/contracts/tauri-bindings";
import type { WorkspaceClient, SettingsClient } from "@/app-shell/clients";
import { createSettingsStore } from "@/features/settings/application/store";
import { mapWorkspaceFromDto } from "@/features/workspace/application/snapshot-mappers";
import { mapSettingsFromDto, mapProfileFromDto } from "@/features/settings/application/snapshot-mappers";
import { mapRuntimeFromDto } from "@/features/runtime/application/snapshot-mappers";
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
        gitRepoPath: null,
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
      mapSettingsFromDto(payload.settings),
      payload.profileCatalog.terminalProfiles.map(mapProfileFromDto),
    );
    expect(loadRuntime).toHaveBeenCalledWith(payload.runtimeProjections.map(mapRuntimeFromDto));
    expect(loadWorkspace).toHaveBeenCalledWith(mapWorkspaceFromDto(payload.workspace));
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

// ---------------------------------------------------------------------------
// AC5: Onboarding flow through coordinator with real settings store
// ---------------------------------------------------------------------------

describe("AppBootstrapCoordinator + real SettingsStore integration", () => {
  function makeSettingsView(
    overrides?: Partial<SettingsView>,
  ): SettingsView {
    return {
      defaultLayout: "1x1",
      defaultTerminalProfileId: "terminal",
      defaultWorkingDirectory: "~",
      defaultCustomCommand: "",
      fontSize: 14,
      theme: "midnight",
      launchFullscreen: false,
      hasCompletedOnboarding: false,
      lastWorkingDirectory: null,
      ...overrides,
    };
  }

  function makeSettingsClient(
    overrides?: Partial<SettingsClient>,
  ): SettingsClient {
    return {
      dispatch: vi.fn().mockResolvedValue(
        makeSettingsView({ hasCompletedOnboarding: true }),
      ),
      listenProjectionUpdated: vi.fn().mockResolvedValue(() => {}),
      ...overrides,
    };
  }

  it("completeOnboarding through coordinator updates real settings store state", async () => {
    const settingsClient = makeSettingsClient();
    const settingsStore = createSettingsStore(settingsClient);

    // Bootstrap the real settings store with onboarding incomplete
    settingsStore
      .getState()
      .loadBootstrap(mapSettingsFromDto(makeSettingsView({ hasCompletedOnboarding: false })), []);

    expect(settingsStore.getState().settings?.hasCompletedOnboarding).toBe(false);

    // Wire up coordinator with real settings store
    const coordinator = createAppBootstrapCoordinator({
      workspaceClient: {
        bootstrap: vi.fn().mockResolvedValue(makeBootstrapPayload()),
        dispatch: vi.fn(),
        listenProjectionUpdated: vi.fn(),
      },
      workspaceStore: {
        getState: () => ({
          beginBootstrap: vi.fn(),
          loadBootstrap: vi.fn().mockResolvedValue(undefined),
          setBootstrapError: vi.fn(),
        }),
      },
      settingsStore,
      runtimeStore: {
        getState: () => ({
          loadBootstrap: vi.fn(),
        }),
      },
    });

    await coordinator.completeOnboarding();

    // Verify the real settings store was updated
    expect(settingsStore.getState().settings?.hasCompletedOnboarding).toBe(true);
    expect(settingsClient.dispatch).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: "update",
        settings: expect.objectContaining({ hasCompletedOnboarding: true }),
      }),
    );
  });

  it("completeOnboarding is idempotent — second call does not dispatch", async () => {
    const settingsClient = makeSettingsClient();
    const settingsStore = createSettingsStore(settingsClient);

    settingsStore
      .getState()
      .loadBootstrap(mapSettingsFromDto(makeSettingsView({ hasCompletedOnboarding: false })), []);

    const coordinator = createAppBootstrapCoordinator({
      workspaceClient: {
        bootstrap: vi.fn().mockResolvedValue(makeBootstrapPayload()),
        dispatch: vi.fn(),
        listenProjectionUpdated: vi.fn(),
      },
      workspaceStore: {
        getState: () => ({
          beginBootstrap: vi.fn(),
          loadBootstrap: vi.fn().mockResolvedValue(undefined),
          setBootstrapError: vi.fn(),
        }),
      },
      settingsStore,
      runtimeStore: {
        getState: () => ({
          loadBootstrap: vi.fn(),
        }),
      },
    });

    await coordinator.completeOnboarding();
    expect(settingsClient.dispatch).toHaveBeenCalledOnce();

    // Second call should be a no-op since onboarding is now complete
    await coordinator.completeOnboarding();
    expect(settingsClient.dispatch).toHaveBeenCalledOnce();
  });

  it("full bootstrap → onboarding flow updates settings store end-to-end", async () => {
    const settingsClient = makeSettingsClient();
    const settingsStore = createSettingsStore(settingsClient);

    const bootstrapPayload: WorkspaceBootstrapView = {
      ...makeBootstrapPayload(),
      settings: makeSettingsView({ hasCompletedOnboarding: false }),
    };

    const coordinator = createAppBootstrapCoordinator({
      workspaceClient: {
        bootstrap: vi.fn().mockResolvedValue(bootstrapPayload),
        dispatch: vi.fn(),
        listenProjectionUpdated: vi.fn(),
      },
      workspaceStore: {
        getState: () => ({
          beginBootstrap: vi.fn(),
          loadBootstrap: vi.fn().mockResolvedValue(undefined),
          setBootstrapError: vi.fn(),
        }),
      },
      settingsStore,
      runtimeStore: {
        getState: () => ({
          loadBootstrap: vi.fn(),
        }),
      },
    });

    // Step 1: Bootstrap distributes settings to real store
    await coordinator.initialize();

    expect(settingsStore.getState().settings).not.toBeNull();
    expect(settingsStore.getState().settings?.hasCompletedOnboarding).toBe(false);
    expect(settingsStore.getState().settings?.fontSize).toBe(14);
    expect(settingsStore.getState().settings?.theme).toBe("midnight");

    // Step 2: Complete onboarding through coordinator
    await coordinator.completeOnboarding();

    expect(settingsStore.getState().settings?.hasCompletedOnboarding).toBe(true);
  });
});
