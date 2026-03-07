import { listen } from "@tauri-apps/api/event";
import type {
  BootstrapSnapshot,
  NewTabRequest,
  PaneLifecycleEvent,
  SplitPaneRequest,
  UpdatePaneCwdRequest,
  UpdatePaneProfileRequest,
  WorkspaceSnapshot,
} from "@/features/workspace/domain";
import { commands } from "@/lib/tauri-bindings";
import { isTauriRuntime } from "@/lib/runtime";
import { ensureTauri, unwrapResult, type UnlistenFn } from "@/lib/bridge/shared";
import type { WorkspaceTransportInterface } from "./workspaceTransport";

const PANE_LIFECYCLE_EVENT_NAME = "pane-lifecycle";

export function createTauriWorkspaceTransport(): WorkspaceTransportInterface {
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
  };
}
