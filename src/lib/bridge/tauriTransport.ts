import { listen } from "@tauri-apps/api/event";
import type {
  BootstrapSnapshot,
  NewTabRequest,
  PtyOutputEvent,
  PtyResizeRequest,
  SplitPaneRequest,
  UpdatePaneCwdRequest,
  UpdatePaneProfileRequest,
  WorkspaceSettings,
  WorkspaceSnapshot,
} from "@/features/workspace/domain";
import { commands, type Result } from "@/lib/tauri-bindings";
import { isTauriRuntime } from "@/lib/runtime";
import { asErrorMessage, type UnlistenFn, type WorkspaceTransport } from "./shared";

const PTY_OUTPUT_EVENT_NAME = "pty-output";

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

export function createTauriTransport(): WorkspaceTransport {
  return {
    async bootstrapWorkspace(): Promise<BootstrapSnapshot> {
      ensureTauri();
      return unwrapResult(await commands.bootstrapWorkspace());
    },

    async createTab(request: NewTabRequest): Promise<WorkspaceSnapshot> {
      ensureTauri();
      return unwrapResult(await commands.createTab(request));
    },

    async closeTab(tabId: string): Promise<WorkspaceSnapshot> {
      ensureTauri();
      return unwrapResult(await commands.closeTab(tabId));
    },

    async setActiveTab(tabId: string): Promise<WorkspaceSnapshot> {
      ensureTauri();
      return unwrapResult(await commands.setActiveTab(tabId));
    },

    async focusPane(tabId: string, paneId: string): Promise<WorkspaceSnapshot> {
      ensureTauri();
      return unwrapResult(await commands.focusPane(tabId, paneId));
    },

    async updatePaneProfile(
      request: UpdatePaneProfileRequest,
    ): Promise<WorkspaceSnapshot> {
      ensureTauri();
      return unwrapResult(await commands.updatePaneProfile(request));
    },

    async updatePaneCwd(request: UpdatePaneCwdRequest): Promise<WorkspaceSnapshot> {
      ensureTauri();
      return unwrapResult(await commands.updatePaneCwd(request));
    },

    async restartPane(paneId: string): Promise<WorkspaceSnapshot> {
      ensureTauri();
      return unwrapResult(await commands.restartPane(paneId));
    },

    async splitPane(request: SplitPaneRequest): Promise<WorkspaceSnapshot> {
      ensureTauri();
      return unwrapResult(await commands.splitPane(request));
    },

    async closePane(paneId: string): Promise<WorkspaceSnapshot> {
      ensureTauri();
      return unwrapResult(await commands.closePane(paneId));
    },

    async writePty(paneId: string, data: string): Promise<void> {
      ensureTauri();
      unwrapResult(await commands.writePty(paneId, data));
    },

    async resizePty(request: PtyResizeRequest): Promise<void> {
      ensureTauri();
      unwrapResult(await commands.resizePty(request));
    },

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

    async listenToPtyOutput(
      handler: (payload: PtyOutputEvent) => void,
    ): Promise<UnlistenFn> {
      if (!isTauriRuntime()) {
        return () => undefined;
      }

      return listen<PtyOutputEvent>(PTY_OUTPUT_EVENT_NAME, (event) => {
        handler(event.payload);
      });
    },
  };
}
