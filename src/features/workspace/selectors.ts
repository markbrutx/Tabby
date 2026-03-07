import type {
  PaneSnapshot,
  TabSnapshot,
  WorkspaceSnapshot,
} from "@/features/workspace/domain";

export function selectActiveTab(
  workspace: WorkspaceSnapshot | null | undefined,
): TabSnapshot | null {
  if (!workspace || workspace.tabs.length === 0) {
    return null;
  }

  return (
    workspace.tabs.find((tab) => tab.id === workspace.activeTabId) ?? workspace.tabs[0] ?? null
  );
}

export function selectActivePane(
  workspace: WorkspaceSnapshot | null | undefined,
): PaneSnapshot | null {
  const activeTab = selectActiveTab(workspace);
  if (!activeTab) {
    return null;
  }

  return (
    activeTab.panes.find((pane) => pane.id === activeTab.activePaneId) ??
    activeTab.panes[0] ??
    null
  );
}