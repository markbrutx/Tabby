import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { GitPane } from "./GitPane";
import type { PaneSnapshotModel } from "@/features/workspace/model/workspaceSnapshot";
import type { GitClient } from "@/app-shell/clients";
import type { GitResultDto } from "@/contracts/tauri-bindings";

function makePaneSnapshot(overrides: Partial<PaneSnapshotModel> = {}): PaneSnapshotModel {
  return {
    id: "pane-1",
    title: "Git",
    sessionId: null,
    cwd: "/repo",
    profileId: "git",
    profileLabel: "Git",
    startupCommand: null,
    status: null,
    paneKind: "git",
    url: null,
    gitRepoPath: "/repo",
    spec: { kind: "git", workingDirectory: "/repo" },
    runtime: null,
    ...overrides,
  };
}

function makeMockGitClient(
  dispatchFn?: (command: unknown) => Promise<GitResultDto>,
): GitClient {
  return {
    dispatch: dispatchFn ?? vi.fn().mockImplementation((cmd: { kind: string }) => {
      if (cmd.kind === "status") {
        return Promise.resolve({
          kind: "status" as const,
          files: [
            { path: "src/main.ts", oldPath: null, indexStatus: "modified", worktreeStatus: "modified" },
          ],
        });
      }
      if (cmd.kind === "repoState") {
        return Promise.resolve({
          kind: "repoState" as const,
          state: { repoPath: "/repo", headBranch: "main", isDetached: false, statusClean: false },
        });
      }
      if (cmd.kind === "diff") {
        return Promise.resolve({ kind: "diff" as const, diffs: [] });
      }
      return Promise.resolve({ kind: cmd.kind });
    }),
  };
}

describe("GitPane", () => {
  it("renders without crash and shows loading state initially", () => {
    const client = makeMockGitClient(
      () => new Promise(() => {}),
    );

    render(<GitPane pane={makePaneSnapshot()} gitClient={client} />);

    expect(screen.getByTestId("git-loading")).toBeInTheDocument();
  });

  it("shows file list after loading completes", async () => {
    const client = makeMockGitClient();

    render(<GitPane pane={makePaneSnapshot()} gitClient={client} />);

    await waitFor(() => {
      expect(screen.getByTestId("git-pane")).toBeInTheDocument();
    });

    expect(screen.getByTestId("git-file-list")).toBeInTheDocument();
    expect(screen.getAllByText("src/main.ts").length).toBeGreaterThanOrEqual(1);
  });

  it("shows error state on failure", async () => {
    const client = makeMockGitClient(
      () => Promise.reject(new Error("git not found")),
    );

    render(<GitPane pane={makePaneSnapshot()} gitClient={client} />);

    await waitFor(() => {
      expect(screen.getByTestId("pane-error-state")).toBeInTheDocument();
    });

    expect(screen.getByText("git not found")).toBeInTheDocument();
  });

  it("shows branch name from repo state", async () => {
    const client = makeMockGitClient();

    render(<GitPane pane={makePaneSnapshot()} gitClient={client} />);

    await waitFor(() => {
      expect(screen.getByText("main")).toBeInTheDocument();
    });
  });

  it("renders commit area and diff area", async () => {
    const client = makeMockGitClient();

    render(<GitPane pane={makePaneSnapshot()} gitClient={client} />);

    await waitFor(() => {
      expect(screen.getByTestId("git-commit-area")).toBeInTheDocument();
    });

    expect(screen.getByTestId("git-diff-area")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("Commit message...")).toBeInTheDocument();
  });

  it("renders view tab buttons", async () => {
    const client = makeMockGitClient();

    render(<GitPane pane={makePaneSnapshot()} gitClient={client} />);

    await waitFor(() => {
      expect(screen.getByTestId("git-pane")).toBeInTheDocument();
    });

    expect(screen.getAllByText("Changes").length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText("History")).toBeInTheDocument();
    expect(screen.getByText("Branches")).toBeInTheDocument();
    expect(screen.getByText("Stash")).toBeInTheDocument();
  });
});
