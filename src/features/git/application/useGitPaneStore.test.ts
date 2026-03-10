import { describe, expect, it, vi, beforeEach } from "vitest";
import { createGitPaneStore, type GitPaneState } from "./useGitPaneStore";
import type { GitClient } from "@/app-shell/clients";
import type { FileStatusDto, GitResultDto } from "@/contracts/tauri-bindings";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function makeStatusResult(
  files: FileStatusDto[] = [],
): GitResultDto {
  return {
    kind: "status" as const,
    files,
  };
}

function makeRepoStateResult(
  overrides: Partial<{
    repoPath: string;
    headBranch: string | null;
    isDetached: boolean;
    statusClean: boolean;
  }> = {},
): GitResultDto {
  return {
    kind: "repoState" as const,
    state: {
      repoPath: "/repo",
      headBranch: "main",
      isDetached: false,
      statusClean: false,
      ...overrides,
    },
  } as GitResultDto;
}

function makeDiffResult(
  diffs: Array<{
    filePath: string;
    oldPath: string | null;
    hunks: Array<unknown>;
    isBinary: boolean;
    fileModeChange: string | null;
  }> = [],
): GitResultDto {
  return { kind: "diff" as const, diffs } as GitResultDto;
}

function makeBranchesResult(
  branches: Array<{
    name: string;
    isCurrent: boolean;
    upstream: string | null;
    ahead: number;
    behind: number;
  }> = [],
): GitResultDto {
  return { kind: "branches" as const, branches } as GitResultDto;
}

function makeLogResult(
  commits: Array<{
    hash: string;
    shortHash: string;
    authorName: string;
    authorEmail: string;
    date: string;
    message: string;
    parentHashes: string[];
  }> = [],
): GitResultDto {
  return { kind: "log" as const, commits } as GitResultDto;
}

function makeBlameResult(
  entries: Array<{
    hash: string;
    author: string;
    date: string;
    lineStart: number;
    lineCount: number;
    content: string;
  }> = [],
): GitResultDto {
  return { kind: "blame" as const, entries } as GitResultDto;
}

function makeStashListResult(
  entries: Array<{ index: number; message: string; date: string }> = [],
): GitResultDto {
  return { kind: "stashList" as const, entries } as GitResultDto;
}

function makeCommitResult(hash: string): GitResultDto {
  return { kind: "commit" as const, hash } as GitResultDto;
}

function makeShowCommitResult(
  diffs: Array<{
    filePath: string;
    oldPath: string | null;
    hunks: Array<{
      oldStart: number;
      oldCount: number;
      newStart: number;
      newCount: number;
      header: string;
      lines: Array<{
        kind: string;
        oldLineNo: number | null;
        newLineNo: number | null;
        content: string;
      }>;
    }>;
    isBinary: boolean;
    fileModeChange: string | null;
  }> = [],
): GitResultDto {
  return { kind: "showCommit" as const, diffs } as GitResultDto;
}

const SAMPLE_FILE = {
  path: "src/main.ts",
  oldPath: null,
  indexStatus: "modified" as const,
  worktreeStatus: "modified" as const,
};

const SAMPLE_COMMIT = {
  hash: "abc123def456abc123def456abc123def456abc1",
  shortHash: "abc123d",
  authorName: "Test Author",
  authorEmail: "test@example.com",
  date: "2026-03-10T12:00:00Z",
  message: "feat: test commit",
  parentHashes: [] as string[],
};

let mockDispatch: ReturnType<typeof vi.fn>;
let gitClient: GitClient;
let store: ReturnType<typeof createGitPaneStore>;

function getState(): GitPaneState {
  return store.getState();
}

