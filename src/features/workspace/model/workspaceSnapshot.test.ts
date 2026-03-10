import { describe, expect, it } from "vitest";
import { buildWorkspaceSnapshotModel } from "@/features/workspace/model/workspaceSnapshot";
import type {
  WorkspaceReadModel,
  PaneReadModel,
  TabReadModel,
  SplitNode,
} from "@/features/workspace/domain/models";
import type { ProfileReadModel } from "@/features/settings/domain/models";
import type { RuntimeReadModel } from "@/features/runtime/domain/models";

function makeSinglePaneLayout(paneId: string): SplitNode {
  return { type: "pane", paneId };
}

function makeTab(overrides: Partial<TabReadModel> & { tabId: string; panes: PaneReadModel[] }): TabReadModel {
  return {
    title: "Tab",
    layout: makeSinglePaneLayout(overrides.panes[0].paneId),
    activePaneId: overrides.panes[0].paneId,
    ...overrides,
  };
}

function makeWorkspace(tabs: TabReadModel[]): WorkspaceReadModel {
  return {
    activeTabId: tabs[0].tabId,
    tabs,
  };
}

function makeRuntime(overrides: Partial<RuntimeReadModel> & { paneId: string }): RuntimeReadModel {
  return {
    runtimeSessionId: null,
    kind: "terminal",
    status: "running",
    lastError: null,
    browserLocation: null,
    terminalCwd: null,
    gitRepoPath: null,
    ...overrides,
  };
}

const defaultProfiles: readonly ProfileReadModel[] = [
  {
    id: "terminal",
    label: "Terminal",
    description: "Default terminal profile",
    startupCommandTemplate: null,
  },
];

