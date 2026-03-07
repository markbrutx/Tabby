import {
  CUSTOM_PROFILE_ID,
  type BootstrapSnapshot,
  type LayoutPreset,
  type NewTabRequest,
  type PaneProfile,
  type PaneSnapshot,
  type PtyOutputEvent,
  type PtyResizeRequest,
  type SplitDirection,
  type SplitNode,
  type SplitPaneRequest,
  type TabSnapshot,
  type UpdatePaneCwdRequest,
  type UpdatePaneProfileRequest,
  type WorkspaceSettings,
  type WorkspaceSnapshot,
} from "@/features/workspace/domain";
import {
  splitPane as treeSplitPane,
  closePane as treeClosePane,
  treeFromPreset,
  paneCountForPreset,
} from "@/features/workspace/splitTree";
import type { UnlistenFn, WorkspaceTransport } from "./shared";

const MOCK_DEFAULT_SETTINGS: WorkspaceSettings = {
  defaultLayout: "1x1",
  defaultProfileId: "terminal",
  defaultWorkingDirectory: "~",
  defaultCustomCommand: "",
  fontSize: 13,
  theme: "midnight",
  launchFullscreen: false,
  hasCompletedOnboarding: true,
};

let idCounter = 0;
function nextId(prefix: string): string {
  idCounter += 1;
  return `${prefix}-${idCounter}`;
}

const BUILT_IN_PROFILES: PaneProfile[] = [
  {
    id: "terminal",
    label: "Terminal",
    description: "Pure login shell",
    startupCommand: null,
  },
  {
    id: "claude",
    label: "Claude Code",
    description: "Open Claude Code in a fresh shell",
    startupCommand: "claude",
  },
  {
    id: "codex",
    label: "Codex",
    description: "Open Codex in a fresh shell",
    startupCommand: "codex",
  },
  {
    id: CUSTOM_PROFILE_ID,
    label: "Custom",
    description: "Run an arbitrary shell command",
    startupCommand: null,
  },
];

function resolveProfile(
  profileId: string,
  startupCommand: string | null,
): { id: string; label: string; startupCommand: string | null } {
  const profile = BUILT_IN_PROFILES.find((p) => p.id === profileId);
  if (!profile) {
    return { id: profileId, label: profileId, startupCommand };
  }

  return {
    id: profile.id,
    label: profile.label,
    startupCommand:
      profile.id === CUSTOM_PROFILE_ID ? startupCommand : profile.startupCommand,
  };
}

function createPane(
  cwd: string,
  profileId: string,
  startupCommand: string | null,
  index: number,
): PaneSnapshot {
  const resolved = resolveProfile(profileId, startupCommand);
  return {
    id: nextId("pane"),
    sessionId: nextId("session"),
    title: `Pane ${index + 1}`,
    cwd,
    profileId: resolved.id,
    profileLabel: resolved.label,
    startupCommand: resolved.startupCommand,
    status: "running",
  };
}

function createTab(
  preset: LayoutPreset,
  cwd: string,
  profileId: string,
  startupCommand: string | null,
  tabIndex: number,
): TabSnapshot {
  const paneCount = paneCountForPreset(preset);
  const panes: PaneSnapshot[] = [];

  for (let i = 0; i < paneCount; i++) {
    panes.push(createPane(cwd, profileId, startupCommand, i));
  }

  const paneIds = panes.map((p) => p.id);
  const layout = treeFromPreset(preset, paneIds);

  return {
    id: nextId("tab"),
    title: `Workspace ${tabIndex}`,
    layout,
    panes,
    activePaneId: panes[0]?.id ?? "",
  };
}

interface MockState {
  tabs: TabSnapshot[];
  activeTabId: string;
  settings: WorkspaceSettings;
  nextTabIndex: number;
  outputListeners: Array<(payload: PtyOutputEvent) => void>;
}

function snapshot(state: MockState): WorkspaceSnapshot {
  return {
    activeTabId: state.activeTabId,
    tabs: state.tabs,
  };
}

function findTabIndex(state: MockState, tabId: string): number {
  const index = state.tabs.findIndex((t) => t.id === tabId);
  if (index === -1) {
    throw new Error(`Tab not found: ${tabId}`);
  }
  return index;
}

function findPane(
  state: MockState,
  paneId: string,
): { tabIndex: number; paneIndex: number } {
  for (let ti = 0; ti < state.tabs.length; ti++) {
    const pi = state.tabs[ti].panes.findIndex((p) => p.id === paneId);
    if (pi !== -1) {
      return { tabIndex: ti, paneIndex: pi };
    }
  }
  throw new Error(`Pane not found: ${paneId}`);
}

function emitMockOutput(state: MockState, paneId: string, sessionId: string, text: string) {
  const payload: PtyOutputEvent = { paneId, sessionId, chunk: text };
  for (const listener of state.outputListeners) {
    listener(payload);
  }
}

