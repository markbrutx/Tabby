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

export type UnlistenFn = () => void;

export interface BrowserBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

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
  listenToPaneLifecycle: (
    handler: (payload: PaneLifecycleEvent) => void,
  ) => Promise<UnlistenFn>;
  trackPaneCwd: (paneId: string, cwd: string) => Promise<void>;
  swapPanes: (paneIdA: string, paneIdB: string) => Promise<WorkspaceSnapshot>;
  createBrowserWebview: (
    paneId: string,
    url: string,
    bounds: BrowserBounds,
  ) => Promise<void>;
  navigateBrowser: (paneId: string, url: string) => Promise<void>;
  closeBrowserWebview: (paneId: string) => Promise<void>;
  setBrowserWebviewBounds: (
    paneId: string,
    bounds: BrowserBounds,
  ) => Promise<void>;
  setBrowserWebviewVisible: (paneId: string, visible: boolean) => Promise<void>;
  listenToBrowserUrlChanged: (
    handler: (event: BrowserUrlChangedEvent) => void,
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
