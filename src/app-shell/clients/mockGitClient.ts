import type { GitCommandDto, GitResultDto } from "@/contracts/tauri-bindings";
import type { GitClient } from "./shared";

function buildMockResult(command: GitCommandDto): GitResultDto {
  switch (command.kind) {
    case "status":
      return {
        kind: "status",
        files: [
          {
            path: "src/main.ts",
            oldPath: null,
            indexStatus: "modified",
            worktreeStatus: "modified",
          },
          {
            path: "README.md",
            oldPath: null,
            indexStatus: "untracked",
            worktreeStatus: "untracked",
          },
          {
            path: "src/utils/helper.ts",
            oldPath: null,
            indexStatus: "added",
            worktreeStatus: "untracked",
          },
        ],
      };
    case "diff":
      return {
        kind: "diff",
        diffs: [
          {
            filePath: command.path ?? "src/main.ts",
            oldPath: null,
            hunks: [
              {
                oldStart: 1,
                oldCount: 3,
                newStart: 1,
                newCount: 4,
                header: "@@ -1,3 +1,4 @@",
                lines: [
                  { kind: "context", oldLineNo: 1, newLineNo: 1, content: "import { app } from './app';" },
                  { kind: "deletion", oldLineNo: 2, newLineNo: null, content: "app.start();" },
                  { kind: "addition", oldLineNo: null, newLineNo: 2, content: "app.init();" },
                  { kind: "addition", oldLineNo: null, newLineNo: 3, content: "app.start();" },
                  { kind: "context", oldLineNo: 3, newLineNo: 4, content: "" },
                ],
              },
            ],
            isBinary: false,
            fileModeChange: null,
          },
        ],
      };
    case "stage":
      return { kind: "stage" };
    case "unstage":
      return { kind: "unstage" };
    case "stageLines":
      return { kind: "stageLines" };
    case "commit":
      return { kind: "commit", hash: "a1b2c3d4e5f6" };
    case "push":
      return { kind: "push" };
    case "pull":
      return { kind: "pull" };
    case "fetch":
      return { kind: "fetch" };
    case "branches":
      return {
        kind: "branches",
        branches: [
          { name: "main", isCurrent: true, upstream: "origin/main", ahead: 1, behind: 0 },
          { name: "feature/git-client", isCurrent: false, upstream: "origin/feature/git-client", ahead: 0, behind: 2 },
          { name: "develop", isCurrent: false, upstream: "origin/develop", ahead: 0, behind: 0 },
        ],
      };
    case "checkoutBranch":
      return { kind: "checkoutBranch" };
    case "createBranch":
      return { kind: "createBranch" };
    case "deleteBranch":
      return { kind: "deleteBranch" };
    case "mergeBranch":
      return { kind: "mergeBranch", message: "Merge branch 'feature/git-client' into main" };
    case "log":
      return {
        kind: "log",
        commits: [
          {
            hash: "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2",
            shortHash: "a1b2c3d",
            authorName: "Developer",
            authorEmail: "dev@example.com",
            date: "2026-03-10T12:00:00Z",
            message: "feat: add git client transport",
            parentHashes: ["b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3"],
          },
          {
            hash: "b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3",
            shortHash: "b2c3d4e",
            authorName: "Developer",
            authorEmail: "dev@example.com",
            date: "2026-03-09T10:30:00Z",
            message: "fix: resolve merge conflict",
            parentHashes: ["c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4"],
          },
        ],
      };
    case "blame":
      return {
        kind: "blame",
        entries: [
          { hash: "a1b2c3d", author: "Developer", date: "2026-03-10", lineStart: 1, lineCount: 5, content: "import { app } from './app';" },
          { hash: "c3d4e5f", author: "Contributor", date: "2026-03-08", lineStart: 6, lineCount: 3, content: "app.start();" },
        ],
      };
    case "stashPush":
      return { kind: "stashPush" };
    case "stashPop":
      return { kind: "stashPop" };
    case "stashList":
      return {
        kind: "stashList",
        entries: [
          { index: 0, message: "WIP on main: work in progress", date: "2026-03-10T11:00:00Z" },
          { index: 1, message: "WIP on feature: partial implementation", date: "2026-03-09T15:00:00Z" },
        ],
      };
    case "stashDrop":
      return { kind: "stashDrop" };
    case "discardChanges":
      return { kind: "discardChanges" };
    case "repoState":
      return {
        kind: "repoState",
        state: {
          repoPath: "/mock/repo",
          headBranch: "main",
          isDetached: false,
          statusClean: false,
        },
      };
  }
}

export function createMockGitClient(): GitClient {
  return {
    async dispatch(command) {
      return buildMockResult(command);
    },
  };
}
