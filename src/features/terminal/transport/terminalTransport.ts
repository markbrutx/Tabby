import type {
  PtyOutputEvent,
  PtyResizeRequest,
} from "@/features/workspace/domain";
import type { UnlistenFn } from "@/lib/bridge/shared";

export interface TerminalTransport {
  writePty: (paneId: string, data: string) => Promise<void>;
  resizePty: (request: PtyResizeRequest) => Promise<void>;
  listenToPtyOutput: (
    handler: (payload: PtyOutputEvent) => void,
  ) => Promise<UnlistenFn>;
  trackPaneCwd: (paneId: string, cwd: string) => Promise<void>;
}