export function createMockTransport(): WorkspaceTransport {
  const state: MockState = {
    tabs: [],
    activeTabId: "",
    settings: { ...MOCK_DEFAULT_SETTINGS },
    nextTabIndex: 1,
    outputListeners: [],
  };

  function addTab(
    preset: LayoutPreset,
    cwd: string,
    profileId: string,
    startupCommand: string | null,
  ): WorkspaceSnapshot {
    const tab = createTab(preset, cwd, profileId, startupCommand, state.nextTabIndex);
    state.tabs = [...state.tabs, tab];
    state.activeTabId = tab.id;
    state.nextTabIndex += 1;

    for (const pane of tab.panes) {
      setTimeout(() => {
        emitMockOutput(
          state,
          pane.id,
          pane.sessionId,
          `\x1b[36m${pane.profileLabel}\x1b[0m session started in \x1b[33m${pane.cwd}\x1b[0m\r\n\r\n` +
          `\x1b[90m[mock] Type here — keystrokes are echoed locally.\x1b[0m\r\n` +
          `\x1b[32m➜\x1b[0m  `,
        );
      }, 80);
    }

    return snapshot(state);
  }

  return {
    async bootstrapWorkspace(): Promise<BootstrapSnapshot> {
      if (state.tabs.length === 0) {
        const workspace = addTab(
          state.settings.defaultLayout,
          state.settings.defaultWorkingDirectory,
          state.settings.defaultProfileId,
          null,
        );

        return {
          workspace,
          settings: { ...state.settings },
          profiles: [...BUILT_IN_PROFILES],
        };
      }

      return {
        workspace: snapshot(state),
        settings: { ...state.settings },
        profiles: [...BUILT_IN_PROFILES],
      };
    },

    async createTab(request: NewTabRequest): Promise<WorkspaceSnapshot> {
      return addTab(
        request.preset,
        request.cwd ?? state.settings.defaultWorkingDirectory,
        request.profileId ?? state.settings.defaultProfileId,
        request.startupCommand ?? null,
      );
    },

    async closeTab(tabId: string): Promise<WorkspaceSnapshot> {
      const index = findTabIndex(state, tabId);
      state.tabs = state.tabs.filter((_, i) => i !== index);

      if (state.tabs.length === 0) {
        return addTab(
          state.settings.defaultLayout,
          state.settings.defaultWorkingDirectory,
          state.settings.defaultProfileId,
          null,
        );
      }

      if (state.activeTabId === tabId) {
        state.activeTabId = state.tabs[Math.max(0, index - 1)].id;
      }

      return snapshot(state);
    },

    async setActiveTab(tabId: string): Promise<WorkspaceSnapshot> {
      findTabIndex(state, tabId);
      state.activeTabId = tabId;
      return snapshot(state);
    },

    async focusPane(tabId: string, paneId: string): Promise<WorkspaceSnapshot> {
      const ti = findTabIndex(state, tabId);
      const tab = state.tabs[ti];
      if (!tab.panes.some((p) => p.id === paneId)) {
        throw new Error(`Pane not found: ${paneId}`);
      }

      state.tabs = state.tabs.map((t, i) =>
        i === ti ? { ...t, activePaneId: paneId } : t,
      );
      state.activeTabId = tabId;
      return snapshot(state);
    },

    async updatePaneProfile(
      request: UpdatePaneProfileRequest,
    ): Promise<WorkspaceSnapshot> {
      const { tabIndex, paneIndex } = findPane(state, request.paneId);
      const resolved = resolveProfile(request.profileId, request.startupCommand);
      const newSessionId = nextId("session");

      state.tabs = state.tabs.map((tab, ti) =>
        ti === tabIndex
          ? {
              ...tab,
              panes: tab.panes.map((pane, pi) =>
                pi === paneIndex
                  ? {
                      ...pane,
                      sessionId: newSessionId,
                      profileId: resolved.id,
                      profileLabel: resolved.label,
                      startupCommand: resolved.startupCommand,
                      status: "running" as const,
                    }
                  : pane,
              ),
            }
          : tab,
      );

      const pane = state.tabs[tabIndex].panes[paneIndex];
      setTimeout(() => {
        emitMockOutput(
          state,
          pane.id,
          newSessionId,
          `\x1b[36m${resolved.label}\x1b[0m profile applied\r\n\x1b[32m➜\x1b[0m  `,
        );
      }, 50);

      return snapshot(state);
    },

    async updatePaneCwd(
      request: UpdatePaneCwdRequest,
    ): Promise<WorkspaceSnapshot> {
      const { tabIndex, paneIndex } = findPane(state, request.paneId);
      const newSessionId = nextId("session");

      state.tabs = state.tabs.map((tab, ti) =>
        ti === tabIndex
          ? {
              ...tab,
              panes: tab.panes.map((pane, pi) =>
                pi === paneIndex
                  ? {
                      ...pane,
                      sessionId: newSessionId,
                      cwd: request.cwd,
                      status: "running" as const,
                    }
                  : pane,
              ),
            }
          : tab,
      );

      const pane = state.tabs[tabIndex].panes[paneIndex];
      setTimeout(() => {
        emitMockOutput(
          state,
          pane.id,
          newSessionId,
          `\x1b[33mcd ${request.cwd}\x1b[0m\r\n\x1b[32m➜\x1b[0m  `,
        );
      }, 50);

      return snapshot(state);
    },

    async restartPane(paneId: string): Promise<WorkspaceSnapshot> {
      const { tabIndex, paneIndex } = findPane(state, paneId);
      const newSessionId = nextId("session");

      state.tabs = state.tabs.map((tab, ti) =>
        ti === tabIndex
          ? {
              ...tab,
              panes: tab.panes.map((pane, pi) =>
                pi === paneIndex
                  ? { ...pane, sessionId: newSessionId, status: "running" as const }
                  : pane,
              ),
            }
          : tab,
      );

      const pane = state.tabs[tabIndex].panes[paneIndex];
      setTimeout(() => {
        emitMockOutput(
          state,
          pane.id,
          newSessionId,
          `\x1b[90m[mock] Session restarted\x1b[0m\r\n\x1b[32m➜\x1b[0m  `,
        );
      }, 50);

      return snapshot(state);
    },

    async splitPane(request: SplitPaneRequest): Promise<WorkspaceSnapshot> {
      const { tabIndex } = findPane(state, request.paneId);
      const tab = state.tabs[tabIndex];
      const sourcePaneIndex = tab.panes.findIndex((p) => p.id === request.paneId);
      const sourcePane = tab.panes[sourcePaneIndex];

      const profileId = request.profileId ?? sourcePane.profileId;
      const cwd = request.cwd ?? sourcePane.cwd;
      const startupCommand = request.startupCommand ?? sourcePane.startupCommand;
      const newPane = createPane(cwd, profileId, startupCommand, tab.panes.length);

      const newLayout = treeSplitPane(tab.layout, request.paneId, request.direction, newPane.id);
      if (!newLayout) {
        throw new Error(`Cannot split pane: ${request.paneId}`);
      }

      state.tabs = state.tabs.map((t, i) =>
        i === tabIndex
          ? { ...t, layout: newLayout, panes: [...t.panes, newPane] }
          : t,
      );

      setTimeout(() => {
        emitMockOutput(
          state,
          newPane.id,
          newPane.sessionId,
          `\x1b[36m${newPane.profileLabel}\x1b[0m session started in \x1b[33m${newPane.cwd}\x1b[0m\r\n` +
          `\x1b[32m➜\x1b[0m  `,
        );
      }, 80);

      return snapshot(state);
    },

    async closePane(paneId: string): Promise<WorkspaceSnapshot> {
      const { tabIndex } = findPane(state, paneId);
      const tab = state.tabs[tabIndex];

      const result = treeClosePane(tab.layout, paneId);

      if (result === undefined) {
        throw new Error(`Pane not found in layout: ${paneId}`);
      }

      if (result === null) {
        // Last pane — remove tab
        state.tabs = state.tabs.filter((_, i) => i !== tabIndex);

        if (state.tabs.length === 0) {
          return addTab(
            state.settings.defaultLayout,
            state.settings.defaultWorkingDirectory,
            state.settings.defaultProfileId,
            null,
          );
        }

        if (state.activeTabId === tab.id) {
          state.activeTabId = state.tabs[Math.max(0, tabIndex - 1)].id;
        }

        return snapshot(state);
      }

      const newPanes = tab.panes.filter((p) => p.id !== paneId);
      const newActivePaneId = tab.activePaneId === paneId
        ? (newPanes[0]?.id ?? "")
        : tab.activePaneId;

      state.tabs = state.tabs.map((t, i) =>
        i === tabIndex
          ? { ...t, layout: result, panes: newPanes, activePaneId: newActivePaneId }
          : t,
      );

      return snapshot(state);
    },

    async writePty(paneId: string, data: string): Promise<void> {
      const { tabIndex, paneIndex } = findPane(state, paneId);
      const pane = state.tabs[tabIndex].panes[paneIndex];

      if (data === "\r") {
        emitMockOutput(state, paneId, pane.sessionId, "\r\n\x1b[32m➜\x1b[0m  ");
      } else if (data === "\x7f") {
        emitMockOutput(state, paneId, pane.sessionId, "\b \b");
      } else {
        emitMockOutput(state, paneId, pane.sessionId, data);
      }
    },

    async resizePty(_request: PtyResizeRequest): Promise<void> {
      // no-op in mock
    },

    async getAppSettings(): Promise<WorkspaceSettings> {
      return { ...state.settings };
    },

    async updateAppSettings(
      settings: WorkspaceSettings,
    ): Promise<WorkspaceSettings> {
      state.settings = { ...settings };
      return { ...state.settings };
    },

    async resetAppSettings(): Promise<WorkspaceSettings> {
      state.settings = { ...MOCK_DEFAULT_SETTINGS };
      return { ...state.settings };
    },

    async listenToPtyOutput(
      handler: (payload: PtyOutputEvent) => void,
    ): Promise<UnlistenFn> {
      state.outputListeners = [...state.outputListeners, handler];
      return () => {
        state.outputListeners = state.outputListeners.filter((h) => h !== handler);
      };
    },
  };
}
