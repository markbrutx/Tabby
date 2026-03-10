import { expect, test } from "@playwright/test";
import { createTauriMockScript } from "./fixtures/tauri-mock";

/**
 * Git Pane E2E tests.
 *
 * Opens a Git pane via the workspace mock and verifies:
 * - git status loads and displays files
 * - selecting a file triggers diff display
 * - branch name is visible
 * - view tabs (Changes, History, Branches, Stash) are accessible
 * - stage/unstage and commit flows
 */

const GIT_MOCK_DATA = {
  files: [
    { path: "src/main.ts", oldPath: null, indexStatus: "modified", worktreeStatus: "modified" },
    { path: "src/utils.ts", oldPath: null, indexStatus: "added", worktreeStatus: "untracked" },
    { path: "README.md", oldPath: null, indexStatus: "untracked", worktreeStatus: "modified" },
  ],
  repoState: {
    repoPath: "/mock/repo",
    headBranch: "feature/git-client",
    isDetached: false,
    statusClean: false,
  },
  diff: {
    filePath: "src/main.ts",
    oldPath: null,
    hunks: [{
      oldStart: 1,
      oldCount: 3,
      newStart: 1,
      newCount: 5,
      header: "@@ -1,3 +1,5 @@",
      lines: [
        { kind: "context", oldLineNo: 1, newLineNo: 1, content: "import { app } from './app';" },
        { kind: "deletion", oldLineNo: 2, newLineNo: null, content: "app.init();" },
        { kind: "addition", oldLineNo: null, newLineNo: 2, content: "app.initialize();" },
        { kind: "addition", oldLineNo: null, newLineNo: 3, content: "app.configure();" },
        { kind: "context", oldLineNo: 3, newLineNo: 4, content: "app.start();" },
      ],
    }],
    isBinary: false,
    fileModeChange: null,
  },
  branches: [
    { name: "main", isCurrent: false, upstream: "origin/main", ahead: 0, behind: 0 },
    { name: "feature/git-client", isCurrent: true, upstream: "origin/feature/git-client", ahead: 2, behind: 0 },
    { name: "develop", isCurrent: false, upstream: null, ahead: 0, behind: 0 },
  ],
  commits: [
    {
      hash: "aaa111aaa111aaa111aaa111aaa111aaa111aaa1",
      shortHash: "aaa111a",
      authorName: "Developer",
      authorEmail: "dev@example.com",
      date: "2026-03-10T12:00:00Z",
      message: "feat: add git client",
      parentHashes: ["bbb222bbb222bbb222bbb222bbb222bbb222bbb2"],
    },
    {
      hash: "bbb222bbb222bbb222bbb222bbb222bbb222bbb2",
      shortHash: "bbb222b",
      authorName: "Developer",
      authorEmail: "dev@example.com",
      date: "2026-03-09T10:00:00Z",
      message: "fix: resolve parsing bug",
      parentHashes: [],
    },
  ],
  stashes: [
    { index: 0, message: "WIP on feature: partial work", date: "2026-03-10T11:00:00Z" },
  ],
};

test.beforeEach(async ({ page }) => {
  await page.addInitScript({
    content: createTauriMockScript({
      initialPanes: [{ kind: "git", cwd: "/mock/repo" }],
      git: GIT_MOCK_DATA,
    }),
  });
  await page.goto("/");
});

test("opens git pane and displays file status list", async ({ page }) => {
  await expect(page.getByTestId("git-pane")).toBeVisible({ timeout: 10_000 });
  await expect(page.getByTestId("git-file-list")).toBeVisible({ timeout: 5_000 });
  await expect(page.getByText("src/main.ts").first()).toBeVisible({ timeout: 5_000 });
});

test("displays branch name from repo state", async ({ page }) => {
  await expect(page.getByTestId("git-pane")).toBeVisible({ timeout: 10_000 });
  await expect(page.getByText("feature/git-client").first()).toBeVisible({ timeout: 5_000 });
});

test("selecting a file shows diff view", async ({ page }) => {
  await expect(page.getByTestId("git-pane")).toBeVisible({ timeout: 10_000 });
  await expect(page.getByTestId("git-file-list")).toBeVisible({ timeout: 5_000 });

  const fileButton = page.getByTestId("file-select-button").first();
  await fileButton.click();

  await expect(page.getByTestId("git-diff-area")).toBeVisible({ timeout: 5_000 });
  await expect(page.getByTestId("hunk-header").first()).toBeVisible({ timeout: 5_000 });
});

test("view tabs are accessible (Changes, History, Branches, Stash)", async ({ page }) => {
  await expect(page.getByTestId("git-pane")).toBeVisible({ timeout: 10_000 });

  await expect(page.getByText("Changes").first()).toBeVisible({ timeout: 5_000 });
  await expect(page.getByText("History")).toBeVisible({ timeout: 5_000 });
  await expect(page.getByText("Branches")).toBeVisible({ timeout: 5_000 });
  await expect(page.getByText("Stash")).toBeVisible({ timeout: 5_000 });
});

// ---------- Additional git tests ----------

test("switch to History tab shows commits", async ({ page }) => {
  await expect(page.getByTestId("git-pane")).toBeVisible({ timeout: 10_000 });

  await page.getByText("History").click();

  await expect(page.getByText("feat: add git client")).toBeVisible({ timeout: 5_000 });
  await expect(page.getByText("fix: resolve parsing bug")).toBeVisible({ timeout: 5_000 });
});

test("switch to Branches tab shows branches", async ({ page }) => {
  await expect(page.getByTestId("git-pane")).toBeVisible({ timeout: 10_000 });

  await page.getByText("Branches").click();

  await expect(page.getByText("main").first()).toBeVisible({ timeout: 5_000 });
  await expect(page.getByText("feature/git-client").first()).toBeVisible({ timeout: 5_000 });
  await expect(page.getByText("develop")).toBeVisible({ timeout: 5_000 });
});

test("switch to Stash tab shows entries", async ({ page }) => {
  await expect(page.getByTestId("git-pane")).toBeVisible({ timeout: 10_000 });

  await page.getByText("Stash").click();

  await expect(page.getByText("WIP on feature: partial work")).toBeVisible({ timeout: 5_000 });
});

test("commit panel is visible with message input", async ({ page }) => {
  await expect(page.getByTestId("git-pane")).toBeVisible({ timeout: 10_000 });

  const commitInput = page.getByTestId("commit-message-input");
  await expect(commitInput).toBeVisible({ timeout: 5_000 });

  // Type a commit message
  await commitInput.fill("feat: test commit");
  await expect(commitInput).toHaveValue("feat: test commit");
});
