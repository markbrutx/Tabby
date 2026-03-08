import { listen } from "@tauri-apps/api/event";
import type { BrowserUrlChangedEvent } from "@/features/workspace/domain";
import { commands } from "@/lib/tauri-bindings";
import { isTauriRuntime } from "@/lib/runtime";
import { ensureTauri, unwrapResult, type BrowserBounds, type UnlistenFn } from "@/lib/bridge/shared";
import type { BrowserTransport } from "./browserTransport";

const BROWSER_URL_CHANGED_EVENT = "browser-url-changed";

export function createTauriBrowserTransport(): BrowserTransport {
  return {
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
