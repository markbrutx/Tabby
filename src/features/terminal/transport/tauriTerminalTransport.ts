import { listen } from "@tauri-apps/api/event";
import type {
  PtyOutputEvent,
  PtyResizeRequest,
} from "@/features/workspace/domain";
import { commands, type Result } from "@/lib/tauri-bindings";
import { isTauriRuntime } from "@/lib/runtime";
import { asErrorMessage, type UnlistenFn } from "@/lib/bridge/shared";
import type { TerminalTransport } from "./terminalTransport";

const PTY_OUTPUT_EVENT_NAME = "pty-output";

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
