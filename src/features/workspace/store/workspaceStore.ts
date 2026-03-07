import { create } from "zustand";
import { createStore } from "zustand/vanilla";
import {
  CUSTOM_PROFILE_ID,
  type LayoutPreset,
  type PaneProfile,
  type SplitPaneRequest,
  type UpdatePaneCwdRequest,
  type UpdatePaneProfileRequest,
  type WorkspaceSettings,
  type WorkspaceSnapshot,
} from "@/features/workspace/domain";
import { asErrorMessage, bridge, type WorkspaceTransport } from "@/lib/bridge";

interface CreateTabOverrides {
  cwd?: string;
  profileId?: string;
  startupCommand?: string;
}

export interface PaneGroupConfig {
  profileId: string;
  workingDirectory: string;
  customCommand?: string;
  count: number;
}

export interface SetupWizardConfig {
  groups: PaneGroupConfig[];
}

export interface WizardTab {
  id: string;
  title: string;
}

interface WorkspaceStore {
  workspace: WorkspaceSnapshot | null;
  settings: WorkspaceSettings | null;
  profiles: PaneProfile[];
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
  updateSettings: (settings: WorkspaceSettings) => Promise<void>;
  resetSettings: () => Promise<void>;
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
    set({ workspace, error: null, isWorking: false, ...extra });
  } catch (error) {
    set({ error: asErrorMessage(error), isWorking: false });
  }
}

async function runSettingsMutation(
  set: SetFn,
  mutation: () => Promise<WorkspaceSettings>,
) {
  set({ isWorking: true });

  try {
    const settings = await mutation();
    set({ settings, error: null, isWorking: false });
  } catch (error) {
    set({ error: asErrorMessage(error), isWorking: false });
  }
}

function createWorkspaceStoreState(transport: WorkspaceTransport) {
  return (set: SetFn, get: () => WorkspaceStore): WorkspaceStore => ({
    workspace: null,
    settings: null,
    profiles: [],
    error: null,
    isHydrating: true,
    isWorking: false,
    wizardTab: null,

    async initialize() {
      set({ isHydrating: true, error: null });

      try {
        const payload = await transport.bootstrapWorkspace();
        const shouldShowWizard =
          payload.workspace.tabs.length === 0;
        set({
          workspace: payload.workspace,
          settings: payload.settings,
          profiles: payload.profiles,
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
      const settings = get().settings;
      if (!settings) {
        return;
      }

      const cwd = overrides.cwd ?? settings.defaultWorkingDirectory;
      const profileId = overrides.profileId ?? settings.defaultProfileId;
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
        const paneConfigs = config.groups.flatMap((group) =>
          Array.from({ length: group.count }, () => ({
            profileId: group.profileId,
            cwd: group.workingDirectory,
            startupCommand: group.customCommand ?? null,
          })),
        );

        const workspace = await transport.createTab({
          preset: "1x1",
          cwd: null,
          profileId: null,
          startupCommand: null,
          paneConfigs,
        });

        const currentSettings = get().settings;
        const settingsUpdate =
          currentSettings && !currentSettings.hasCompletedOnboarding
            ? {
                settings: await transport.updateAppSettings({
                  ...currentSettings,
                  hasCompletedOnboarding: true,
                }),
              }
            : {};

        set({
          workspace,
          ...settingsUpdate,
          error: null,
          isWorking: false,
          wizardTab: null,
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
      await runWorkspaceMutation(
        set,
        () => transport.closeTab(tabId),
        (ws) => (ws.tabs.length === 0 ? { wizardTab: makeWizardTab(ws) } : {}),
      );
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
      await runWorkspaceMutation(
        set,
        () => transport.closePane(paneId),
        (ws) => (ws.tabs.length === 0 ? { wizardTab: makeWizardTab(ws) } : {}),
      );
    },

    async updateSettings(settings) {
      await runSettingsMutation(set, () => transport.updateAppSettings(settings));
    },

    async resetSettings() {
      await runSettingsMutation(set, () => transport.resetAppSettings());
    },

    clearError() {
      set({ error: null });
    },
  });
}

export function createWorkspaceStore(transport: WorkspaceTransport = bridge) {
  return createStore<WorkspaceStore>(createWorkspaceStoreState(transport));
}

export const useWorkspaceStore = create<WorkspaceStore>(createWorkspaceStoreState(bridge));
