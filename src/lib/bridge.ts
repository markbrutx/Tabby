import { getBrowserTransport } from "@/lib/bridge/browserTransport";
import { createTauriTransport } from "@/lib/bridge/tauriTransport";
import { asErrorMessage, type WorkspaceTransport } from "@/lib/bridge/shared";

export const bridge: WorkspaceTransport =
  getBrowserTransport() ?? createTauriTransport();

export { asErrorMessage };
export type { BrowserBounds, WorkspaceTransport } from "@/lib/bridge/shared";
