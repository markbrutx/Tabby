import type { BrowserUrlChangedEvent } from "@/features/workspace/domain";
import type { BrowserBounds, UnlistenFn } from "@/lib/bridge/shared";
import type { MockState } from "@/features/workspace/transport/mockWorkspaceState";
import { findPane } from "@/features/workspace/transport/mockWorkspaceState";
import type { BrowserTransport } from "./browserTransport";

export function createMockBrowserTransport(state: MockState): BrowserTransport {
  return {
    async createBrowserWebview(
      _paneId: string,
      _url: string,
      _bounds: BrowserBounds,
    ): Promise<void> {
      // no-op in mock — iframe used instead
    },

    async navigateBrowser(paneId: string, url: string): Promise<void> {
      let tabIndex: number;
      let paneIndex: number;
      try {
        ({ tabIndex, paneIndex } = findPane(state, paneId));
      } catch {
        return;
      }

      state.tabs = state.tabs.map((tab, ti) =>
        ti === tabIndex
          ? {
              ...tab,
              panes: tab.panes.map((pane, pi) =>
                pi === paneIndex ? { ...pane, url } : pane,
              ),
            }
          : tab,
      );

      const payload: BrowserUrlChangedEvent = { paneId, url };
      for (const listener of state.browserUrlListeners) {
        listener(payload);
      }
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
      handler: (event: BrowserUrlChangedEvent) => void,
    ): Promise<UnlistenFn> {
      state.browserUrlListeners = [...state.browserUrlListeners, handler];
      return () => {
        state.browserUrlListeners = state.browserUrlListeners.filter(
          (listener) => listener !== handler,
        );
      };
    },
  };
}
