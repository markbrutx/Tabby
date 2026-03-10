import { expect, test } from "@playwright/test";

/**
 * Git Pane E2E test.
 *
 * Opens a Git pane via the workspace mock and verifies that:
 * - the git status loads and displays files
 * - selecting a file triggers diff display
 * - the branch name is visible
 * - view tabs (Changes, History, Branches, Stash) are accessible
 *
 * All data is mocked — no real git repository is used.
 */

function buildGitTauriMockScript(): string {
  return `
    (function () {
      // ---- state ----
      var profiles = [
        { id: "terminal", label: "Terminal", description: "Login shell", startupCommandTemplate: null },
      ];

      var settings = {
        defaultLayout: "1x1",
        defaultTerminalProfileId: "terminal",
        defaultWorkingDirectory: "~/projects",
        defaultCustomCommand: "",
        fontSize: 13,
        theme: "midnight",
        launchFullscreen: true,
        hasCompletedOnboarding: true,
        lastWorkingDirectory: null,
      };

      var tabCounter = 0;
      var paneCounter = 0;
      var sessionCounter = 0;
      var eventIdCounter = 0;
      var callbackRegistry = {};
      var eventListeners = {};

      function nextTabId() { return "tab-" + (++tabCounter); }
      function nextPaneId() { return "pane-" + (++paneCounter); }
      function nextSessionId() { return "session-" + (++sessionCounter); }

      function makePaneView(spec) {
        var paneId = nextPaneId();
        return { paneId: paneId, title: "Git", spec: spec };
      }

      function makePaneRuntime(paneId, kind) {
        return {
          paneId: paneId,
          runtimeSessionId: nextSessionId(),
          kind: kind || "terminal",
          status: "running",
          lastError: null,
          browserLocation: null,
          gitRepoPath: kind === "git" ? "/mock/repo" : null,
        };
      }

      function leafNode(paneId) {
        return { type: "pane", paneId: paneId };
      }

      // Start with a Git pane directly
      var gitSpec = { kind: "git", working_directory: "/mock/repo" };
      var gitPane = makePaneView(gitSpec);
      var tab = {
        tabId: nextTabId(),
        title: "Git Workspace",
        layout: leafNode(gitPane.paneId),
        panes: [gitPane],
        activePaneId: gitPane.paneId,
      };

      var workspace = { activeTabId: tab.tabId, tabs: [tab] };
      var runtimes = {};
      runtimes[gitPane.paneId] = makePaneRuntime(gitPane.paneId, "git");

      function currentView() {
        return JSON.parse(JSON.stringify(workspace));
      }

      function allRuntimes() {
        return Object.values(runtimes);
      }

      function emitEvent(eventName, payload) {
        var listeners = eventListeners[eventName] || [];
        listeners.forEach(function (entry) {
          var cb = callbackRegistry[entry.handlerId];
          if (cb) {
            cb({ event: eventName, id: entry.eventId, payload: payload });
          }
        });
      }

      // ---- Git mock data ----
      var mockFiles = [
        { path: "src/main.ts", oldPath: null, indexStatus: "modified", worktreeStatus: "modified" },
        { path: "src/utils.ts", oldPath: null, indexStatus: "added", worktreeStatus: "untracked" },
        { path: "README.md", oldPath: null, indexStatus: "untracked", worktreeStatus: "modified" },
      ];

      var mockRepoState = {
        repoPath: "/mock/repo",
        headBranch: "feature/git-client",
        isDetached: false,
        statusClean: false,
      };

      var mockDiff = {
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
      };

      var mockBranches = [
        { name: "main", isCurrent: false, upstream: "origin/main", ahead: 0, behind: 0 },
        { name: "feature/git-client", isCurrent: true, upstream: "origin/feature/git-client", ahead: 2, behind: 0 },
        { name: "develop", isCurrent: false, upstream: null, ahead: 0, behind: 0 },
      ];

      var mockCommits = [
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
      ];

      var mockStashes = [
        { index: 0, message: "WIP on feature: partial work", date: "2026-03-10T11:00:00Z" },
      ];

      function handleGitCommand(command) {
        switch (command.kind) {
          case "status":
            return Promise.resolve({ kind: "status", files: mockFiles });
          case "repoState":
            return Promise.resolve({ kind: "repoState", state: mockRepoState });
          case "diff":
            if (command.path === "src/main.ts") {
              return Promise.resolve({ kind: "diff", diffs: [mockDiff] });
            }
            return Promise.resolve({ kind: "diff", diffs: [] });
          case "branches":
            return Promise.resolve({ kind: "branches", branches: mockBranches });
          case "log":
            return Promise.resolve({ kind: "log", commits: mockCommits });
          case "stashList":
            return Promise.resolve({ kind: "stashList", entries: mockStashes });
          case "blame":
            return Promise.resolve({ kind: "blame", entries: [] });
          case "showCommit":
            return Promise.resolve({ kind: "showCommit", diffs: [mockDiff] });
          case "stage":
          case "unstage":
          case "stageLines":
          case "commit":
          case "push":
          case "pull":
          case "fetch":
          case "checkoutBranch":
          case "createBranch":
          case "deleteBranch":
          case "mergeBranch":
          case "stashPush":
          case "stashPop":
          case "stashDrop":
          case "discardChanges":
            return Promise.resolve({ kind: command.kind });
          default:
            return Promise.resolve({ kind: command.kind });
        }
      }

      window.__TAURI_INTERNALS__ = {
        transformCallback: function (callback, once) {
          var id = ++eventIdCounter;
          if (once) {
            callbackRegistry[id] = function () {
              callback.apply(null, arguments);
              delete callbackRegistry[id];
            };
          } else {
            callbackRegistry[id] = callback;
          }
          return id;
        },

        invoke: function (cmd, args) {
          if (cmd === "plugin:event|listen") {
            var eventName = args.event;
            var handlerId = args.handler;
            var eid = ++eventIdCounter;
            if (!eventListeners[eventName]) eventListeners[eventName] = [];
            eventListeners[eventName].push({ handlerId: handlerId, eventId: eid });
            return Promise.resolve(eid);
          }

          if (cmd === "plugin:event|unlisten") {
            var evName = args.event;
            var evId = args.eventId;
            if (eventListeners[evName]) {
              eventListeners[evName] = eventListeners[evName].filter(function (e) { return e.eventId !== evId; });
            }
            return Promise.resolve();
          }

          if (cmd === "bootstrap_shell") {
            return Promise.resolve({
              workspace: currentView(),
              settings: JSON.parse(JSON.stringify(settings)),
              profileCatalog: { terminalProfiles: profiles.slice() },
              runtimeProjections: allRuntimes(),
            });
          }

          if (cmd === "dispatch_workspace_command") {
            return Promise.resolve(currentView());
          }

          if (cmd === "dispatch_settings_command") {
            return Promise.resolve(JSON.parse(JSON.stringify(settings)));
          }

          if (cmd === "dispatch_runtime_command") {
            return Promise.resolve(null);
          }

          if (cmd === "dispatch_git_command") {
            return handleGitCommand(args.command);
          }

          if (cmd === "dispatch_browser_surface_command") {
            return Promise.resolve(null);
          }

          return Promise.resolve(null);
        },
      };
    })();
  `;
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript({ content: buildGitTauriMockScript() });
  await page.goto("/");
});

