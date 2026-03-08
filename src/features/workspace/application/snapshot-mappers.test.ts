import { describe, expect, it } from "vitest";
import type {
  PaneSpecDto,
  PaneView,
  SplitNodeDto,
  TabView,
  WorkspaceView,
} from "@/contracts/tauri-bindings";
import {
  mapPaneSpecFromDto,
  mapSplitNodeFromDto,
  mapPaneFromDto,
  mapTabFromDto,
  mapWorkspaceFromDto,
} from "./snapshot-mappers";

describe("mapPaneSpecFromDto", () => {
  it("maps a terminal PaneSpecDto to camelCase PaneSpec", () => {
    const dto: PaneSpecDto = {
      kind: "terminal",
      launch_profile_id: "zsh-default",
      working_directory: "/Users/dev/project",
      command_override: "npm start",
    };

    const result = mapPaneSpecFromDto(dto);

    expect(result).toEqual({
      kind: "terminal",
      launchProfileId: "zsh-default",
      workingDirectory: "/Users/dev/project",
      commandOverride: "npm start",
    });
  });

  it("maps a browser PaneSpecDto to camelCase PaneSpec", () => {
    const dto: PaneSpecDto = {
      kind: "browser",
      initial_url: "https://example.com",
    };

    const result = mapPaneSpecFromDto(dto);

    expect(result).toEqual({
      kind: "browser",
      initialUrl: "https://example.com",
    });
  });

  it("preserves null command_override as null commandOverride", () => {
    const dto: PaneSpecDto = {
      kind: "terminal",
      launch_profile_id: "default",
      working_directory: "~",
      command_override: null,
    };

    const result = mapPaneSpecFromDto(dto);

    expect(result.kind).toBe("terminal");
    if (result.kind === "terminal") {
      expect(result.commandOverride).toBeNull();
    }
  });
});

describe("mapSplitNodeFromDto", () => {
  it("maps a single pane node", () => {
    const dto: SplitNodeDto = { type: "pane", paneId: "p1" };

    const result = mapSplitNodeFromDto(dto);

    expect(result).toEqual({ type: "pane", paneId: "p1" });
  });

  it("maps a nested split node recursively", () => {
    const dto: SplitNodeDto = {
      type: "split",
      direction: "horizontal",
      ratio: 0.5,
      first: { type: "pane", paneId: "p1" },
      second: { type: "pane", paneId: "p2" },
    };

    const result = mapSplitNodeFromDto(dto);

    expect(result).toEqual({
      type: "split",
      direction: "horizontal",
      ratio: 0.5,
      first: { type: "pane", paneId: "p1" },
      second: { type: "pane", paneId: "p2" },
    });
  });

  it("maps deeply nested split trees", () => {
    const dto: SplitNodeDto = {
      type: "split",
      direction: "vertical",
      ratio: 0.6,
      first: {
        type: "split",
        direction: "horizontal",
        ratio: 0.5,
        first: { type: "pane", paneId: "p1" },
        second: { type: "pane", paneId: "p2" },
      },
      second: { type: "pane", paneId: "p3" },
    };

    const result = mapSplitNodeFromDto(dto);

    expect(result.type).toBe("split");
    if (result.type === "split") {
      expect(result.direction).toBe("vertical");
      expect(result.first.type).toBe("split");
    }
  });
});

describe("mapPaneFromDto", () => {
  it("maps PaneView to PaneReadModel with converted spec", () => {
    const dto: PaneView = {
      paneId: "p1",
      title: "Terminal",
      spec: {
        kind: "terminal",
        launch_profile_id: "zsh",
        working_directory: "/tmp",
        command_override: null,
      },
    };

    const result = mapPaneFromDto(dto);

    expect(result.paneId).toBe("p1");
    expect(result.title).toBe("Terminal");
    expect(result.spec.kind).toBe("terminal");
    if (result.spec.kind === "terminal") {
      expect(result.spec.launchProfileId).toBe("zsh");
      expect(result.spec.workingDirectory).toBe("/tmp");
    }
  });
});

describe("mapTabFromDto", () => {
  it("maps TabView to TabReadModel with all nested conversions", () => {
    const dto: TabView = {
      tabId: "t1",
      title: "Tab 1",
      layout: { type: "pane", paneId: "p1" },
      panes: [
        {
          paneId: "p1",
          title: "Terminal",
          spec: {
            kind: "terminal",
            launch_profile_id: "zsh",
            working_directory: "~",
            command_override: null,
          },
        },
      ],
      activePaneId: "p1",
    };

    const result = mapTabFromDto(dto);

    expect(result.tabId).toBe("t1");
    expect(result.title).toBe("Tab 1");
    expect(result.activePaneId).toBe("p1");
    expect(result.panes).toHaveLength(1);
    expect(result.panes[0].spec.kind).toBe("terminal");
  });
});

describe("mapWorkspaceFromDto", () => {
  it("maps a full WorkspaceView to WorkspaceReadModel", () => {
    const dto: WorkspaceView = {
      activeTabId: "t1",
      tabs: [
        {
          tabId: "t1",
          title: "Tab 1",
          layout: { type: "pane", paneId: "p1" },
          panes: [
            {
              paneId: "p1",
              title: "Terminal",
              spec: {
                kind: "terminal",
                launch_profile_id: "default",
                working_directory: "/home",
                command_override: null,
              },
            },
          ],
          activePaneId: "p1",
        },
      ],
    };

    const result = mapWorkspaceFromDto(dto);

    expect(result.activeTabId).toBe("t1");
    expect(result.tabs).toHaveLength(1);
    expect(result.tabs[0].panes[0].spec.kind).toBe("terminal");
    if (result.tabs[0].panes[0].spec.kind === "terminal") {
      expect(result.tabs[0].panes[0].spec.launchProfileId).toBe("default");
      expect(result.tabs[0].panes[0].spec.workingDirectory).toBe("/home");
    }
  });

  it("maps an empty workspace with no tabs", () => {
    const dto: WorkspaceView = {
      activeTabId: "",
      tabs: [],
    };

    const result = mapWorkspaceFromDto(dto);

    expect(result.activeTabId).toBe("");
    expect(result.tabs).toHaveLength(0);
  });

  it("result does not contain snake_case field names", () => {
    const dto: WorkspaceView = {
      activeTabId: "t1",
      tabs: [
        {
          tabId: "t1",
          title: "Tab",
          layout: { type: "pane", paneId: "p1" },
          panes: [
            {
              paneId: "p1",
              title: "Browser",
              spec: { kind: "browser", initial_url: "https://google.com" },
            },
          ],
          activePaneId: "p1",
        },
      ],
    };

    const result = mapWorkspaceFromDto(dto);
    const json = JSON.stringify(result);

    expect(json).not.toContain("launch_profile_id");
    expect(json).not.toContain("working_directory");
    expect(json).not.toContain("command_override");
    expect(json).not.toContain("initial_url");
    expect(json).toContain("initialUrl");
  });
});