describe("buildWorkspaceSnapshotModel", () => {
  it("returns null when workspace is null", () => {
    expect(buildWorkspaceSnapshotModel(null, {}, defaultProfiles)).toBeNull();
  });

  it("builds a terminal pane snapshot", () => {
    const pane: PaneReadModel = {
      paneId: "pane-t1",
      title: "Terminal",
      spec: {
        kind: "terminal",
        launchProfileId: "terminal",
        workingDirectory: "/home/user",
        commandOverride: null,
      },
    };
    const workspace = makeWorkspace([makeTab({ tabId: "tab-1", panes: [pane] })]);
    const runtimes: Record<string, RuntimeReadModel> = {
      "pane-t1": makeRuntime({
        paneId: "pane-t1",
        runtimeSessionId: "sess-1",
        terminalCwd: "/home/user/project",
      }),
    };

    const result = buildWorkspaceSnapshotModel(workspace, runtimes, defaultProfiles);
    const snap = result!.tabs[0].panes[0];

    expect(snap.paneKind).toBe("terminal");
    expect(snap.cwd).toBe("/home/user/project");
    expect(snap.profileId).toBe("terminal");
    expect(snap.sessionId).toBe("sess-1");
    expect(snap.url).toBeNull();
    expect(snap.gitRepoPath).toBeUndefined();
  });

  it("builds a browser pane snapshot", () => {
    const pane: PaneReadModel = {
      paneId: "pane-b1",
      title: "Browser",
      spec: { kind: "browser", initialUrl: "https://example.com" },
    };
    const workspace = makeWorkspace([makeTab({ tabId: "tab-1", panes: [pane] })]);

    const result = buildWorkspaceSnapshotModel(workspace, {}, defaultProfiles);
    const snap = result!.tabs[0].panes[0];

    expect(snap.paneKind).toBe("browser");
    expect(snap.url).toBe("https://example.com");
    expect(snap.profileId).toBe("browser");
    expect(snap.gitRepoPath).toBeUndefined();
  });

  it("builds a git pane snapshot with correct paneKind and gitRepoPath", () => {
    const pane: PaneReadModel = {
      paneId: "pane-g1",
      title: "Git",
      spec: { kind: "git", workingDirectory: "/home/user/repo" },
    };
    const workspace = makeWorkspace([makeTab({ tabId: "tab-1", panes: [pane] })]);
    const runtimes: Record<string, RuntimeReadModel> = {
      "pane-g1": makeRuntime({
        paneId: "pane-g1",
        kind: "git",
        runtimeSessionId: "git-sess-1",
        gitRepoPath: "/home/user/repo",
      }),
    };

    const result = buildWorkspaceSnapshotModel(workspace, runtimes, defaultProfiles);
    const snap = result!.tabs[0].panes[0];

    expect(snap.paneKind).toBe("git");
    expect(snap.gitRepoPath).toBe("/home/user/repo");
    expect(snap.cwd).toBe("/home/user/repo");
    expect(snap.profileId).toBe("git");
    expect(snap.profileLabel).toBe("Git");
    expect(snap.sessionId).toBe("git-sess-1");
    expect(snap.status).toBe("running");
    expect(snap.startupCommand).toBeNull();
    expect(snap.url).toBeNull();
  });

  it("uses runtime gitRepoPath over spec workingDirectory when available", () => {
    const pane: PaneReadModel = {
      paneId: "pane-g1",
      title: "Git",
      spec: { kind: "git", workingDirectory: "/original/path" },
    };
    const workspace = makeWorkspace([makeTab({ tabId: "tab-1", panes: [pane] })]);
    const runtimes: Record<string, RuntimeReadModel> = {
      "pane-g1": makeRuntime({
        paneId: "pane-g1",
        kind: "git",
        gitRepoPath: "/updated/path",
      }),
    };

    const result = buildWorkspaceSnapshotModel(workspace, runtimes, defaultProfiles);
    const snap = result!.tabs[0].panes[0];

    expect(snap.gitRepoPath).toBe("/updated/path");
  });

  it("falls back to spec workingDirectory when runtime has no gitRepoPath", () => {
    const pane: PaneReadModel = {
      paneId: "pane-g2",
      title: "Git View",
      spec: { kind: "git", workingDirectory: "/projects/myapp" },
    };
    const workspace = makeWorkspace([makeTab({ tabId: "tab-1", panes: [pane] })]);

    const result = buildWorkspaceSnapshotModel(workspace, {}, defaultProfiles);
    const snap = result!.tabs[0].panes[0];

    expect(snap.paneKind).toBe("git");
    expect(snap.gitRepoPath).toBe("/projects/myapp");
    expect(snap.sessionId).toBeNull();
    expect(snap.status).toBeNull();
    expect(snap.runtime).toBeNull();
  });

  it("handles mixed pane types in a single tab", () => {
    const terminalPane: PaneReadModel = {
      paneId: "pane-t",
      title: "Terminal",
      spec: {
        kind: "terminal",
        launchProfileId: "terminal",
        workingDirectory: "/tmp",
        commandOverride: null,
      },
    };
    const gitPane: PaneReadModel = {
      paneId: "pane-g",
      title: "Git",
      spec: { kind: "git", workingDirectory: "/repos/tabby" },
    };
    const tab = makeTab({ tabId: "tab-1", panes: [terminalPane, gitPane] });
    const layout: SplitNode = {
      type: "split",
      direction: "horizontal",
      ratio: 500,
      first: { type: "pane", paneId: "pane-t" },
      second: { type: "pane", paneId: "pane-g" },
    };
    const workspace = makeWorkspace([{ ...tab, layout }]);

    const result = buildWorkspaceSnapshotModel(workspace, {}, defaultProfiles);
    const panes = result!.tabs[0].panes;

    expect(panes).toHaveLength(2);
    expect(panes[0].paneKind).toBe("terminal");
    expect(panes[0].gitRepoPath).toBeUndefined();
    expect(panes[1].paneKind).toBe("git");
    expect(panes[1].gitRepoPath).toBe("/repos/tabby");
  });
});
