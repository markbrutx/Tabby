import type { PaneRuntimeStatus } from "@/features/workspace/domain";

export interface WorkspaceSummary {
  activeTabId: string | null;
  activeTabTitle: string | null;
  activePaneId: string | null;
  activePaneTitle: string | null;
  activePaneStatus: PaneRuntimeStatus | null;
  paneCount: number;
  tabCount: number;
}
