import { isTauriRuntime } from "@/lib/runtime";
import { createMockTransport } from "./mockTransport";
import type { WorkspaceTransport } from "./shared";

declare global {
  interface Window {
    __TABBY_MOCK__?: WorkspaceTransport;
  }
}

let cachedMock: WorkspaceTransport | null = null;

export function getBrowserTransport(): WorkspaceTransport | null {
  if (typeof window === "undefined") {
    return null;
  }

  if (window.__TABBY_MOCK__) {
    return window.__TABBY_MOCK__;
  }

  if (isTauriRuntime()) {
    return null;
  }

  if (!cachedMock) {
    cachedMock = createMockTransport();
  }

  return cachedMock;
}