beforeEach(() => {
  mockDispatch = vi.fn();
  gitClient = { dispatch: mockDispatch };
  store = createGitPaneStore({ gitClient, paneId: "test-pane" });
});

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("createGitPaneStore", () => {
  it("has correct initial state", () => {
    const state = getState();
    expect(state.files).toEqual([]);
    expect(state.selectedFile).toBeNull();
    expect(state.diffContent).toBeNull();
    expect(state.repoState).toBeNull();
    expect(state.activeView).toBe("changes");
    expect(state.loading).toBe(true);
    expect(state.error).toBeNull();
    expect(state.branches).toEqual([]);
    expect(state.commitLog).toEqual([]);
    expect(state.hasMoreCommits).toBe(true);
    expect(state.stashes).toEqual([]);
  });

  describe("refreshStatus", () => {
    it("fetches status and repo state in parallel", async () => {
      mockDispatch
        .mockResolvedValueOnce(makeStatusResult([SAMPLE_FILE]))
        .mockResolvedValueOnce(makeRepoStateResult());

      await getState().refreshStatus();

      expect(mockDispatch).toHaveBeenCalledTimes(2);
      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({ kind: "status", pane_id: "test-pane" }),
      );
      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({ kind: "repoState", pane_id: "test-pane" }),
      );

      const state = getState();
      expect(state.files).toHaveLength(1);
      expect(state.files[0].path).toBe("src/main.ts");
      expect(state.repoState?.headBranch).toBe("main");
      expect(state.loading).toBe(false);
      expect(state.error).toBeNull();
    });

    it("sets error on failure", async () => {
      mockDispatch.mockRejectedValue(new Error("network error"));

      await getState().refreshStatus();

      const state = getState();
      expect(state.error).toBe("network error");
      expect(state.loading).toBe(false);
    });

    it("handles non-Error rejection", async () => {
      mockDispatch.mockRejectedValue("string error");

      await getState().refreshStatus();

      expect(getState().error).toBe("Failed to refresh status");
    });
  });

  describe("selectFile", () => {
    it("dispatches diff command and sets diffContent", async () => {
      const diff = {
        filePath: "src/main.ts",
        oldPath: null,
        hunks: [],
        isBinary: false,
        fileModeChange: null,
      };
      mockDispatch.mockResolvedValueOnce(makeDiffResult([diff]));

      await getState().selectFile("src/main.ts");

      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({
          kind: "diff",
          pane_id: "test-pane",
          path: "src/main.ts",
          staged: false,
        }),
      );

      const state = getState();
      expect(state.selectedFile).toBe("src/main.ts");
      expect(state.diffContent).toEqual(diff);
    });

    it("clears selection when null", async () => {
      await getState().selectFile(null);

      expect(mockDispatch).not.toHaveBeenCalled();
      expect(getState().selectedFile).toBeNull();
      expect(getState().diffContent).toBeNull();
    });

    it("sets error on diff failure", async () => {
      mockDispatch.mockRejectedValue(new Error("diff failed"));

      await getState().selectFile("file.ts");

      expect(getState().error).toBe("diff failed");
    });
  });

  describe("setActiveView", () => {
    it("updates active view", () => {
      getState().setActiveView("history");
      expect(getState().activeView).toBe("history");

      getState().setActiveView("branches");
      expect(getState().activeView).toBe("branches");
    });
  });

  describe("stageFiles", () => {
    it("dispatches stage then refreshes status", async () => {
      mockDispatch
        .mockResolvedValueOnce({ kind: "stage" } as GitResultDto)
        .mockResolvedValueOnce(makeStatusResult([]));

      await getState().stageFiles(["src/main.ts"]);

      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({
          kind: "stage",
          pane_id: "test-pane",
          paths: ["src/main.ts"],
        }),
      );
      expect(mockDispatch).toHaveBeenCalledTimes(2);
    });

    it("sets error on stage failure", async () => {
      mockDispatch.mockRejectedValue(new Error("stage failed"));

      await getState().stageFiles(["file.ts"]);

      expect(getState().error).toBe("stage failed");
    });
  });

  describe("unstageFiles", () => {
    it("dispatches unstage then refreshes status", async () => {
      mockDispatch
        .mockResolvedValueOnce({ kind: "unstage" } as GitResultDto)
        .mockResolvedValueOnce(makeStatusResult([]));

      await getState().unstageFiles(["file.ts"]);

      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({ kind: "unstage", paths: ["file.ts"] }),
      );
    });
  });

  describe("discardChanges", () => {
    it("dispatches discardChanges then refreshes status", async () => {
      mockDispatch
        .mockResolvedValueOnce({ kind: "discardChanges" } as GitResultDto)
        .mockResolvedValueOnce(makeStatusResult([]));

      await getState().discardChanges(["file.ts"]);

      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({ kind: "discardChanges", paths: ["file.ts"] }),
      );
    });

    it("sets error on discard failure", async () => {
      mockDispatch.mockRejectedValue(new Error("discard failed"));

      await getState().discardChanges(["file.ts"]);

      expect(getState().error).toBe("discard failed");
    });
  });

  describe("stageLines", () => {
    it("dispatches stageLines then refreshes status and diff", async () => {
      mockDispatch
        .mockResolvedValueOnce({ kind: "stageLines" } as GitResultDto)
        .mockResolvedValueOnce(makeStatusResult([]))
        .mockResolvedValueOnce(makeDiffResult([]));

      await getState().stageLines("file.ts", ["1-5"]);

      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({
          kind: "stageLines",
          path: "file.ts",
          line_ranges: ["1-5"],
        }),
      );
      expect(getState().stagedLines.size).toBe(0);
    });
  });

  describe("commit", () => {
    it("dispatches commit with message and amend flag", async () => {
      mockDispatch.mockResolvedValueOnce(makeCommitResult("abc1234"));

      await getState().commit("feat: test", false);

      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({
          kind: "commit",
          pane_id: "test-pane",
          message: "feat: test",
          amend: false,
        }),
      );
    });

    it("dispatches commit with amend=true", async () => {
      mockDispatch.mockResolvedValueOnce(makeCommitResult("abc1234"));

      await getState().commit("fix: amend", true);

      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({ amend: true }),
      );
    });

    it("throws on unexpected result kind", async () => {
      mockDispatch.mockResolvedValueOnce({ kind: "status" } as GitResultDto);

      await expect(getState().commit("msg", false)).rejects.toThrow(
        "Unexpected commit result",
      );
    });
  });

  describe("fetchLastCommitInfo", () => {
    it("returns commit info from log with max_count=1", async () => {
      mockDispatch.mockResolvedValueOnce(makeLogResult([SAMPLE_COMMIT]));

      const result = await getState().fetchLastCommitInfo();

      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({
          kind: "log",
          max_count: 1,
        }),
      );
      expect(result).not.toBeNull();
      expect(result?.message).toBe("feat: test commit");
    });

    it("returns null when no commits", async () => {
      mockDispatch.mockResolvedValueOnce(makeLogResult([]));

      const result = await getState().fetchLastCommitInfo();

      expect(result).toBeNull();
    });
  });

  describe("listBranches", () => {
    it("fetches and stores branches", async () => {
      const branches = [
        { name: "main", isCurrent: true, upstream: "origin/main", ahead: 1, behind: 0 },
        { name: "feature", isCurrent: false, upstream: null, ahead: 0, behind: 0 },
      ];
      mockDispatch.mockResolvedValueOnce(makeBranchesResult(branches));

      await getState().listBranches();

      const state = getState();
      expect(state.branches).toHaveLength(2);
      expect(state.branches[0].name).toBe("main");
      expect(state.branchesLoading).toBe(false);
    });

    it("sets loading while fetching", async () => {
      let resolvePromise: (v: GitResultDto) => void;
      const promise = new Promise<GitResultDto>((resolve) => {
        resolvePromise = resolve;
      });
      mockDispatch.mockReturnValueOnce(promise);

      const listPromise = getState().listBranches();
      expect(getState().branchesLoading).toBe(true);

      resolvePromise!(makeBranchesResult([]));
      await listPromise;

      expect(getState().branchesLoading).toBe(false);
    });

    it("sets error on failure", async () => {
      mockDispatch.mockRejectedValue(new Error("branch error"));

      await getState().listBranches();

      expect(getState().error).toBe("branch error");
      expect(getState().branchesLoading).toBe(false);
    });
  });

  describe("checkoutBranch", () => {
    it("dispatches checkout then refreshes branches and status", async () => {
      mockDispatch
        .mockResolvedValueOnce({ kind: "checkoutBranch" } as GitResultDto)
        .mockResolvedValueOnce(makeBranchesResult([]))
        .mockResolvedValueOnce(makeStatusResult([]))
        .mockResolvedValueOnce(makeRepoStateResult());

      await getState().checkoutBranch("feature");

      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({ kind: "checkoutBranch", name: "feature" }),
      );
    });

    it("sets error on checkout failure", async () => {
      mockDispatch.mockRejectedValue(new Error("checkout failed"));

      await getState().checkoutBranch("bad-branch");

      expect(getState().error).toBe("checkout failed");
    });
  });

  describe("createBranch", () => {
    it("dispatches createBranch with name and start point", async () => {
      mockDispatch
        .mockResolvedValueOnce({ kind: "createBranch" } as GitResultDto)
        .mockResolvedValueOnce(makeBranchesResult([]))
        .mockResolvedValueOnce(makeStatusResult([]))
        .mockResolvedValueOnce(makeRepoStateResult());

      await getState().createBranch("feat/new", "main");

      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({
          kind: "createBranch",
          name: "feat/new",
          start_point: "main",
        }),
      );
    });

    it("dispatches createBranch with null start point", async () => {
      mockDispatch
        .mockResolvedValueOnce({ kind: "createBranch" } as GitResultDto)
        .mockResolvedValueOnce(makeBranchesResult([]))
        .mockResolvedValueOnce(makeStatusResult([]))
        .mockResolvedValueOnce(makeRepoStateResult());

      await getState().createBranch("hotfix", null);

      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({ start_point: null }),
      );
    });
  });

  describe("deleteBranch", () => {
    it("dispatches deleteBranch then refreshes branches", async () => {
      mockDispatch
        .mockResolvedValueOnce({ kind: "deleteBranch" } as GitResultDto)
        .mockResolvedValueOnce(makeBranchesResult([]));

      await getState().deleteBranch("old-branch", false);

      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({
          kind: "deleteBranch",
          name: "old-branch",
          force: false,
        }),
      );
    });

    it("passes force flag", async () => {
      mockDispatch
        .mockResolvedValueOnce({ kind: "deleteBranch" } as GitResultDto)
        .mockResolvedValueOnce(makeBranchesResult([]));

      await getState().deleteBranch("force-branch", true);

      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({ force: true }),
      );
    });
  });

  describe("fetchCommitLog", () => {
    it("fetches commits with max_count=50", async () => {
      const commits = Array.from({ length: 50 }, (_, i) => ({
        ...SAMPLE_COMMIT,
        hash: `hash${i}${"0".repeat(34)}`,
        shortHash: `h${i}`,
        message: `commit ${i}`,
      }));
      mockDispatch.mockResolvedValueOnce(makeLogResult(commits));

      await getState().fetchCommitLog();

      const state = getState();
      expect(state.commitLog).toHaveLength(50);
      expect(state.hasMoreCommits).toBe(true);
      expect(state.commitLogLoading).toBe(false);
    });

    it("sets hasMoreCommits=false when fewer than 50 returned", async () => {
      mockDispatch.mockResolvedValueOnce(makeLogResult([SAMPLE_COMMIT]));

      await getState().fetchCommitLog();

      expect(getState().hasMoreCommits).toBe(false);
    });

    it("sets error on failure", async () => {
      mockDispatch.mockRejectedValue(new Error("log error"));

      await getState().fetchCommitLog();

      expect(getState().error).toBe("log error");
      expect(getState().commitLogLoading).toBe(false);
    });
  });

  describe("fetchMoreCommits", () => {
    it("appends to existing commit log with skip offset", async () => {
      // First, load initial commits
      mockDispatch.mockResolvedValueOnce(
        makeLogResult([{ ...SAMPLE_COMMIT, message: "first" }]),
      );
      await getState().fetchCommitLog();

      // Override hasMoreCommits since we got < 50
      // Need to manually set state to test fetchMoreCommits
      store.setState({ hasMoreCommits: true });

      mockDispatch.mockResolvedValueOnce(
        makeLogResult([{ ...SAMPLE_COMMIT, hash: "bbb" + "0".repeat(37), message: "second" }]),
      );
      await getState().fetchMoreCommits();

      expect(getState().commitLog).toHaveLength(2);
      expect(mockDispatch).toHaveBeenLastCalledWith(
        expect.objectContaining({ kind: "log", skip: 1 }),
      );
    });

    it("does nothing when hasMoreCommits is false", async () => {
      store.setState({ hasMoreCommits: false });
      mockDispatch.mockClear();

      await getState().fetchMoreCommits();

      expect(mockDispatch).not.toHaveBeenCalled();
    });

    it("does nothing when already loading", async () => {
      store.setState({ commitLogLoading: true, hasMoreCommits: true });
      mockDispatch.mockClear();

      await getState().fetchMoreCommits();

      expect(mockDispatch).not.toHaveBeenCalled();
    });
  });

  describe("selectCommit", () => {
    it("fetches and sets commit diff content", async () => {
      const diff = {
        filePath: "src/main.ts",
        oldPath: null,
        hunks: [
          {
            oldStart: 1,
            oldCount: 2,
            newStart: 1,
            newCount: 3,
            header: "@@ -1,2 +1,3 @@",
            lines: [
              { kind: "context", oldLineNo: 1, newLineNo: 1, content: "line 1" },
              { kind: "addition", oldLineNo: null, newLineNo: 2, content: "new line" },
            ],
          },
        ],
        isBinary: false,
        fileModeChange: null,
      };
      mockDispatch.mockResolvedValueOnce(makeShowCommitResult([diff]));

      await getState().selectCommit("abc123");

      const state = getState();
      expect(state.selectedCommitHash).toBe("abc123");
      expect(state.commitDiffContent?.filePath).toBe("src/main.ts");
      expect(state.commitDiffContent?.hunks).toHaveLength(1);
    });

    it("clears selection when null", async () => {
      await getState().selectCommit(null);

      expect(getState().selectedCommitHash).toBeNull();
      expect(getState().commitDiffContent).toBeNull();
    });

    it("sets commitDiffContent to null when no diffs", async () => {
      mockDispatch.mockResolvedValueOnce(makeShowCommitResult([]));

      await getState().selectCommit("abc123");

      expect(getState().commitDiffContent).toBeNull();
    });
  });

  describe("fetchBlame", () => {
    it("fetches blame entries and sets active view", async () => {
      const entries = [
        { hash: "abc123", author: "Alice", date: "2026-03-10", lineStart: 1, lineCount: 5, content: "code" },
      ];
      mockDispatch.mockResolvedValueOnce(makeBlameResult(entries));

      await getState().fetchBlame("src/main.ts");

      const state = getState();
      expect(state.activeView).toBe("blame");
      expect(state.blameFilePath).toBe("src/main.ts");
      expect(state.blameEntries).toHaveLength(1);
      expect(state.blameEntries[0].author).toBe("Alice");
      expect(state.blameLoading).toBe(false);
    });

    it("sets error on blame failure", async () => {
      mockDispatch.mockRejectedValue(new Error("blame failed"));

      await getState().fetchBlame("file.ts");

      expect(getState().error).toBe("blame failed");
      expect(getState().blameLoading).toBe(false);
    });
  });

  describe("listStashes", () => {
    it("fetches and stores stash entries", async () => {
      const entries = [
        { index: 0, message: "WIP on main", date: "2026-03-10" },
        { index: 1, message: "WIP on feature", date: "2026-03-09" },
      ];
      mockDispatch.mockResolvedValueOnce(makeStashListResult(entries));

      await getState().listStashes();

      const state = getState();
      expect(state.stashes).toHaveLength(2);
      expect(state.stashes[0].message).toBe("WIP on main");
      expect(state.stashesLoading).toBe(false);
    });

    it("sets error on failure", async () => {
      mockDispatch.mockRejectedValue(new Error("stash error"));

      await getState().listStashes();

      expect(getState().error).toBe("stash error");
      expect(getState().stashesLoading).toBe(false);
    });
  });

  describe("stashPush", () => {
    it("dispatches stashPush with message then refreshes", async () => {
      mockDispatch
        .mockResolvedValueOnce({ kind: "stashPush" } as GitResultDto)
        .mockResolvedValueOnce(makeStashListResult([]))
        .mockResolvedValueOnce(makeStatusResult([]))
        .mockResolvedValueOnce(makeRepoStateResult());

      await getState().stashPush("my stash");

      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({ kind: "stashPush", message: "my stash" }),
      );
    });

    it("dispatches stashPush with null message", async () => {
      mockDispatch
        .mockResolvedValueOnce({ kind: "stashPush" } as GitResultDto)
        .mockResolvedValueOnce(makeStashListResult([]))
        .mockResolvedValueOnce(makeStatusResult([]))
        .mockResolvedValueOnce(makeRepoStateResult());

      await getState().stashPush(null);

      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({ message: null }),
      );
    });
  });

  describe("stashPop", () => {
    it("dispatches stashPop then refreshes", async () => {
      mockDispatch
        .mockResolvedValueOnce({ kind: "stashPop" } as GitResultDto)
        .mockResolvedValueOnce(makeStashListResult([]))
        .mockResolvedValueOnce(makeStatusResult([]))
        .mockResolvedValueOnce(makeRepoStateResult());

      await getState().stashPop(0);

      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({ kind: "stashPop", index: 0 }),
      );
    });
  });

  describe("stashApply", () => {
    it("dispatches apply (via stashPop) then refreshes", async () => {
      mockDispatch
        .mockResolvedValueOnce({ kind: "stashPop" } as GitResultDto)
        .mockResolvedValueOnce(makeStashListResult([]))
        .mockResolvedValueOnce(makeStatusResult([]))
        .mockResolvedValueOnce(makeRepoStateResult());

      await getState().stashApply(1);

      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({ kind: "stashPop", index: 1 }),
      );
    });
  });

  describe("stashDrop", () => {
    it("dispatches stashDrop then refreshes stash list", async () => {
      mockDispatch
        .mockResolvedValueOnce({ kind: "stashDrop" } as GitResultDto)
        .mockResolvedValueOnce(makeStashListResult([]));

      await getState().stashDrop(2);

      expect(mockDispatch).toHaveBeenCalledWith(
        expect.objectContaining({ kind: "stashDrop", index: 2 }),
      );
    });

    it("sets error on drop failure", async () => {
      mockDispatch.mockRejectedValue(new Error("drop failed"));

      await getState().stashDrop(0);

      expect(getState().error).toBe("drop failed");
    });
  });
});
