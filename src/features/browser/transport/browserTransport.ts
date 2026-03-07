import type { BrowserUrlChangedEvent } from "@/features/workspace/domain";
import type { BrowserBounds, UnlistenFn } from "@/lib/bridge/shared";

export interface BrowserTransport {
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
