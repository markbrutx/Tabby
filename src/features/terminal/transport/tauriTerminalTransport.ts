import { listen } from "@tauri-apps/api/event";
import type {
  PtyOutputEvent,
  PtyResizeRequest,
} from "@/features/workspace/domain";
import { commands } from "@/lib/tauri-bindings";
import { isTauriRuntime } from "@/lib/runtime";
import { ensureTauri, unwrapResult, type UnlistenFn } from "@/lib/bridge/shared";
import type { TerminalTransport } from "./terminalTransport";

const PTY_OUTPUT_EVENT_NAME = "pty-output";

export function createTauriTerminalTransport(): TerminalTransport {
  return {
    async writePty(paneId: string, data: string): Promise<void> {
      ensureTauri();
      unwrapResult(await commands.writePty(paneId, data));
    },

    async resizePty(request: PtyResizeRequest): Promise<void> {
      ensureTauri();
      unwrapResult(await commands.resizePty(request));
    },

    async listenToPtyOutput(
      handler: (payload: PtyOutputEvent) => void,
    ): Promise<UnlistenFn> {
      if (!isTauriRuntime()) {
        return () => undefined;
      }

      return listen<PtyOutputEvent>(PTY_OUTPUT_EVENT_NAME, (event) => {
        handler(event.payload);
      });
    },

    async trackPaneCwd(paneId: string, cwd: string): Promise<void> {
      ensureTauri();
      unwrapResult(await commands.trackPaneCwd(paneId, cwd));
    },
  };
}
