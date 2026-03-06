import { describe, expect, it } from "vitest";
import type { WorkspaceSnapshot } from "@/features/workspace/domain";
import {
  selectActivePane,
  selectActiveTab,
  selectVisiblePanes,
  selectWorkspaceSummary,
} from "@/features/workspace/selectors";

const workspace: WorkspaceSnapshot = {
  activeTabId: "tab-2",
  tabs: [
    {
      id: "tab-1",
      title: "Workspace 1",
      preset: "1x2",
      activePaneId: "pane-2",
      panes: [
        {
          id: "pane-1",
          sessionId: "session-1",
          title: "Pane 1",
          cwd: "/tmp/alpha",
          profileId: "terminal",
          profileLabel: "Terminal",
          startupCommand: null,
          status: "running",
        },
        {
          id: "pane-2",
          sessionId: "session-2",
          title: "Pane 2",
          cwd: "/tmp/bravo",
          profileId: "claude",
          profileLabel: "Claude Code",
          startupCommand: "claude",
          status: "running",
        },
      ],
    },
    {
      id: "tab-2",
      title: "Workspace 2",
      preset: "2x2",
      activePaneId: "pane-4",
      panes: [
        {
          id: "pane-3",
          sessionId: "session-3",
          title: "Pane 1",
          cwd: "/tmp/charlie",
          profileId: "terminal",
          profileLabel: "Terminal",
          startupCommand: null,
          status: "starting",
        },
        {
          id: "pane-4",
          sessionId: "session-4",
          title: "Pane 2",
          cwd: "/tmp/delta",
          profileId: "codex",
          profileLabel: "Codex",
          startupCommand: "codex",
          status: "running",
        },
      ],
    },
  ],
};

describe("workspace selectors", () => {
  it("derives the active tab and pane from the workspace snapshot", () => {
    expect(selectActiveTab(workspace)?.id).toBe("tab-2");
    expect(selectActivePane(workspace)?.id).toBe("pane-4");
  });

  it("returns only the panes visible in the active tab", () => {
    expect(selectVisiblePanes(workspace).map((pane) => pane.id)).toEqual([
      "pane-3",
      "pane-4",
    ]);
  });

  it("builds a compact summary for the workspace header", () => {
    expect(selectWorkspaceSummary(workspace)).toEqual({
      activeTabId: "tab-2",
      activeTabTitle: "Workspace 2",
      activePaneId: "pane-4",
      activePaneTitle: "Pane 2",
      activePaneStatus: "running",
      paneCount: 4,
      tabCount: 2,
    });
  });
});
