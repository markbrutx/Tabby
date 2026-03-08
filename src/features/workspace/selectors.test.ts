import { describe, expect, it } from "vitest";
import type { SplitNode, WorkspaceSnapshot } from "@/features/workspace/domain";
import {
  selectActivePane,
  selectActiveTab,
} from "@/features/workspace/selectors";

const layout1x2: SplitNode = {
  type: "split",
  direction: "horizontal",
  ratio: 500,
  first: { type: "pane", paneId: "pane-1" },
  second: { type: "pane", paneId: "pane-2" },
};

const layout2pane: SplitNode = {
  type: "split",
  direction: "horizontal",
  ratio: 500,
  first: { type: "pane", paneId: "pane-3" },
  second: { type: "pane", paneId: "pane-4" },
};

const workspace: WorkspaceSnapshot = {
  activeTabId: "tab-2",
  tabs: [
    {
      id: "tab-1",
      title: "Workspace 1",
      layout: layout1x2,
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
          paneKind: "terminal",
          url: null,
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
          paneKind: "terminal",
          url: null,
        },
      ],
    },
    {
      id: "tab-2",
      title: "Workspace 2",
      layout: layout2pane,
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
          paneKind: "terminal",
          url: null,
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
          paneKind: "terminal",
          url: null,
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

});
