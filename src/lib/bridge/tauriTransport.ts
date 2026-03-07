import { listen } from "@tauri-apps/api/event";
import type {
  BootstrapSnapshot,
  BrowserUrlChangedEvent,
  NewTabRequest,
  PaneLifecycleEvent,
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
import {
  asErrorMessage,
  type BrowserBounds,
  type UnlistenFn,
  type WorkspaceTransport,
} from "./shared";

const PTY_OUTPUT_EVENT_NAME = "pty-output";
const PANE_LIFECYCLE_EVENT_NAME = "pane-lifecycle";
const BROWSER_URL_CHANGED_EVENT = "browser-url-changed";

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

    async trackPaneCwd(paneId: string, cwd: string): Promise<void> {
      ensureTauri();
      unwrapResult(await commands.trackPaneCwd(paneId, cwd));
    },

    async swapPanes(paneIdA: string, paneIdB: string): Promise<WorkspaceSnapshot> {
      ensureTauri();
      return unwrapResult(await commands.swapPanes(paneIdA, paneIdB));
    },

    async listenToPaneLifecycle(
      handler: (payload: PaneLifecycleEvent) => void,
    ): Promise<UnlistenFn> {
      if (!isTauriRuntime()) {
        return () => undefined;
      }

      return listen<PaneLifecycleEvent>(PANE_LIFECYCLE_EVENT_NAME, (event) => {
        handler(event.payload);
      });
    },

    async createBrowserWebview(
      paneId: string,
      url: string,
      bounds: BrowserBounds,
    ): Promise<void> {
      ensureTauri();
      unwrapResult(
        await commands.createBrowserWebview(
          paneId,
          url,
          bounds.x,
          bounds.y,
          bounds.width,
          bounds.height,
        ),
      );
    },

    async navigateBrowser(paneId: string, url: string): Promise<void> {
      ensureTauri();
      unwrapResult(await commands.navigateBrowser(paneId, url));
    },

    async closeBrowserWebview(paneId: string): Promise<void> {
      ensureTauri();
      unwrapResult(await commands.closeBrowserWebview(paneId));
    },

    async setBrowserWebviewBounds(
      paneId: string,
      bounds: BrowserBounds,
    ): Promise<void> {
      ensureTauri();
      unwrapResult(
        await commands.setBrowserWebviewBounds(
          paneId,
          bounds.x,
          bounds.y,
          bounds.width,
          bounds.height,
        ),
      );
    },

    async setBrowserWebviewVisible(paneId: string, visible: boolean): Promise<void> {
      ensureTauri();
      unwrapResult(await commands.setBrowserWebviewVisible(paneId, visible));
    },

    async listenToBrowserUrlChanged(
      handler: (event: BrowserUrlChangedEvent) => void,
    ): Promise<UnlistenFn> {
      if (!isTauriRuntime()) {
        return () => undefined;
      }

      return listen<BrowserUrlChangedEvent>(BROWSER_URL_CHANGED_EVENT, (event) => {
        handler(event.payload);
      });
    },
  };
}
