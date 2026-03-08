import { create } from "zustand";
import { asErrorMessage, shellClients } from "@/app-shell/clients";
import type { PaneProfile, SettingsCommandDto, WorkspaceSettings } from "@/features/workspace/domain";

interface SettingsState {
  settings: WorkspaceSettings | null;
  profiles: PaneProfile[];
  loadBootstrap: (settings: WorkspaceSettings, profiles: PaneProfile[]) => void;
  initializeListeners: () => Promise<void>;
  updateSettings: (settings: WorkspaceSettings) => Promise<void>;
  resetSettings: () => Promise<void>;
}

let settingsListenersReady: Promise<void> | null = null;

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: null,
  profiles: [],

  loadBootstrap(settings, profiles) {
    set({ settings, profiles });
    void get().initializeListeners();
  },

  async initializeListeners() {
    if (settingsListenersReady) {
      await settingsListenersReady;
      return;
    }

    settingsListenersReady = shellClients.settings
      .listenProjectionUpdated((payload) => {
        set({
          settings: payload.settings,
          profiles: payload.profileCatalog.terminalProfiles,
        });
      })
      .then(() => undefined);

    await settingsListenersReady;
  },

  async updateSettings(settings) {
    try {
      const next = await shellClients.settings.dispatch({
        kind: "update",
        settings,
      } satisfies SettingsCommandDto);
      set({ settings: next });
    } catch (error) {
      throw new Error(asErrorMessage(error));
    }
  },

  async resetSettings() {
    try {
      const next = await shellClients.settings.dispatch({
        kind: "reset",
      } satisfies SettingsCommandDto);
      set({ settings: next });
    } catch (error) {
      throw new Error(asErrorMessage(error));
    }
  },
}));
