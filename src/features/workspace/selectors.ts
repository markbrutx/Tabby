import type {
  PaneSnapshotModel,
  TabSnapshotModel,
  WorkspaceSnapshotModel,
} from "@/features/workspace/model/workspaceSnapshot";

export function selectActiveTab(
  workspace: WorkspaceSnapshotModel | null | undefined,
): TabSnapshotModel | null {
  if (!workspace || workspace.tabs.length === 0) {
    return null;
  }

  return (
    workspace.tabs.find((tab) => tab.id === workspace.activeTabId) ??
    workspace.tabs[0] ??
    null
  );
}

export function selectActivePane(
  workspace: WorkspaceSnapshotModel | null | undefined,
): PaneSnapshotModel | null {
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
