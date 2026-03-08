import { create } from "zustand";
import { asErrorMessage } from "@/app-shell/clients";
import type { SettingsCommandDto, SettingsView } from "@/contracts/tauri-bindings";
import type {
  ProfileReadModel,
  SettingsReadModel,
} from "@/features/settings/domain/models";
import {
  mapProfileFromDto,
  mapSettingsFromDto,
} from "@/features/settings/application/snapshot-mappers";
import type { SettingsClient } from "@/app-shell/clients";

export interface SettingsState {
  settings: SettingsReadModel | null;
  profiles: ProfileReadModel[];
  loadBootstrap: (settings: SettingsView, profiles: readonly { id: string; label: string; description: string; startupCommandTemplate: string | null }[]) => void;
  initializeListeners: () => Promise<void>;
  updateSettings: (settings: SettingsReadModel) => Promise<void>;
  resetSettings: () => Promise<void>;
}

function toSettingsView(model: SettingsReadModel): SettingsView {
  return {
    defaultLayout: model.defaultLayout,
    defaultTerminalProfileId: model.defaultTerminalProfileId,
    defaultWorkingDirectory: model.defaultWorkingDirectory,
    defaultCustomCommand: model.defaultCustomCommand,
    fontSize: model.fontSize,
    theme: model.theme,
    launchFullscreen: model.launchFullscreen,
    hasCompletedOnboarding: model.hasCompletedOnboarding,
    lastWorkingDirectory: model.lastWorkingDirectory,
  };
}

export function createSettingsStore(settingsClient: SettingsClient) {
  let settingsListenersReady: Promise<void> | null = null;

  return create<SettingsState>((set, get) => ({
    settings: null,
    profiles: [],

    loadBootstrap(settings, profiles) {
      set({
        settings: mapSettingsFromDto(settings),
        profiles: profiles.map(mapProfileFromDto),
      });
      void get().initializeListeners();
    },

    async initializeListeners() {
      if (settingsListenersReady) {
        await settingsListenersReady;
        return;
      }

      settingsListenersReady = settingsClient
        .listenProjectionUpdated((payload) => {
          set({
            settings: mapSettingsFromDto(payload.settings),
            profiles: payload.profileCatalog.terminalProfiles.map(mapProfileFromDto),
          });
        })
        .then(() => undefined);

      await settingsListenersReady;
    },

    async updateSettings(settings) {
      try {
        const nextDto = await settingsClient.dispatch({
          kind: "update",
          settings: toSettingsView(settings),
        } satisfies SettingsCommandDto);
        set({ settings: mapSettingsFromDto(nextDto) });
      } catch (error) {
        throw new Error(asErrorMessage(error));
      }
    },

    async resetSettings() {
      try {
        const nextDto = await settingsClient.dispatch({
          kind: "reset",
        } satisfies SettingsCommandDto);
        set({ settings: mapSettingsFromDto(nextDto) });
      } catch (error) {
        throw new Error(asErrorMessage(error));
      }
    },
  }));
}
