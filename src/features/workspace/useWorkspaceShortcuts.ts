import { useEffect } from "react";
import type { WorkspaceSnapshot } from "@/features/workspace/domain";

interface WorkspaceShortcutsProps {
  workspace: WorkspaceSnapshot | null;
  defaultLayout: string;
  onCreateTab: () => Promise<void>;
  onCloseTab: (tabId: string) => Promise<void>;
  onSelectTab: (tabId: string) => Promise<void>;
}

export function useWorkspaceShortcuts({
  workspace,
  onCreateTab,
  onCloseTab,
  onSelectTab,
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
      if (Number.isNaN(index) || index < 0 || index > 8) {
        return;
      }

      const tab = workspace.tabs[index];
      if (!tab) {
        return;
      }

      event.preventDefault();
      void onSelectTab(tab.id);
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [onCloseTab, onCreateTab, onSelectTab, workspace]);
}
