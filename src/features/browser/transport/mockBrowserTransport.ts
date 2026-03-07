import type { BrowserUrlChangedEvent } from "@/features/workspace/domain";
import type { BrowserBounds, UnlistenFn } from "@/lib/bridge/shared";
import type { BrowserTransport } from "./browserTransport";

export function createMockBrowserTransport(): BrowserTransport {
  return {
    async createBrowserWebview(
      _paneId: string,
      _url: string,
      _bounds: BrowserBounds,
    ): Promise<void> {
      // no-op in mock — iframe used instead
    },

    async navigateBrowser(_paneId: string, _url: string): Promise<void> {
      // no-op in mock
    },

    async closeBrowserWebview(_paneId: string): Promise<void> {
      // no-op in mock
    },

    async setBrowserWebviewBounds(
      _paneId: string,
      _bounds: BrowserBounds,
    ): Promise<void> {
      // no-op in mock
    },

    async setBrowserWebviewVisible(_paneId: string, _visible: boolean): Promise<void> {
      // no-op in mock
    },

    async listenToBrowserUrlChanged(
      _handler: (event: BrowserUrlChangedEvent) => void,
    ): Promise<UnlistenFn> {
      return () => undefined;
    },
  };
}
