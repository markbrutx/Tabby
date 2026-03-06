import { create } from "zustand";
import { createStore } from "zustand/vanilla";
import type {
  LayoutPreset,
  UpdatePaneCwdRequest,
  UpdatePaneProfileRequest,
  WorkspaceSettings,
  WorkspaceSnapshot,
} from "@/features/workspace/domain";
import { asErrorMessage, bridge, type WorkspaceTransport } from "@/lib/bridge";

interface CreateTabOverrides {
  cwd?: string;
  profileId?: string;
  startupCommand?: string;
}

export interface WorkspaceStore {
  workspace: WorkspaceSnapshot | null;
  settings: WorkspaceSettings | null;
  profiles: { id: string; label: string; description: string; startupCommand: string | null }[];
  error: string | null;
  isHydrating: boolean;
  isWorking: boolean;
  settingsOpen: boolean;
  initialize: () => Promise<void>;
  createTab: (preset: LayoutPreset, overrides?: CreateTabOverrides) => Promise<void>;
  closeTab: (tabId: string) => Promise<void>;
  setActiveTab: (tabId: string) => Promise<void>;
  focusPane: (tabId: string, paneId: string) => Promise<void>;
  updatePaneProfile: (request: UpdatePaneProfileRequest) => Promise<void>;
  updatePaneCwd: (request: UpdatePaneCwdRequest) => Promise<void>;
  restartPane: (paneId: string) => Promise<void>;
  updateSettings: (settings: WorkspaceSettings) => Promise<void>;
  setSettingsOpen: (value: boolean) => void;
  clearError: () => void;
}

async function runWorkspaceMutation(
  set: (
    partial:
      | Partial<WorkspaceStore>
      | ((state: WorkspaceStore) => Partial<WorkspaceStore>),
  ) => void,
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

function createWorkspaceStoreState(transport: WorkspaceTransport) {
  return (set: (partial:
      | Partial<WorkspaceStore>
      | ((state: WorkspaceStore) => Partial<WorkspaceStore>),
    ) => void, get: () => WorkspaceStore): WorkspaceStore => ({
    workspace: null,
    settings: null,
    profiles: [],
    error: null,
    isHydrating: true,
    isWorking: false,
    settingsOpen: false,

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
        (profileId === "custom" ? settings.defaultCustomCommand : "");

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

    async updateSettings(settings) {
      set({ isWorking: true });

      try {
        const nextSettings = await transport.updateAppSettings(settings);
        set({ settings: nextSettings, error: null, isWorking: false });
      } catch (error) {
        set({ error: asErrorMessage(error), isWorking: false });
      }
    },

    setSettingsOpen(value) {
      set({ settingsOpen: value });
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
