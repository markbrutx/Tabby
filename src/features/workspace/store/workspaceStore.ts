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

interface WorkspaceStore {
  workspace: WorkspaceSnapshot | null;
  settings: WorkspaceSettings | null;
  profiles: PaneProfile[];
  error: string | null;
  isHydrating: boolean;
  isWorking: boolean;
  initialize: () => Promise<void>;
  createTab: (preset: LayoutPreset, overrides?: CreateTabOverrides) => Promise<void>;
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

async function runWorkspaceMutation(
  set: SetFn,
  mutation: () => Promise<WorkspaceSnapshot>,
) {
  set({ isWorking: true });

  try {
    const workspace = await mutation();
    set({ workspace, error: null, isWorking: false });
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

    async initialize() {
      set({ isHydrating: true, error: null });

      try {
        const payload = await transport.bootstrapWorkspace();
        set({
          workspace: payload.workspace,
          settings: payload.settings,
          profiles: payload.profiles,
          error: null,
          isHydrating: false,
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
