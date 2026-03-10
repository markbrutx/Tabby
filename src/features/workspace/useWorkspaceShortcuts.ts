import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import type { WorkspaceSnapshotModel } from "@/features/workspace/model/workspaceSnapshot";
import { selectActivePane, selectActiveTab } from "@/features/workspace/selectors";
import {
  findAdjacentPane,
  findNextPane,
  findPreviousPane,
} from "@/features/workspace/layoutReadModel";
import { isTauriRuntime } from "@/lib/runtime";

interface WorkspaceShortcutsProps {
  workspace: WorkspaceSnapshotModel | null;
  onCreateTab: () => void | Promise<void>;
  onCloseTab: (tabId: string) => void | Promise<void>;
  onClosePane: (paneId: string) => void | Promise<void>;
  onSelectTab: (tabId: string) => Promise<void>;
  onFocusPane: (tabId: string, paneId: string) => Promise<void>;
  onRestartPane: (paneId: string) => Promise<void>;
  onSplitRight: (paneId: string) => void;
  onSplitDown: (paneId: string) => void;
  onOpenSettings: () => void;
  onOpenShortcuts: () => void;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onZoomReset: () => void;
}

export function useWorkspaceShortcuts(props: WorkspaceShortcutsProps) {
  const propsRef = useRef(props);
  useEffect(() => {
    propsRef.current = props;
  });

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      const {
        workspace,
        onCreateTab,
        onCloseTab,
        onClosePane,
        onSelectTab,
        onFocusPane,
        onRestartPane,
        onSplitRight,
        onSplitDown,
        onOpenSettings,
        onOpenShortcuts,
        onZoomIn,
        onZoomOut,
        onZoomReset,
      } = propsRef.current;

      if (!workspace || !event.metaKey) {
        return;
      }

      // Cmd+T — New tab
      if (event.key.toLowerCase() === "t") {
        event.preventDefault();
        void onCreateTab();
        return;
      }

      // Cmd+, — Settings
      if (event.key === ",") {
        event.preventDefault();
        onOpenSettings();
        return;
      }

      // Cmd+/ — Shortcuts help
      if (event.key === "/") {
        event.preventDefault();
        onOpenShortcuts();
        return;
      }

      // Cmd+= or Cmd++ — Zoom in
      if (event.key === "=" || event.key === "+") {
        event.preventDefault();
        onZoomIn();
        return;
      }

      // Cmd+- — Zoom out
      if (event.key === "-") {
        event.preventDefault();
        onZoomOut();
        return;
      }

      // Cmd+0 — Reset zoom
      if (event.key === "0") {
        event.preventDefault();
        onZoomReset();
        return;
      }

      const activeTab = selectActiveTab(workspace);
      const activePane = selectActivePane(workspace);

      // Cmd+Shift+W — Close entire tab
      if (event.shiftKey && event.key.toLowerCase() === "w") {
        event.preventDefault();
        if (activeTab) {
          void onCloseTab(activeTab.id);
        }
        return;
      }

      // Cmd+W — Close active pane
      if (event.key.toLowerCase() === "w") {
        event.preventDefault();
        if (activePane) {
          void onClosePane(activePane.id);
        }
        return;
      }

      // Cmd+D — Split right
      if (event.key.toLowerCase() === "d") {
        event.preventDefault();
        if (activePane) {
          onSplitRight(activePane.id);
        }
        return;
      }

      // Cmd+E — Split down
      if (event.key.toLowerCase() === "e") {
        event.preventDefault();
        if (activePane) {
          onSplitDown(activePane.id);
        }
        return;
      }

      // Cmd+1..9 — Switch tabs
      const index = Number(event.key) - 1;
      if (!Number.isNaN(index) && index >= 0 && index <= 8) {
        const tab = workspace.tabs[index];
        if (tab) {
          event.preventDefault();
          void onSelectTab(tab.id);
        }
        return;
      }

      if (!activeTab || !activePane) {
        return;
      }

      // Alt+Arrow — Navigate adjacent pane (tree-based)
      if (event.altKey) {
        const directionMap: Record<string, "up" | "down" | "left" | "right"> = {
          ArrowUp: "up",
          ArrowDown: "down",
          ArrowLeft: "left",
          ArrowRight: "right",
        };
        const navDirection = directionMap[event.key];
        if (navDirection) {
          const nextPaneId = findAdjacentPane(
            activeTab.layout,
            activePane.id,
            navDirection,
          );
          if (nextPaneId) {
            event.preventDefault();
            void onFocusPane(activeTab.id, nextPaneId);
          }
          return;
        }
      }

      // Cmd+] — Next pane (DFS order)
      if (event.key === "]") {
        const nextPaneId = findNextPane(activeTab.layout, activePane.id);
        if (nextPaneId) {
          event.preventDefault();
          void onFocusPane(activeTab.id, nextPaneId);
        }
        return;
      }

      // Cmd+[ — Previous pane (DFS order)
      if (event.key === "[") {
        const nextPaneId = findPreviousPane(activeTab.layout, activePane.id);
        if (nextPaneId) {
          event.preventDefault();
          void onFocusPane(activeTab.id, nextPaneId);
        }
        return;
      }

      // Cmd+Shift+R — Restart active pane
      if (event.shiftKey && event.key.toLowerCase() === "r") {
        event.preventDefault();
        void onRestartPane(activePane.id);
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, []);

  useEffect(() => {
    if (!isTauriRuntime()) return;

    let cancelled = false;
    const unlisteners: Array<() => void> = [];

    function reg(event: string, handler: () => void) {
      void listen(event, handler).then((unlisten) => {
        if (cancelled) {
          unlisten();
        } else {
          unlisteners.push(unlisten);
        }
      });
    }

    reg("shortcut-new-tab", () => {
      void propsRef.current.onCreateTab();
    });

    reg("shortcut-close-pane", () => {
      const { workspace, onClosePane } = propsRef.current;
      if (!workspace) return;
      const pane = selectActivePane(workspace);
      if (pane) void onClosePane(pane.id);
    });

    reg("shortcut-close-tab", () => {
      const { workspace, onCloseTab } = propsRef.current;
      if (!workspace) return;
      const tab = selectActiveTab(workspace);
      if (tab) void onCloseTab(tab.id);
    });

    reg("shortcut-split-right", () => {
      const { workspace, onSplitRight } = propsRef.current;
      if (!workspace) return;
      const pane = selectActivePane(workspace);
      if (pane) onSplitRight(pane.id);
    });

    reg("shortcut-split-down", () => {
      const { workspace, onSplitDown } = propsRef.current;
      if (!workspace) return;
      const pane = selectActivePane(workspace);
      if (pane) onSplitDown(pane.id);
    });

    reg("shortcut-restart-pane", () => {
      const { workspace, onRestartPane } = propsRef.current;
      if (!workspace) return;
      const pane = selectActivePane(workspace);
      if (pane) void onRestartPane(pane.id);
    });

    reg("shortcut-next-pane", () => {
      const { workspace, onFocusPane } = propsRef.current;
      if (!workspace) return;
      const tab = selectActiveTab(workspace);
      const pane = selectActivePane(workspace);
      if (!tab || !pane) return;
      const next = findNextPane(tab.layout, pane.id);
      if (next) void onFocusPane(tab.id, next);
    });

    reg("shortcut-prev-pane", () => {
      const { workspace, onFocusPane } = propsRef.current;
      if (!workspace) return;
      const tab = selectActiveTab(workspace);
      const pane = selectActivePane(workspace);
      if (!tab || !pane) return;
      const prev = findPreviousPane(tab.layout, pane.id);
      if (prev) void onFocusPane(tab.id, prev);
    });

    reg("shortcut-shortcuts-help", () => {
      propsRef.current.onOpenShortcuts();
    });

    reg("menu-zoom-in", () => {
      propsRef.current.onZoomIn();
    });

    reg("menu-zoom-out", () => {
      propsRef.current.onZoomOut();
    });

    reg("menu-zoom-reset", () => {
      propsRef.current.onZoomReset();
    });

    for (let i = 1; i <= 9; i++) {
      const index = i - 1;
      reg(`shortcut-tab-${i}`, () => {
        const { workspace, onSelectTab } = propsRef.current;
        if (!workspace) return;
        const tab = workspace.tabs[index];
        if (tab) void onSelectTab(tab.id);
      });
    }

    return () => {
      cancelled = true;
      for (const unlisten of unlisteners) {
        unlisten();
      }
    };
  }, []);
}
