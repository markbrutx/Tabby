import { describe, expect, it } from "vitest";
import type { TabSnapshot } from "@/features/workspace/domain";
import {
  selectAdjacentPaneId,
  selectNextPaneId,
  selectPreviousPaneId,
} from "@/features/workspace/navigation";

const gridTab: TabSnapshot = {
  id: "tab-1",
  title: "Workspace 1",
  preset: "2x2",
  activePaneId: "pane-1",
  panes: [
    {
      id: "pane-1",
      sessionId: "session-1",
      title: "Pane 1",
      cwd: "/tmp/1",
      profileId: "terminal",
      profileLabel: "Terminal",
      startupCommand: null,
      status: "running",
    },
    {
      id: "pane-2",
      sessionId: "session-2",
      title: "Pane 2",
      cwd: "/tmp/2",
      profileId: "terminal",
      profileLabel: "Terminal",
      startupCommand: null,
      status: "running",
    },
    {
      id: "pane-3",
      sessionId: "session-3",
      title: "Pane 3",
      cwd: "/tmp/3",
      profileId: "terminal",
      profileLabel: "Terminal",
      startupCommand: null,
      status: "running",
    },
    {
      id: "pane-4",
      sessionId: "session-4",
      title: "Pane 4",
      cwd: "/tmp/4",
      profileId: "terminal",
      profileLabel: "Terminal",
      startupCommand: null,
      status: "running",
    },
  ],
};

describe("workspace navigation", () => {
  it("cycles to the next and previous pane in row-major order", () => {
    expect(selectNextPaneId(gridTab, "pane-2")).toBe("pane-3");
    expect(selectPreviousPaneId(gridTab, "pane-3")).toBe("pane-2");
  });

  it("moves focus directionally inside the current layout grid", () => {
    expect(selectAdjacentPaneId(gridTab, "pane-1", "right")).toBe("pane-2");
    expect(selectAdjacentPaneId(gridTab, "pane-1", "down")).toBe("pane-3");
    expect(selectAdjacentPaneId(gridTab, "pane-4", "left")).toBe("pane-3");
    expect(selectAdjacentPaneId(gridTab, "pane-4", "up")).toBe("pane-2");
  });

  it("returns null when moving outside the available pane bounds", () => {
    expect(selectAdjacentPaneId(gridTab, "pane-1", "left")).toBeNull();
    expect(selectAdjacentPaneId(gridTab, "pane-2", "up")).toBeNull();
  });
});