test("opens git pane and displays file status list", async ({ page }) => {
  // Wait for the git pane to load
  await expect(page.getByTestId("git-pane")).toBeVisible({ timeout: 10_000 });

  // Should show the file list
  await expect(page.getByTestId("git-file-list")).toBeVisible({ timeout: 5_000 });

  // Should display at least one file from the mock data
  await expect(page.getByText("src/main.ts").first()).toBeVisible({ timeout: 5_000 });
});

test("displays branch name from repo state", async ({ page }) => {
  await expect(page.getByTestId("git-pane")).toBeVisible({ timeout: 10_000 });

  // The branch name should be visible
  await expect(page.getByText("feature/git-client").first()).toBeVisible({ timeout: 5_000 });
});

test("selecting a file shows diff view", async ({ page }) => {
  await expect(page.getByTestId("git-pane")).toBeVisible({ timeout: 10_000 });
  await expect(page.getByTestId("git-file-list")).toBeVisible({ timeout: 5_000 });

  // Click on src/main.ts to select it
  const fileButton = page.getByTestId("file-select-button").first();
  await fileButton.click();

  // Diff viewer should appear with diff content
  await expect(page.getByTestId("git-diff-area")).toBeVisible({ timeout: 5_000 });

  // Hunk header should be visible
  await expect(page.getByTestId("hunk-header").first()).toBeVisible({ timeout: 5_000 });
});

test("view tabs are accessible (Changes, History, Branches, Stash)", async ({ page }) => {
  await expect(page.getByTestId("git-pane")).toBeVisible({ timeout: 10_000 });

  // All view tabs should be visible
  await expect(page.getByText("Changes").first()).toBeVisible({ timeout: 5_000 });
  await expect(page.getByText("History")).toBeVisible({ timeout: 5_000 });
  await expect(page.getByText("Branches")).toBeVisible({ timeout: 5_000 });
  await expect(page.getByText("Stash")).toBeVisible({ timeout: 5_000 });
});
