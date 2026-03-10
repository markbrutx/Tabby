import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import "@/features/paneRenderers";
import { SplitTreeRenderer } from "./SplitTreeRenderer";
import type { TabSnapshotModel } from "@/features/workspace/model/workspaceSnapshot";
import type { GitClient } from "@/app-shell/clients";
import type { GitResultDto } from "@/contracts/tauri-bindings";
import type { ThemeDefinition } from "@/features/theme/domain/models";

const MOCK_THEME: ThemeDefinition = {
  id: "midnight",
  name: "Midnight",
  kind: "dark",
  builtIn: true,
  colors: {
    bg: "#120b08",
    surface: "#1b120d",
    text: "#f8ece2",
    textSoft: "#dcc7ba",
    textMuted: "#b79684",
    accent: "#f2a084",
    accentStrong: "#e97d61",
    accentSoft: "rgba(233, 125, 97, 0.18)",
    border: "rgba(252, 232, 217, 0.08)",
    borderStrong: "rgba(252, 232, 217, 0.16)",
    danger: "#d56a67",
    dangerStrong: "#bf5656",
    dangerSoft: "rgba(213, 106, 103, 0.14)",
    warning: "#d6a06f",
    surfaceOverlay: "rgba(255, 240, 232, 0.05)",
    surfaceHover: "rgba(255, 240, 232, 0.1)",
    scrollbar: "rgba(255, 240, 232, 0.18)",
    tokenKeyword: "#c792ea",
    tokenString: "#c3e88d",
    tokenComment: "#637777",
    tokenNumber: "#f78c6c",
    tokenType: "#ffcb6b",
    tokenPunctuation: "#89ddff",
  },
};

function makeMockGitClient(): GitClient {
  return {
    dispatch: vi.fn().mockImplementation((cmd: { kind: string }): Promise<GitResultDto> => {
      if (cmd.kind === "status") {
        return Promise.resolve({
          kind: "status" as const,
          files: [],
        });
      }
      if (cmd.kind === "repoState") {
        return Promise.resolve({
          kind: "repoState" as const,
          state: { repoPath: "/my-repo", headBranch: "main", isDetached: false, statusClean: true },
        });
      }
      if (cmd.kind === "diff") {
        return Promise.resolve({ kind: "diff" as const, diffs: [] });
      }
      return Promise.resolve({ kind: cmd.kind } as GitResultDto);
    }),
  };
}

function makeGitTab(overrides: Partial<TabSnapshotModel> = {}): TabSnapshotModel {
  return {
    id: "tab-1",
    title: "Git Tab",
    layout: { type: "pane", paneId: "pane-git-1" },
    panes: [
      {
        id: "pane-git-1",
        title: "Git",
        sessionId: null,
        cwd: "/my-repo",
        profileId: "git",
        profileLabel: "Git",
        startupCommand: null,
        status: null,
        paneKind: "git",
        url: null,
        gitRepoPath: "/my-repo",
        spec: { kind: "git", workingDirectory: "/my-repo" },
        runtime: null,
      },
    ],
    activePaneId: "pane-git-1",
    ...overrides,
  };
}

const defaultHandlers = {
  onFocus: vi.fn().mockResolvedValue(undefined),
  onRestart: vi.fn().mockResolvedValue(undefined),
  onClosePane: vi.fn(),
  onSwapPaneSlots: vi.fn(),
  onOpenGitView: vi.fn(),
  onToggleCollapse: vi.fn(),
  collapsedPaneIds: new Set<string>(),
};

describe("SplitTreeRenderer", () => {
  it("renders GitPane when paneKind is git", async () => {
    const gitClient = makeMockGitClient();
    const tab = makeGitTab();

    render(
      <SplitTreeRenderer
        tab={tab}

        theme={MOCK_THEME}
        visible={true}
        gitClient={gitClient}
        {...defaultHandlers}
      />,
    );

    // GitPaneHeader should render with repo name
    expect(screen.getByTestId("git-pane-header")).toBeInTheDocument();
    expect(screen.getByTestId("git-pane-header-repo")).toHaveTextContent("my-repo");

    // GitPane should appear after loading
    await waitFor(() => {
      expect(screen.getByTestId("git-pane")).toBeInTheDocument();
    });
  });

  it("renders GitPaneHeader with close button when multiple panes exist", async () => {
    const gitClient = makeMockGitClient();
    const tab = makeGitTab({
      layout: {
        type: "split",
        direction: "horizontal",
        ratio: 500,
        first: { type: "pane", paneId: "pane-git-1" },
        second: { type: "pane", paneId: "pane-git-2" },
      },
      panes: [
        {
          id: "pane-git-1",
          title: "Git",
          sessionId: null,
          cwd: "/repo-a",
          profileId: "git",
          profileLabel: "Git",
          startupCommand: null,
          status: null,
          paneKind: "git",
          url: null,
          gitRepoPath: "/repo-a",
          spec: { kind: "git", workingDirectory: "/repo-a" },
          runtime: null,
        },
        {
          id: "pane-git-2",
          title: "Git",
          sessionId: null,
          cwd: "/repo-b",
          profileId: "git",
          profileLabel: "Git",
          startupCommand: null,
          status: null,
          paneKind: "git",
          url: null,
          gitRepoPath: "/repo-b",
          spec: { kind: "git", workingDirectory: "/repo-b" },
          runtime: null,
        },
      ],
    });

    render(
      <SplitTreeRenderer
        tab={tab}

        theme={MOCK_THEME}
        visible={true}
        gitClient={gitClient}
        {...defaultHandlers}
      />,
    );

    const headers = screen.getAllByTestId("git-pane-header");
    expect(headers).toHaveLength(2);

    // Close buttons should be visible when there are multiple panes
    const closeButtons = screen.getAllByTestId("git-pane-header-close");
    expect(closeButtons).toHaveLength(2);
  });
});
