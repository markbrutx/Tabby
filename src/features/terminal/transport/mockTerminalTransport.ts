import type {
  PtyOutputEvent,
  PtyResizeRequest,
} from "@/features/workspace/domain";
import type { UnlistenFn } from "@/lib/bridge/shared";
import type { TerminalTransport } from "./terminalTransport";

interface MockTerminalState {
  findPane: (paneId: string) => { tabIndex: number; paneIndex: number };
  getTabs: () => ReadonlyArray<{ panes: ReadonlyArray<{ id: string; sessionId: string; paneKind: string; cwd: string }> }>;
  updatePaneCwd: (paneId: string, cwd: string) => void;
  updateLastWorkingDirectory: (cwd: string) => void;
  emitMockOutput: (paneId: string, sessionId: string, text: string) => void;
  addOutputListener: (handler: (payload: PtyOutputEvent) => void) => UnlistenFn;
}

export function createMockTerminalTransport(state: MockTerminalState): TerminalTransport {
  return {
    async writePty(paneId: string, data: string): Promise<void> {
      const { tabIndex, paneIndex } = state.findPane(paneId);
      const pane = state.getTabs()[tabIndex].panes[paneIndex];

      if (pane.paneKind === "browser") return;

      if (data === "\r") {
        state.emitMockOutput(paneId, pane.sessionId, "\r\n\x1b[32m\u279c\x1b[0m  ");
      } else if (data === "\x7f") {
        state.emitMockOutput(paneId, pane.sessionId, "\b \b");
      } else {
        state.emitMockOutput(paneId, pane.sessionId, data);
      }
    },

    async resizePty(_request: PtyResizeRequest): Promise<void> {
      // no-op in mock
    },

    async listenToPtyOutput(
      handler: (payload: PtyOutputEvent) => void,
    ): Promise<UnlistenFn> {
      return state.addOutputListener(handler);
    },

    async trackPaneCwd(paneId: string, cwd: string): Promise<void> {
      state.updatePaneCwd(paneId, cwd);
      state.updateLastWorkingDirectory(cwd);
    },
  };
}
