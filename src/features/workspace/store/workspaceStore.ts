import { create } from "zustand";
import type {
  LayoutPreset,
  UpdatePaneCwdRequest,
  UpdatePaneProfileRequest,
  WorkspaceSettings,
  WorkspaceSnapshot,
} from "@/features/workspace/domain";
import { bridge, asErrorMessage } from "@/lib/bridge";

interface CreateTabOverrides {
  cwd?: string;
  profileId?: string;
  startupCommand?: string;
}

interface WorkspaceStore {
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

export const useWorkspaceStore = create<WorkspaceStore>((set, get) => ({
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
      const payload = await bridge.bootstrapWorkspace();
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
      bridge.createTab({
        preset,
        cwd,
        profileId,
        startupCommand: startupCommand || null,
      }),
    );
  },

  async closeTab(tabId) {
    await runWorkspaceMutation(set, () => bridge.closeTab(tabId));
  },

  async setActiveTab(tabId) {
    await runWorkspaceMutation(set, () => bridge.setActiveTab(tabId));
  },

  async focusPane(tabId, paneId) {
    await runWorkspaceMutation(set, () => bridge.focusPane(tabId, paneId));
  },

  async updatePaneProfile(request) {
    await runWorkspaceMutation(set, () => bridge.updatePaneProfile(request));
  },

  async updatePaneCwd(request) {
    await runWorkspaceMutation(set, () => bridge.updatePaneCwd(request));
  },

  async restartPane(paneId) {
    await runWorkspaceMutation(set, () => bridge.restartPane(paneId));
  },

  async updateSettings(settings) {
    set({ isWorking: true });

    try {
      const nextSettings = await bridge.updateAppSettings(settings);
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
}));
