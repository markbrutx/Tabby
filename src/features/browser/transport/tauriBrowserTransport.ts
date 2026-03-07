import { listen } from "@tauri-apps/api/event";
import type { BrowserUrlChangedEvent } from "@/features/workspace/domain";
import { commands, type Result } from "@/lib/tauri-bindings";
import { isTauriRuntime } from "@/lib/runtime";
import { asErrorMessage, type BrowserBounds, type UnlistenFn } from "@/lib/bridge/shared";
import type { BrowserTransport } from "./browserTransport";

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
