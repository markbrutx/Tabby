import { getBrowserTransport } from "@/lib/bridge/browserTransport";
import { createTauriTransport } from "@/lib/bridge/tauriTransport";
import { asErrorMessage, type WorkspaceTransport } from "@/lib/bridge/shared";

export const bridge: WorkspaceTransport =
  getBrowserTransport() ?? createTauriTransport();

export { asErrorMessage };
export type { BrowserBounds, WorkspaceTransport } from "@/lib/bridge/shared";
export type { TerminalTransport } from "@/features/terminal/transport/terminalTransport";
export type { WorkspaceTransportInterface } from "@/features/workspace/transport/workspaceTransport";
export type { BrowserTransport } from "@/features/browser/transport/browserTransport";
export type { SettingsTransport } from "@/features/settings/transport/settingsTransport";
