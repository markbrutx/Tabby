import type {
  PaneSnapshot,
  TabSnapshot,
  WorkspaceSnapshot,
} from "@/features/workspace/domain";
import type { WorkspaceSummary } from "@/features/workspace/types";

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

export function selectVisiblePanes(
  workspace: WorkspaceSnapshot | null | undefined,
): PaneSnapshot[] {
  return selectActiveTab(workspace)?.panes ?? [];
}

export function selectWorkspaceSummary(
  workspace: WorkspaceSnapshot | null | undefined,
): WorkspaceSummary {
  const activeTab = selectActiveTab(workspace);
  const activePane = selectActivePane(workspace);

  return {
    activeTabId: activeTab?.id ?? null,
    activeTabTitle: activeTab?.title ?? null,
    activePaneId: activePane?.id ?? null,
    activePaneTitle: activePane?.title ?? null,
    activePaneStatus: activePane?.status ?? null,
    paneCount: workspace?.tabs.reduce((count, tab) => count + tab.panes.length, 0) ?? 0,
    tabCount: workspace?.tabs.length ?? 0,
  };
}
