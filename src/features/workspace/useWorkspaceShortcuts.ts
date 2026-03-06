import { useEffect } from "react";
import type { WorkspaceSnapshot } from "@/features/workspace/domain";
import {
  selectAdjacentPaneId,
  selectNextPaneId,
  selectPreviousPaneId,
} from "@/features/workspace/navigation";
import { selectActivePane, selectActiveTab } from "@/features/workspace/selectors";

interface WorkspaceShortcutsProps {
  workspace: WorkspaceSnapshot | null;
  defaultLayout: string;
  onCreateTab: () => Promise<void>;
  onCloseTab: (tabId: string) => Promise<void>;
  onSelectTab: (tabId: string) => Promise<void>;
  onFocusPane: (tabId: string, paneId: string) => Promise<void>;
  onRestartPane: (paneId: string) => Promise<void>;
}

export function useWorkspaceShortcuts({
  workspace,
  onCreateTab,
  onCloseTab,
  onSelectTab,
  onFocusPane,
  onRestartPane,
}: WorkspaceShortcutsProps) {
  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (!workspace || !event.metaKey) {
        return;
      }

      if (event.key.toLowerCase() === "t") {
        event.preventDefault();
        void onCreateTab();
        return;
      }

      if (event.key.toLowerCase() === "w") {
        event.preventDefault();
        if (workspace.activeTabId) {
          void onCloseTab(workspace.activeTabId);
        }
        return;
      }

      const index = Number(event.key) - 1;
      if (!Number.isNaN(index) && index >= 0 && index <= 8) {
        const tab = workspace.tabs[index];
        if (!tab) {
          return;
        }

        event.preventDefault();
        void onSelectTab(tab.id);
        return;
      }

      const activeTab = selectActiveTab(workspace);
      const activePane = selectActivePane(workspace);
      if (!activeTab || !activePane) {
        return;
      }

      if (event.altKey && event.key === "ArrowLeft") {
        const nextPaneId = selectAdjacentPaneId(activeTab, activePane.id, "left");
        if (nextPaneId) {
          event.preventDefault();
          void onFocusPane(activeTab.id, nextPaneId);
        }
        return;
      }

      if (event.altKey && event.key === "ArrowRight") {
        const nextPaneId = selectAdjacentPaneId(activeTab, activePane.id, "right");
        if (nextPaneId) {
          event.preventDefault();
          void onFocusPane(activeTab.id, nextPaneId);
        }
        return;
      }

      if (event.altKey && event.key === "ArrowUp") {
        const nextPaneId = selectAdjacentPaneId(activeTab, activePane.id, "up");
        if (nextPaneId) {
          event.preventDefault();
          void onFocusPane(activeTab.id, nextPaneId);
        }
        return;
      }

      if (event.altKey && event.key === "ArrowDown") {
        const nextPaneId = selectAdjacentPaneId(activeTab, activePane.id, "down");
        if (nextPaneId) {
          event.preventDefault();
          void onFocusPane(activeTab.id, nextPaneId);
        }
        return;
      }

      if (event.shiftKey && event.key === "]") {
        const nextPaneId = selectNextPaneId(activeTab, activePane.id);
        if (nextPaneId) {
          event.preventDefault();
          void onFocusPane(activeTab.id, nextPaneId);
        }
        return;
      }

      if (event.shiftKey && event.key === "[") {
        const nextPaneId = selectPreviousPaneId(activeTab, activePane.id);
        if (nextPaneId) {
          event.preventDefault();
          void onFocusPane(activeTab.id, nextPaneId);
        }
        return;
      }

      if (event.shiftKey && event.key.toLowerCase() === "r") {
        event.preventDefault();
        void onRestartPane(activePane.id);
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [onCloseTab, onCreateTab, onFocusPane, onRestartPane, onSelectTab, workspace]);
}
