import type { PaneRuntimeView } from "@/contracts/tauri-bindings";
import {
  BROWSER_PROFILE_ID,
  DEFAULT_BROWSER_URL,
  type PaneSpec,
  type SplitNode,
  type WorkspaceReadModel,
} from "@/features/workspace/domain/models";
import type { ProfileReadModel } from "@/features/settings/domain/models";
import type { RuntimeStatus } from "@/features/runtime/domain/models";

export interface PaneSnapshotModel {
  id: string;
  title: string;
  sessionId: string | null;
  cwd: string;
  profileId: string;
  profileLabel: string;
  startupCommand: string | null;
  status: RuntimeStatus | null;
  paneKind: "terminal" | "browser";
  url: string | null;
  spec: PaneSpec;
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

function findProfileLabel(profileId: string, profiles: readonly ProfileReadModel[]): string {
  return profiles.find((profile) => profile.id === profileId)?.label ?? profileId;
}

function defaultStartupCommand(profileId: string, profiles: readonly ProfileReadModel[]): string | null {
  return profiles.find((profile) => profile.id === profileId)?.startupCommandTemplate ?? null;
}

export function buildWorkspaceSnapshotModel(
  workspace: WorkspaceReadModel | null,
  runtimes: Record<string, PaneRuntimeView>,
  profiles: readonly ProfileReadModel[],
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
            url: runtime?.browserLocation ?? pane.spec.initialUrl ?? DEFAULT_BROWSER_URL,
            spec: pane.spec,
            runtime,
          };
        }

        return {
          id: pane.paneId,
          title: pane.title,
          sessionId: runtime?.runtimeSessionId ?? null,
          cwd: pane.spec.workingDirectory,
          profileId: pane.spec.launchProfileId,
          profileLabel: findProfileLabel(pane.spec.launchProfileId, profiles),
          startupCommand:
            pane.spec.commandOverride ??
            defaultStartupCommand(pane.spec.launchProfileId, profiles),
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
