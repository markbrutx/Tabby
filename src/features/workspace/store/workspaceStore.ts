import { create } from "zustand";
import { createStore } from "zustand/vanilla";
import {
  BROWSER_PROFILE_ID,
  CUSTOM_PROFILE_ID,
  type LayoutPreset,
  type PaneLifecycleEvent,
  type PaneProfile,
  type SplitPaneRequest,
  type UpdatePaneCwdRequest,
  type UpdatePaneProfileRequest,
  type WorkspaceSettings,
  type WorkspaceSnapshot,
} from "@/features/workspace/domain";
import { asErrorMessage, bridge, type WorkspaceTransport } from "@/lib/bridge";
import { useSettingsStore, createSettingsStore } from "@/features/settings/store/settingsStore";

import type { SetupWizardConfig, WizardTab } from "./types";

interface CreateTabOverrides {
  cwd?: string;
  profileId?: string;
  startupCommand?: string;
}

interface WorkspaceStore {
  workspace: WorkspaceSnapshot | null;
  error: string | null;
  isHydrating: boolean;
  isWorking: boolean;
  wizardTab: WizardTab | null;
  initialize: () => Promise<void>;
  createTab: (preset: LayoutPreset, overrides?: CreateTabOverrides) => Promise<void>;
  createTabFromWizard: (config: SetupWizardConfig) => Promise<void>;
  openSetupWizard: () => void;
  closeSetupWizard: () => void;
  closeTab: (tabId: string) => Promise<void>;
  setActiveTab: (tabId: string) => Promise<void>;
  focusPane: (tabId: string, paneId: string) => Promise<void>;
  updatePaneProfile: (request: UpdatePaneProfileRequest) => Promise<void>;
  updatePaneCwd: (request: UpdatePaneCwdRequest) => Promise<void>;
  restartPane: (paneId: string) => Promise<void>;
  splitPane: (request: SplitPaneRequest) => Promise<void>;
  closePane: (paneId: string) => Promise<void>;
  swapPanes: (paneIdA: string, paneIdB: string) => Promise<void>;
  clearError: () => void;
}

type SetFn = (
  partial:
    | Partial<WorkspaceStore>
    | ((state: WorkspaceStore) => Partial<WorkspaceStore>),
) => void;

function makeWizardTab(workspace?: WorkspaceSnapshot | null): WizardTab {
  const nextIndex = (workspace?.tabs.length ?? 0) + 1;
  return {
    id: `__wizard_${Date.now()}__`,
    title: `Workspace ${nextIndex}`,
  };
}

async function runWorkspaceMutation(
  set: SetFn,
  mutation: () => Promise<WorkspaceSnapshot>,
  onSuccess?: (workspace: WorkspaceSnapshot) => Partial<WorkspaceStore>,
) {
  set({ isWorking: true });

  try {
    const workspace = await mutation();
    const extra = onSuccess?.(workspace) ?? {};
    set({
      workspace,
      error: null,
      isWorking: false,
      wizardTab: workspace.tabs.length === 0 ? makeWizardTab(workspace) : null,
      ...extra,
    });
  } catch (error) {
    set({ error: asErrorMessage(error), isWorking: false });
  }
}

interface SettingsAccessor {
  loadSettings: (settings: WorkspaceSettings, profiles: PaneProfile[]) => void;
  getSettings: () => WorkspaceSettings | null;
  getProfiles: () => PaneProfile[];
  updateSettings: (settings: WorkspaceSettings) => Promise<void>;
}

