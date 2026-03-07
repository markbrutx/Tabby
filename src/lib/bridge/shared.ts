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

export type UnlistenFn = () => void;

export interface WorkspaceTransport {
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
  writePty: (paneId: string, data: string) => Promise<void>;
  resizePty: (request: PtyResizeRequest) => Promise<void>;
  getAppSettings: () => Promise<WorkspaceSettings>;
  updateAppSettings: (settings: WorkspaceSettings) => Promise<WorkspaceSettings>;
  resetAppSettings: () => Promise<WorkspaceSettings>;
  listenToPtyOutput: (
    handler: (payload: PtyOutputEvent) => void,
  ) => Promise<UnlistenFn>;
}

export function asErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === "string") {
    return error;
  }

  if (error && typeof error === "object") {
    const [firstValue] = Object.values(error as Record<string, unknown>);
    if (typeof firstValue === "string") {
      return firstValue;
    }
  }

  return "Unknown error";
}
