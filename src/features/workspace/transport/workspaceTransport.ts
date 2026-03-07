import type {
  BootstrapSnapshot,
  NewTabRequest,
  PaneLifecycleEvent,
  SplitPaneRequest,
  UpdatePaneCwdRequest,
  UpdatePaneProfileRequest,
  WorkspaceSnapshot,
} from "@/features/workspace/domain";
import type { UnlistenFn } from "@/lib/bridge/shared";

export interface WorkspaceTransportInterface {
  bootstrapWorkspace: () => Promise<BootstrapSnapshot>;
  createTab: (request: NewTabRequest) => Promise<WorkspaceSnapshot>;
  closeTab: (tabId: string) => Promise<WorkspaceSnapshot>;
  setActiveTab: (tabId: string) => Promise<WorkspaceSnapshot>;
  focusPane: (tabId: string, paneId: string) => Promise<WorkspaceSnapshot>;
  updatePaneProfile: (
    request: UpdatePaneProfileRequest,
  ) => Promise<WorkspaceSnapshot>;
  updatePaneCwd: (request: UpdatePaneCwdRequest) => Promise<WorkspaceSnapshot>;
  restartPane: (paneId: string) => Promise<WorkspaceSnapshot>;
  splitPane: (request: SplitPaneRequest) => Promise<WorkspaceSnapshot>;
  closePane: (paneId: string) => Promise<WorkspaceSnapshot>;
  swapPanes: (paneIdA: string, paneIdB: string) => Promise<WorkspaceSnapshot>;
  listenToPaneLifecycle: (
    handler: (payload: PaneLifecycleEvent) => void,
  ) => Promise<UnlistenFn>;
}
