import type { WorkspaceSettings } from "@/features/workspace/domain";
import { commands } from "@/lib/tauri-bindings";
import { ensureTauri, unwrapResult } from "@/lib/bridge/shared";
import type { SettingsTransport } from "./settingsTransport";

export function createTauriSettingsTransport(): SettingsTransport {
  return {
    async getAppSettings(): Promise<WorkspaceSettings> {
      ensureTauri();
      return unwrapResult(await commands.getAppSettings());
    },

    async updateAppSettings(settings: WorkspaceSettings): Promise<WorkspaceSettings> {
      ensureTauri();
      return unwrapResult(await commands.updateAppSettings(settings));
    },

    async resetAppSettings(): Promise<WorkspaceSettings> {
      ensureTauri();
      return unwrapResult(await commands.resetAppSettings());
    },
  };
}
