import type { WorkspaceSettings } from "@/features/workspace/domain";
import { commands, type Result } from "@/lib/tauri-bindings";
import { isTauriRuntime } from "@/lib/runtime";
import { asErrorMessage } from "@/lib/bridge/shared";
import type { SettingsTransport } from "./settingsTransport";

function ensureTauri() {
  if (!isTauriRuntime()) {
    throw new Error("Live terminals are available only inside the Tauri shell.");
  }
}

function unwrapResult<T>(result: Result<T, unknown>): T {
  if (result.status === "ok") {
    return result.data;
  }

  throw new Error(asErrorMessage(result.error));
}

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
