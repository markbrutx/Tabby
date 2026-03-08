import {
  BROWSER_PROFILE_ID,
  DEFAULT_BROWSER_URL,
  type PaneProfile,
  type PaneRuntimeStatus,
  type PaneRuntimeView,
  type PaneSpecDto,
  type SplitNode,
  type WorkspaceView,
} from "@/features/workspace/domain";

export interface PaneSnapshotModel {
  id: string;
  title: string;
  sessionId: string | null;
  cwd: string;
  profileId: string;
  profileLabel: string;
  startupCommand: string | null;
  status: PaneRuntimeStatus | null;
  paneKind: "terminal" | "browser";
  url: string | null;
  spec: PaneSpecDto;
  runtime: PaneRuntimeView | null;
}

export interface TabSnapshotModel {
  id: string;
  title: string;
  layout: SplitNode;
  panes: PaneSnapshotModel[];
  activePaneId: string;
}

export interface WorkspaceSnapshotModel {
  activeTabId: string;
  tabs: TabSnapshotModel[];
}

function findProfileLabel(profileId: string, profiles: PaneProfile[]): string {
  return profiles.find((profile) => profile.id === profileId)?.label ?? profileId;
}

function defaultStartupCommand(profileId: string, profiles: PaneProfile[]): string | null {
  return profiles.find((profile) => profile.id === profileId)?.startupCommandTemplate ?? null;
}

export function buildWorkspaceSnapshotModel(
  workspace: WorkspaceView | null,
  runtimes: Record<string, PaneRuntimeView>,
  profiles: PaneProfile[],
): WorkspaceSnapshotModel | null {
  if (!workspace) {
    return null;
  }

  return {
    activeTabId: workspace.activeTabId,
    tabs: workspace.tabs.map((tab) => ({
      id: tab.tabId,
      title: tab.title,
      layout: tab.layout,
      activePaneId: tab.activePaneId,
      panes: tab.panes.map((pane) => {
        const runtime = runtimes[pane.paneId] ?? null;
        if (pane.spec.kind === "browser") {
          return {
            id: pane.paneId,
            title: pane.title,
            sessionId: runtime?.runtimeSessionId ?? null,
            cwd: "~",
            profileId: BROWSER_PROFILE_ID,
            profileLabel: "Browser",
            startupCommand: null,
            status: runtime?.status ?? null,
            paneKind: "browser" as const,
            url: runtime?.browserLocation ?? pane.spec.initial_url ?? DEFAULT_BROWSER_URL,
            spec: pane.spec,
            runtime,
          };
        }

        return {
          id: pane.paneId,
          title: pane.title,
          sessionId: runtime?.runtimeSessionId ?? null,
          cwd: pane.spec.working_directory,
          profileId: pane.spec.launch_profile_id,
          profileLabel: findProfileLabel(pane.spec.launch_profile_id, profiles),
          startupCommand:
            pane.spec.command_override ??
            defaultStartupCommand(pane.spec.launch_profile_id, profiles),
          status: runtime?.status ?? null,
          paneKind: "terminal" as const,
          url: null,
          spec: pane.spec,
          runtime,
        };
      }),
    })),
  };
}
