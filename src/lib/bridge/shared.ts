import type { TerminalTransport } from "@/features/terminal/transport/terminalTransport";
import type { WorkspaceTransportInterface } from "@/features/workspace/transport/workspaceTransport";
import type { BrowserTransport } from "@/features/browser/transport/browserTransport";
import type { SettingsTransport } from "@/features/settings/transport/settingsTransport";

export type UnlistenFn = () => void;

export interface BrowserBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export type WorkspaceTransport =
  TerminalTransport &
  WorkspaceTransportInterface &
  BrowserTransport &
  SettingsTransport;

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
