import {
  BROWSER_PROFILE_ID,
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

function resolveEffectiveCwd(state: MockState, cwd?: string | null): string {
  const explicit = cwd?.trim();
  if (explicit) {
    return explicit;
  }

  const configured = state.settings.defaultWorkingDirectory.trim();
  if (configured) {
    return configured;
  }

  const last = state.settings.lastWorkingDirectory?.trim();
  if (last) {
    return last;
  }

  return "~";
}

function resolveEffectiveProfileId(state: MockState, profileId?: string | null): string {
  const explicit = profileId?.trim();
  if (explicit) {
    return explicit;
  }

  return state.settings.defaultProfileId || "terminal";
}

function publishWorkspace(state: MockState, workspace: WorkspaceSnapshot): WorkspaceSnapshot {
  for (const listener of state.workspaceListeners) {
    listener(workspace);
  }

  return workspace;
}

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
            resolveEffectiveCwd(state),
            resolveEffectiveProfileId(state),
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
            cwd: resolveEffectiveCwd(state, cfg.cwd),
            profileId: resolveEffectiveProfileId(state, cfg.profileId),
            startupCommand: cfg.startupCommand,
            url: cfg.url,
          }))
        : uniformSlots(
            request.preset,
            resolveEffectiveCwd(state, request.cwd),
            resolveEffectiveProfileId(state, request.profileId),
            request.startupCommand ?? null,
          );
      return publishWorkspace(state, addTab(state, request.preset, slots, !!hasPaneConfigs));
    },

    async closeTab(tabId: string): Promise<WorkspaceSnapshot> {
      const index = findTabIndex(state, tabId);
      state.tabs = state.tabs.filter((_, i) => i !== index);

      if (state.tabs.length === 0) {
        return publishWorkspace(state, addTab(
          state,
          state.settings.defaultLayout,
          uniformSlots(
            state.settings.defaultLayout,
            resolveEffectiveCwd(state),
            resolveEffectiveProfileId(state),
            null,
          ),
        ));
      }

      if (state.activeTabId === tabId) {
        state.activeTabId = state.tabs[Math.max(0, index - 1)].id;
      }

      return publishWorkspace(state, snapshot(state));
    },

    async setActiveTab(tabId: string): Promise<WorkspaceSnapshot> {
      findTabIndex(state, tabId);
      state.activeTabId = tabId;
      return publishWorkspace(state, snapshot(state));
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
      return publishWorkspace(state, snapshot(state));
    },

    async updatePaneProfile(
      request: UpdatePaneProfileRequest,
    ): Promise<WorkspaceSnapshot> {
      const { tabIndex, paneIndex } = findPane(state, request.paneId);
      const resolved = resolveProfile(request.profileId, request.startupCommand);
      const previousPane = state.tabs[tabIndex].panes[paneIndex];
      const switchingToBrowser = resolved.id === BROWSER_PROFILE_ID;
      const newSessionId = switchingToBrowser
        ? previousPane.paneKind === "browser"
          ? previousPane.sessionId
          : nextId("browser")
        : nextId("session");

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
                      paneKind: switchingToBrowser ? "browser" as const : "terminal" as const,
                      url: switchingToBrowser ? (pane.url ?? "https://google.com") : null,
                      status: "running" as const,
                    }
                  : pane,
              ),
            }
          : tab,
      );

      if (!switchingToBrowser) {
        const pane = state.tabs[tabIndex].panes[paneIndex];
        setTimeout(() => {
          emitMockOutput(
            state,
            pane.id,
            newSessionId,
            `\x1b[36m${resolved.label}\x1b[0m profile applied\r\n\x1b[32m\u279c\x1b[0m  `,
          );
        }, 50);
      }

      return publishWorkspace(state, snapshot(state));
    },

    async updatePaneCwd(
      request: UpdatePaneCwdRequest,
    ): Promise<WorkspaceSnapshot> {
      const { tabIndex, paneIndex } = findPane(state, request.paneId);
      const nextCwd = resolveEffectiveCwd(state, request.cwd);
      const pane = state.tabs[tabIndex].panes[paneIndex];
      if (pane.paneKind === "browser") {
        state.tabs = state.tabs.map((tab, ti) =>
          ti === tabIndex
            ? {
                ...tab,
                panes: tab.panes.map((candidate, pi) =>
                  pi === paneIndex ? { ...candidate, cwd: nextCwd } : candidate,
                ),
              }
            : tab,
        );

        return publishWorkspace(state, snapshot(state));
      }

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
                      cwd: nextCwd,
                      status: "running" as const,
                    }
                  : pane,
              ),
            }
          : tab,
      );

      const updatedPane = state.tabs[tabIndex].panes[paneIndex];
      setTimeout(() => {
        emitMockOutput(
          state,
          updatedPane.id,
          newSessionId,
          `\x1b[33mcd ${nextCwd}\x1b[0m\r\n\x1b[32m\u279c\x1b[0m  `,
        );
      }, 50);

      return publishWorkspace(state, snapshot(state));
    },

    async restartPane(paneId: string): Promise<WorkspaceSnapshot> {
      const { tabIndex, paneIndex } = findPane(state, paneId);
      const restartingPane = state.tabs[tabIndex].panes[paneIndex];
      if (restartingPane.paneKind === "browser") {
        return snapshot(state);
      }

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

      const restartedPane = state.tabs[tabIndex].panes[paneIndex];
      setTimeout(() => {
        emitMockOutput(
          state,
          restartedPane.id,
          newSessionId,
          `\x1b[90m[mock] Session restarted\x1b[0m\r\n\x1b[32m\u279c\x1b[0m  `,
        );
      }, 50);

      return publishWorkspace(state, snapshot(state));
    },

    async splitPane(request: SplitPaneRequest): Promise<WorkspaceSnapshot> {
      const { tabIndex } = findPane(state, request.paneId);
      const tab = state.tabs[tabIndex];
      const sourcePaneIndex = tab.panes.findIndex((p) => p.id === request.paneId);
      const sourcePane = tab.panes[sourcePaneIndex];

      const profileId = resolveEffectiveProfileId(state, request.profileId ?? sourcePane.profileId);
      const cwd = request.cwd?.trim() ? request.cwd.trim() : sourcePane.cwd;
      const startupCommand = request.startupCommand ?? sourcePane.startupCommand;
      const newPane = createPane(
        cwd,
        profileId,
        startupCommand,
        tab.panes.length,
        profileId === BROWSER_PROFILE_ID ? sourcePane.url : null,
      );

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

      return publishWorkspace(state, snapshot(state));
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
          return publishWorkspace(state, addTab(
            state,
            state.settings.defaultLayout,
            uniformSlots(
              state.settings.defaultLayout,
              resolveEffectiveCwd(state),
              resolveEffectiveProfileId(state),
              null,
            ),
          ));
        }

        if (state.activeTabId === tab.id) {
          state.activeTabId = state.tabs[Math.max(0, tabIndex - 1)].id;
        }

        return publishWorkspace(state, snapshot(state));
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

      return publishWorkspace(state, snapshot(state));
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

      return publishWorkspace(state, snapshot(state));
    },

    async listenToPaneLifecycle(
      handler: (payload: PaneLifecycleEvent) => void,
    ): Promise<UnlistenFn> {
      state.lifecycleListeners = [...state.lifecycleListeners, handler];
      return () => {
        state.lifecycleListeners = state.lifecycleListeners.filter((h) => h !== handler);
      };
    },

    async listenToWorkspaceChanged(
      handler: (workspace: WorkspaceSnapshot) => void,
    ): Promise<UnlistenFn> {
      state.workspaceListeners = [...state.workspaceListeners, handler];
      return () => {
        state.workspaceListeners = state.workspaceListeners.filter((h) => h !== handler);
      };
    },
  };
}
