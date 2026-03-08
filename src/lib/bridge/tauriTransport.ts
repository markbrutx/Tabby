import { createTauriTerminalTransport } from "@/features/terminal/transport/tauriTerminalTransport";
import { createTauriWorkspaceTransport } from "@/features/workspace/transport/tauriWorkspaceTransport";
import { createTauriBrowserTransport } from "@/features/browser/transport/tauriBrowserTransport";
import { createTauriSettingsTransport } from "@/features/settings/transport/tauriSettingsTransport";
import type { WorkspaceTransport } from "./shared";

export function createTauriTransport(): WorkspaceTransport {
  return {
    ...createTauriTerminalTransport(),
    ...createTauriWorkspaceTransport(),
    ...createTauriBrowserTransport(),
    ...createTauriSettingsTransport(),
  };
}
