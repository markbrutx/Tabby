import { describe, it, expect } from "vitest";
import type { GitCommandDto } from "@/contracts/tauri-bindings";
import { createMockGitClient } from "./mockGitClient";

const PANE_ID = "test-pane";

describe("createMockGitClient", () => {
  const client = createMockGitClient();

  const commandCases: Array<{ command: GitCommandDto; expectedKind: string }> = [
    { command: { kind: "status", pane_id: PANE_ID }, expectedKind: "status" },
    { command: { kind: "diff", pane_id: PANE_ID, path: null, staged: false }, expectedKind: "diff" },
    { command: { kind: "stage", pane_id: PANE_ID, paths: ["file.ts"] }, expectedKind: "stage" },
    { command: { kind: "unstage", pane_id: PANE_ID, paths: ["file.ts"] }, expectedKind: "unstage" },
    { command: { kind: "stageLines", pane_id: PANE_ID, path: "file.ts", line_ranges: ["1-3"] }, expectedKind: "stageLines" },
    { command: { kind: "commit", pane_id: PANE_ID, message: "test commit", amend: false }, expectedKind: "commit" },
    { command: { kind: "push", pane_id: PANE_ID, remote: null, branch: null }, expectedKind: "push" },
    { command: { kind: "pull", pane_id: PANE_ID, remote: null, branch: null }, expectedKind: "pull" },
    { command: { kind: "fetch", pane_id: PANE_ID, remote: null }, expectedKind: "fetch" },
    { command: { kind: "branches", pane_id: PANE_ID }, expectedKind: "branches" },
    { command: { kind: "checkoutBranch", pane_id: PANE_ID, name: "main" }, expectedKind: "checkoutBranch" },
    { command: { kind: "createBranch", pane_id: PANE_ID, name: "feat", start_point: null }, expectedKind: "createBranch" },
    { command: { kind: "deleteBranch", pane_id: PANE_ID, name: "old", force: false }, expectedKind: "deleteBranch" },
    { command: { kind: "mergeBranch", pane_id: PANE_ID, name: "feature" }, expectedKind: "mergeBranch" },
    { command: { kind: "log", pane_id: PANE_ID, max_count: null, skip: null, path: null }, expectedKind: "log" },
    { command: { kind: "showCommit", pane_id: PANE_ID, hash: "abc123" }, expectedKind: "showCommit" },
    { command: { kind: "blame", pane_id: PANE_ID, path: "file.ts" }, expectedKind: "blame" },
    { command: { kind: "stashPush", pane_id: PANE_ID, message: null }, expectedKind: "stashPush" },
    { command: { kind: "stashPop", pane_id: PANE_ID, index: null }, expectedKind: "stashPop" },
    { command: { kind: "stashList", pane_id: PANE_ID }, expectedKind: "stashList" },
    { command: { kind: "stashDrop", pane_id: PANE_ID, index: 0 }, expectedKind: "stashDrop" },
    { command: { kind: "discardChanges", pane_id: PANE_ID, paths: ["file.ts"] }, expectedKind: "discardChanges" },
    { command: { kind: "repoState", pane_id: PANE_ID }, expectedKind: "repoState" },
  ];

  it.each(commandCases)(
    "returns $expectedKind result for $expectedKind command",
    async ({ command, expectedKind }) => {
      const result = await client.dispatch(command);
      expect(result.kind).toBe(expectedKind);
    },
  );

  it("returns file status entries for status command", async () => {
    const result = await client.dispatch({ kind: "status", pane_id: PANE_ID });
    if (result.kind !== "status") throw new Error("unexpected kind");
    expect(result.files.length).toBeGreaterThan(0);
    expect(result.files[0]).toHaveProperty("path");
    expect(result.files[0]).toHaveProperty("indexStatus");
    expect(result.files[0]).toHaveProperty("worktreeStatus");
  });

  it("returns diff hunks with lines for diff command", async () => {
    const result = await client.dispatch({ kind: "diff", pane_id: PANE_ID, path: "test.ts", staged: false });
    if (result.kind !== "diff") throw new Error("unexpected kind");
    expect(result.diffs.length).toBeGreaterThan(0);
    expect(result.diffs[0].hunks.length).toBeGreaterThan(0);
    expect(result.diffs[0].hunks[0].lines.length).toBeGreaterThan(0);
  });

  it("returns branch list with current branch for branches command", async () => {
    const result = await client.dispatch({ kind: "branches", pane_id: PANE_ID });
    if (result.kind !== "branches") throw new Error("unexpected kind");
    expect(result.branches.length).toBeGreaterThan(0);
    const current = result.branches.find((b) => b.isCurrent);
    expect(current).toBeDefined();
  });

  it("returns commit log entries for log command", async () => {
    const result = await client.dispatch({ kind: "log", pane_id: PANE_ID, max_count: null, skip: null, path: null });
    if (result.kind !== "log") throw new Error("unexpected kind");
    expect(result.commits.length).toBeGreaterThan(0);
    expect(result.commits[0]).toHaveProperty("hash");
    expect(result.commits[0]).toHaveProperty("shortHash");
    expect(result.commits[0]).toHaveProperty("authorName");
    expect(result.commits[0]).toHaveProperty("message");
  });

  it("returns stash entries for stashList command", async () => {
    const result = await client.dispatch({ kind: "stashList", pane_id: PANE_ID });
    if (result.kind !== "stashList") throw new Error("unexpected kind");
    expect(result.entries.length).toBeGreaterThan(0);
    expect(result.entries[0]).toHaveProperty("index");
    expect(result.entries[0]).toHaveProperty("message");
  });

  it("returns repo state for repoState command", async () => {
    const result = await client.dispatch({ kind: "repoState", pane_id: PANE_ID });
    if (result.kind !== "repoState") throw new Error("unexpected kind");
    expect(result.state).toHaveProperty("repoPath");
    expect(result.state).toHaveProperty("headBranch");
    expect(result.state).toHaveProperty("isDetached");
    expect(result.state).toHaveProperty("statusClean");
  });

  it("returns a commit hash for commit command", async () => {
    const result = await client.dispatch({ kind: "commit", pane_id: PANE_ID, message: "test", amend: false });
    if (result.kind !== "commit") throw new Error("unexpected kind");
    expect(result.hash).toBeTruthy();
  });

  it("returns blame entries for blame command", async () => {
    const result = await client.dispatch({ kind: "blame", pane_id: PANE_ID, path: "test.ts" });
    if (result.kind !== "blame") throw new Error("unexpected kind");
    expect(result.entries.length).toBeGreaterThan(0);
    expect(result.entries[0]).toHaveProperty("hash");
    expect(result.entries[0]).toHaveProperty("author");
  });

  it("returns merge message for mergeBranch command", async () => {
    const result = await client.dispatch({ kind: "mergeBranch", pane_id: PANE_ID, name: "feature" });
    if (result.kind !== "mergeBranch") throw new Error("unexpected kind");
    expect(result.message).toBeTruthy();
  });

  it("uses provided path in diff command", async () => {
    const result = await client.dispatch({ kind: "diff", pane_id: PANE_ID, path: "custom/path.ts", staged: true });
    if (result.kind !== "diff") throw new Error("unexpected kind");
    expect(result.diffs[0].filePath).toBe("custom/path.ts");
  });
});
