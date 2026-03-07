import { useEffect, useRef } from "react";
import type { WorkspaceSnapshot } from "@/features/workspace/domain";
import { selectActivePane, selectActiveTab } from "@/features/workspace/selectors";
import {
  findAdjacentPane,
  findNextPane,
  findPreviousPane,
} from "@/features/workspace/splitTree";

interface WorkspaceShortcutsProps {
  workspace: WorkspaceSnapshot | null;
  onCreateTab: () => Promise<void>;
  onCloseTab: (tabId: string) => Promise<void>;
  onClosePane: (paneId: string) => Promise<void>;
  onSelectTab: (tabId: string) => Promise<void>;
  onFocusPane: (tabId: string, paneId: string) => Promise<void>;
  onRestartPane: (paneId: string) => Promise<void>;
  onSplitHorizontal: (paneId: string) => void;
  onSplitVertical: (paneId: string) => void;
  onOpenSettings: () => void;
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
        onSplitHorizontal,
        onSplitVertical,
        onOpenSettings,
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

      // Cmd+D — Split horizontally
      if (!event.shiftKey && event.key.toLowerCase() === "d") {
        event.preventDefault();
        if (activePane) {
          onSplitHorizontal(activePane.id);
        }
        return;
      }

      // Cmd+Shift+D — Split vertically
      if (event.shiftKey && event.key.toLowerCase() === "d") {
        event.preventDefault();
        if (activePane) {
          onSplitVertical(activePane.id);
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

      // Cmd+Shift+] — Next pane (DFS order)
      if (event.shiftKey && event.key === "]") {
        const nextPaneId = findNextPane(activeTab.layout, activePane.id);
        if (nextPaneId) {
          event.preventDefault();
          void onFocusPane(activeTab.id, nextPaneId);
        }
        return;
      }

      // Cmd+Shift+[ — Previous pane (DFS order)
      if (event.shiftKey && event.key === "[") {
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
}
