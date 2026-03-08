import { create } from "zustand";
import { createStore } from "zustand/vanilla";
import type { PaneProfile, WorkspaceSettings } from "@/features/workspace/domain";
import { asErrorMessage, bridge, type WorkspaceTransport } from "@/lib/bridge";

interface SettingsStore {
  settings: WorkspaceSettings | null;
  profiles: PaneProfile[];
  loadSettings: (settings: WorkspaceSettings, profiles: PaneProfile[]) => void;
  updateSettings: (settings: WorkspaceSettings) => Promise<void>;
  resetSettings: () => Promise<void>;
}

type SetFn = (
  partial:
    | Partial<SettingsStore>
    | ((state: SettingsStore) => Partial<SettingsStore>),
) => void;

function createSettingsStoreState(transport: WorkspaceTransport) {
  return (set: SetFn): SettingsStore => ({
    settings: null,
    profiles: [],

    loadSettings(settings, profiles) {
      set({ settings, profiles });
    },

    async updateSettings(settings) {
      try {
        const updated = await transport.updateAppSettings(settings);
        set({ settings: updated });
      } catch (error) {
        throw new Error(asErrorMessage(error));
      }
    },

    async resetSettings() {
      try {
        const defaults = await transport.resetAppSettings();
        set({ settings: defaults });
      } catch (error) {
        throw new Error(asErrorMessage(error));
      }
    },
  });
}

export function createSettingsStore(transport: WorkspaceTransport = bridge) {
  return createStore<SettingsStore>(createSettingsStoreState(transport));
}

export const useSettingsStore = create<SettingsStore>(createSettingsStoreState(bridge));
