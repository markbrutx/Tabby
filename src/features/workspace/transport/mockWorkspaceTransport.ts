import {
  type BootstrapSnapshot,
  type NewTabRequest,
  type PaneLifecycleEvent,
  type SplitPaneRequest,
  type UpdatePaneCwdRequest,
  type UpdatePaneProfileRequest,
  type WorkspaceSnapshot,
} from "@/features/workspace/domain";
import {
  splitPane as treeSplitPane,
  closePane as treeClosePane,
  swapPanes as treeSwapPanes,
} from "@/features/workspace/splitTree";
import type { UnlistenFn } from "@/lib/bridge/shared";
import type { WorkspaceTransportInterface } from "./workspaceTransport";
import {
  type MockState,
  BUILT_IN_PROFILES,
  addTab,
  createPane,
  emitMockOutput,
  findPane,
  findTabIndex,
  nextId,
  resolveProfile,
  snapshot,
  uniformSlots,
} from "./mockWorkspaceState";

export { createMockState, MOCK_DEFAULT_SETTINGS } from "./mockWorkspaceState";

export function createMockWorkspaceTransport(
  state: MockState,
): WorkspaceTransportInterface {
  return {
    async bootstrapWorkspace(): Promise<BootstrapSnapshot> {
      if (state.tabs.length === 0 && state.settings.hasCompletedOnboarding) {
        const workspace = addTab(
          state,
          state.settings.defaultLayout,
          uniformSlots(
            state.settings.defaultLayout,
            state.settings.defaultWorkingDirectory || "~",
            state.settings.defaultProfileId || "terminal",
            null,
          ),
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
      const hasPaneConfigs = request.paneConfigs && request.paneConfigs.length > 0;
      const slots = hasPaneConfigs
        ? request.paneConfigs!.map((cfg) => ({
            cwd: cfg.cwd,
            profileId: cfg.profileId,
            startupCommand: cfg.startupCommand,
            url: cfg.url,
          }))
        : uniformSlots(
            request.preset,
            request.cwd ?? state.settings.defaultWorkingDirectory,
            request.profileId ?? state.settings.defaultProfileId,
            request.startupCommand ?? null,
          );
      return addTab(state, request.preset, slots, !!hasPaneConfigs);
    },

    async closeTab(tabId: string): Promise<WorkspaceSnapshot> {
      const index = findTabIndex(state, tabId);
      state.tabs = state.tabs.filter((_, i) => i !== index);

      if (state.tabs.length === 0) {
        return addTab(
          state,
          state.settings.defaultLayout,
          uniformSlots(
            state.settings.defaultLayout,
            state.settings.defaultWorkingDirectory,
            state.settings.defaultProfileId,
            null,
          ),
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
          `\x1b[36m${resolved.label}\x1b[0m profile applied\r\n\x1b[32m\u279c\x1b[0m  `,
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
          `\x1b[33mcd ${request.cwd}\x1b[0m\r\n\x1b[32m\u279c\x1b[0m  `,
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
          `\x1b[90m[mock] Session restarted\x1b[0m\r\n\x1b[32m\u279c\x1b[0m  `,
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
          `\x1b[32m\u279c\x1b[0m  `,
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
        state.tabs = state.tabs.filter((_, i) => i !== tabIndex);

        if (state.tabs.length === 0) {
          return addTab(
            state,
            state.settings.defaultLayout,
            uniformSlots(
              state.settings.defaultLayout,
              state.settings.defaultWorkingDirectory,
              state.settings.defaultProfileId,
              null,
            ),
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

    async swapPanes(paneIdA: string, paneIdB: string): Promise<WorkspaceSnapshot> {
      const tabIndex = state.tabs.findIndex(
        (tab) =>
          tab.panes.some((p) => p.id === paneIdA) &&
          tab.panes.some((p) => p.id === paneIdB),
      );
      if (tabIndex === -1) {
        throw new Error(`Panes ${paneIdA} and ${paneIdB} must be in the same tab`);
      }

      const tab = state.tabs[tabIndex];
      const newLayout = treeSwapPanes(tab.layout, paneIdA, paneIdB);
      if (!newLayout) {
        throw new Error(`Failed to swap panes ${paneIdA} and ${paneIdB}`);
      }

      state.tabs = state.tabs.map((t, i) =>
        i === tabIndex ? { ...t, layout: newLayout } : t,
      );

      return snapshot(state);
    },

    async listenToPaneLifecycle(
      handler: (payload: PaneLifecycleEvent) => void,
    ): Promise<UnlistenFn> {
      state.lifecycleListeners = [...state.lifecycleListeners, handler];
      return () => {
        state.lifecycleListeners = state.lifecycleListeners.filter((h) => h !== handler);
      };
    },
  };
}
