import type { WorkspaceSettings } from "@/features/workspace/domain";
import type { SettingsTransport } from "./settingsTransport";

export interface MockSettingsState {
  getSettings: () => WorkspaceSettings;
  setSettings: (settings: WorkspaceSettings) => void;
  getDefaultSettings: () => WorkspaceSettings;
}

export function createMockSettingsTransport(state: MockSettingsState): SettingsTransport {
  return {
    async getAppSettings(): Promise<WorkspaceSettings> {
      return { ...state.getSettings() };
    },

    async updateAppSettings(
      settings: WorkspaceSettings,
    ): Promise<WorkspaceSettings> {
      state.setSettings({ ...settings });
      return { ...state.getSettings() };
    },

    async resetAppSettings(): Promise<WorkspaceSettings> {
      state.setSettings({ ...state.getDefaultSettings() });
      return { ...state.getSettings() };
    },
  };
}
