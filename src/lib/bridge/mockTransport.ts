import {
  createMockState,
  createMockWorkspaceTransport,
  MOCK_DEFAULT_SETTINGS,
} from "@/features/workspace/transport/mockWorkspaceTransport";
import { createMockTerminalTransport } from "@/features/terminal/transport/mockTerminalTransport";
import { createMockBrowserTransport } from "@/features/browser/transport/mockBrowserTransport";
import { createMockSettingsTransport } from "@/features/settings/transport/mockSettingsTransport";
import type { WorkspaceTransport } from "./shared";

export function createMockTransport(): WorkspaceTransport {
  const state = createMockState();

  const terminalState = {
    findPane(paneId: string) {
      for (let ti = 0; ti < state.tabs.length; ti++) {
        const pi = state.tabs[ti].panes.findIndex((p) => p.id === paneId);
        if (pi !== -1) {
          return { tabIndex: ti, paneIndex: pi };
        }
      }
      throw new Error(`Pane not found: ${paneId}`);
    },
    getTabs() {
      return state.tabs;
    },
    updatePaneCwd(paneId: string, cwd: string) {
      const { tabIndex, paneIndex } = terminalState.findPane(paneId);
      state.tabs = state.tabs.map((tab, ti) =>
        ti === tabIndex
          ? {
              ...tab,
              panes: tab.panes.map((pane, pi) =>
                pi === paneIndex ? { ...pane, cwd } : pane,
              ),
            }
          : tab,
      );
    },
    updateLastWorkingDirectory(cwd: string) {
      state.settings = { ...state.settings, lastWorkingDirectory: cwd };
    },
    emitMockOutput(paneId: string, sessionId: string, text: string) {
      const payload = { paneId, sessionId, chunk: text };
      for (const listener of state.outputListeners) {
        listener(payload);
      }
    },
    addOutputListener(handler: (payload: { paneId: string; sessionId: string; chunk: string }) => void) {
      state.outputListeners = [...state.outputListeners, handler];
      return () => {
        state.outputListeners = state.outputListeners.filter((h) => h !== handler);
      };
    },
  };

  const settingsState = {
    getSettings() {
      return state.settings;
    },
    setSettings(settings: typeof state.settings) {
      state.settings = settings;
    },
    getDefaultSettings() {
      return { ...MOCK_DEFAULT_SETTINGS };
    },
  };

  return {
    ...createMockWorkspaceTransport(state),
    ...createMockTerminalTransport(terminalState),
    ...createMockBrowserTransport(),
    ...createMockSettingsTransport(settingsState),
  };
}