function createWorkspaceStoreState(
  transport: WorkspaceTransport,
  settingsStoreAccessor: () => SettingsAccessor,
) {
  return (set: SetFn, get: () => WorkspaceStore): WorkspaceStore => {
    let paneLifecycleUnlisten: (() => void) | null = null;
    let paneLifecycleInitPromise: Promise<void> | null = null;
    let workspaceChangedUnlisten: (() => void) | null = null;
    let workspaceChangedInitPromise: Promise<void> | null = null;
    let browserUrlUnlisten: (() => void) | null = null;
    let browserUrlInitPromise: Promise<void> | null = null;

    const handlePaneLifecycle = (event: PaneLifecycleEvent) => {
      set((state) => {
        const workspace = state.workspace;
        if (!workspace) {
          return {};
        }

        const tabs = workspace.tabs.map((tab) => ({
          ...tab,
          panes: tab.panes.map((pane) => {
            if (pane.id !== event.paneId) {
              return pane;
            }
            if (event.sessionId && pane.sessionId !== event.sessionId) {
              return pane;
            }
            return { ...pane, status: event.status };
          }),
        }));

        return { workspace: { ...workspace, tabs } };
      });
    };

    const handleWorkspaceChanged = (workspace: WorkspaceSnapshot) => {
      set({
        workspace,
        error: null,
        wizardTab: workspace.tabs.length === 0 ? makeWizardTab(workspace) : null,
      });
    };

    const handleBrowserUrlChanged = (event: { paneId: string; url: string }) => {
      set((state) => {
        const workspace = state.workspace;
        if (!workspace) {
          return {};
        }

        let changed = false;
        const tabs = workspace.tabs.map((tab) => ({
          ...tab,
          panes: tab.panes.map((pane) => {
            if (pane.id !== event.paneId || pane.url === event.url) {
              return pane;
            }

            changed = true;
            return { ...pane, url: event.url };
          }),
        }));

        if (!changed) {
          return {};
        }

        return { workspace: { ...workspace, tabs } };
      });
    };

    const ensurePaneLifecycleListener = async () => {
      if (paneLifecycleUnlisten || paneLifecycleInitPromise) {
        await paneLifecycleInitPromise;
        return;
      }

      paneLifecycleInitPromise = transport
        .listenToPaneLifecycle(handlePaneLifecycle)
        .then((unlisten) => {
          paneLifecycleUnlisten = unlisten;
        })
        .finally(() => {
          paneLifecycleInitPromise = null;
        });

      await paneLifecycleInitPromise;
    };

    const ensureWorkspaceChangedListener = async () => {
      if (workspaceChangedUnlisten || workspaceChangedInitPromise) {
        await workspaceChangedInitPromise;
        return;
      }

      workspaceChangedInitPromise = transport
        .listenToWorkspaceChanged(handleWorkspaceChanged)
        .then((unlisten) => {
          workspaceChangedUnlisten = unlisten;
        })
        .finally(() => {
          workspaceChangedInitPromise = null;
        });

      await workspaceChangedInitPromise;
    };

    const ensureBrowserUrlListener = async () => {
      if (browserUrlUnlisten || browserUrlInitPromise) {
        await browserUrlInitPromise;
        return;
      }

      browserUrlInitPromise = transport
        .listenToBrowserUrlChanged(handleBrowserUrlChanged)
        .then((unlisten) => {
          browserUrlUnlisten = unlisten;
        })
        .finally(() => {
          browserUrlInitPromise = null;
        });

      await browserUrlInitPromise;
    };

    return {
      workspace: null,
      error: null,
      isHydrating: true,
      isWorking: false,
      wizardTab: null,

      async initialize() {
        set({ isHydrating: true, error: null });

        try {
          const payload = await transport.bootstrapWorkspace();
          await Promise.all([
            ensurePaneLifecycleListener(),
            ensureWorkspaceChangedListener(),
            ensureBrowserUrlListener(),
          ]);
          settingsStoreAccessor().loadSettings(payload.settings, payload.profiles);
          const shouldShowWizard = payload.workspace.tabs.length === 0;
          set({
            workspace: payload.workspace,
            error: null,
            isHydrating: false,
            wizardTab: shouldShowWizard
              ? makeWizardTab(payload.workspace)
              : null,
          });
        } catch (error) {
          set({
            error: asErrorMessage(error),
            isHydrating: false,
          });
        }
      },

      async createTab(preset, overrides = {}) {
        const settings = settingsStoreAccessor().getSettings();
        if (!settings) {
          return;
        }

        const cwd = overrides.cwd ?? settings.defaultWorkingDirectory;
        const profileId = overrides.profileId || settings.defaultProfileId || "terminal";
        const startupCommand =
          overrides.startupCommand ??
          (profileId === CUSTOM_PROFILE_ID ? settings.defaultCustomCommand : "");

        await runWorkspaceMutation(set, () =>
          transport.createTab({
            preset,
            cwd,
            profileId,
            startupCommand: startupCommand || null,
          }),
        );
      },

      async createTabFromWizard(config) {
        set({ isWorking: true });

        try {
          const currentSettings = settingsStoreAccessor().getSettings();
          const defaultProfileId = currentSettings?.defaultProfileId || "terminal";
          const paneConfigs = config.groups.flatMap((group) => {
            const profileId = group.profileId || defaultProfileId;
            const customCommand = group.customCommand?.trim() ?? "";
            if (profileId === CUSTOM_PROFILE_ID && !customCommand) {
              throw new Error("Custom profile requires a startup command");
            }

            const isBrowser = profileId === BROWSER_PROFILE_ID;
            return Array.from({ length: group.count }, () => ({
              profileId,
              cwd: isBrowser ? (group.workingDirectory || "~") : group.workingDirectory,
              startupCommand: profileId === CUSTOM_PROFILE_ID ? customCommand : null,
              url: isBrowser ? (group.url || null) : null,
            }));
          });

          const workspace = await transport.createTab({
            preset: "1x1",
            cwd: null,
            profileId: null,
            startupCommand: null,
            paneConfigs,
          });

          if (currentSettings && !currentSettings.hasCompletedOnboarding) {
            await settingsStoreAccessor().updateSettings({
              ...currentSettings,
              hasCompletedOnboarding: true,
            });
          }

          set({
            workspace,
            error: null,
            isWorking: false,
            wizardTab: workspace.tabs.length === 0 ? makeWizardTab(workspace) : null,
          });
        } catch (error) {
          set({ error: asErrorMessage(error), isWorking: false });
        }
      },

      openSetupWizard() {
        set({ wizardTab: makeWizardTab(get().workspace) });
      },

      closeSetupWizard() {
        const workspace = get().workspace;
        if (workspace && workspace.tabs.length === 0) {
          return;
        }
        set({ wizardTab: null });
      },

      async closeTab(tabId) {
        await runWorkspaceMutation(set, () => transport.closeTab(tabId));
      },

      async setActiveTab(tabId) {
        await runWorkspaceMutation(set, () => transport.setActiveTab(tabId));
      },

      async focusPane(tabId, paneId) {
        await runWorkspaceMutation(set, () => transport.focusPane(tabId, paneId));
      },

      async updatePaneProfile(request) {
        await runWorkspaceMutation(set, () => transport.updatePaneProfile(request));
      },

      async updatePaneCwd(request) {
        await runWorkspaceMutation(set, () => transport.updatePaneCwd(request));
      },

      async restartPane(paneId) {
        await runWorkspaceMutation(set, () => transport.restartPane(paneId));
      },

      async splitPane(request) {
        await runWorkspaceMutation(set, () => transport.splitPane(request));
      },

      async closePane(paneId) {
        await runWorkspaceMutation(set, () => transport.closePane(paneId));
      },

      async swapPanes(paneIdA, paneIdB) {
        await runWorkspaceMutation(set, () => transport.swapPanes(paneIdA, paneIdB));
      },

      clearError() {
        set({ error: null });
      },
    };
  };
}

export function createWorkspaceStore(transport: WorkspaceTransport = bridge) {
  const settingsStore = createSettingsStore(transport);
  const workspaceStore = createStore<WorkspaceStore>(createWorkspaceStoreState(transport, () => ({
    loadSettings: settingsStore.getState().loadSettings,
    getSettings: () => settingsStore.getState().settings,
    getProfiles: () => settingsStore.getState().profiles,
    updateSettings: settingsStore.getState().updateSettings,
  })));
  return Object.assign(workspaceStore, { settingsStore });
}

export const useWorkspaceStore = create<WorkspaceStore>(
  createWorkspaceStoreState(bridge, () => ({
    loadSettings: useSettingsStore.getState().loadSettings,
    getSettings: () => useSettingsStore.getState().settings,
    getProfiles: () => useSettingsStore.getState().profiles,
    updateSettings: useSettingsStore.getState().updateSettings,
  })),
);
