/**
 * Shared Tauri mock for E2E tests.
 *
 * Returns a JS string that, when injected via `page.addInitScript()`,
 * sets up `window.__TAURI_INTERNALS__` with an in-memory workspace,
 * settings, event system, and command dispatch.
 *
 * Supports terminal, browser, and git pane types.
 */

interface TauriMockOptions {
  initialPanes?: Array<{
    kind: "terminal" | "browser" | "git";
    profileId?: string;
    cwd?: string;
    url?: string;
    commandOverride?: string | null;
  }>;
  settings?: Partial<{
    defaultLayout: string;
    defaultTerminalProfileId: string;
    defaultWorkingDirectory: string;
    defaultCustomCommand: string;
    fontSize: number;
    theme: string;
    launchFullscreen: boolean;
    hasCompletedOnboarding: boolean;
    lastWorkingDirectory: string | null;
  }>;
  /** 0 = never fail (default), 1 = fail first call then succeed, -1 = always fail */
  bootstrapFailureCount?: number;
  git?: {
    files?: Array<{
      path: string;
      oldPath?: string | null;
      indexStatus: string;
      worktreeStatus: string;
    }>;
    repoState?: {
      repoPath: string;
      headBranch: string;
      isDetached: boolean;
      statusClean: boolean;
    };
    diff?: {
      filePath: string;
      oldPath?: string | null;
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
      fileModeChange?: string | null;
    };
    branches?: Array<{
      name: string;
      isCurrent: boolean;
      upstream: string | null;
      ahead: number;
      behind: number;
    }>;
    commits?: Array<{
      hash: string;
      shortHash: string;
      authorName: string;
      authorEmail: string;
      date: string;
      message: string;
      parentHashes: string[];
    }>;
    stashes?: Array<{
      index: number;
      message: string;
      date: string;
    }>;
  };
}

export function createTauriMockScript(options?: TauriMockOptions): string {
  const opts = JSON.stringify(options ?? {});
  return `
    (function () {
      var opts = ${opts};

      // ---- profiles ----
      var profiles = [
        { id: "terminal", label: "Terminal", description: "Login shell", startupCommandTemplate: null },
        { id: "claude", label: "Claude Code", description: "Claude Code", startupCommandTemplate: "claude" },
        { id: "codex", label: "Codex", description: "Codex", startupCommandTemplate: "codex" },
        { id: "custom", label: "Custom", description: "Custom command", startupCommandTemplate: null },
      ];

      // ---- settings ----
      var settings = Object.assign({
        defaultLayout: "1x1",
        defaultTerminalProfileId: "terminal",
        defaultWorkingDirectory: "~/projects",
        defaultCustomCommand: "",
        fontSize: 13,
        theme: "midnight",
        launchFullscreen: true,
        hasCompletedOnboarding: true,
        lastWorkingDirectory: null,
      }, opts.settings || {});

      // ---- counters & registries ----
      var tabCounter = 0;
      var paneCounter = 0;
      var sessionCounter = 0;
      var eventIdCounter = 0;
      var callbackRegistry = {};
      var eventListeners = {};
      var bootstrapAttempts = 0;
      var bootstrapFailureCount = opts.bootstrapFailureCount || 0;

      function nextTabId() { return "tab-" + (++tabCounter); }
      function nextPaneId() { return "pane-" + (++paneCounter); }
      function nextSessionId() { return "session-" + (++sessionCounter); }

      // ---- pane factory ----
      function makePaneView(paneSpec) {
        var paneId = nextPaneId();
        var spec;
        if (paneSpec.kind === "browser") {
          spec = { kind: "browser", initial_url: paneSpec.url || paneSpec.initial_url || "https://google.com" };
        } else if (paneSpec.kind === "git") {
          spec = { kind: "git", working_directory: paneSpec.cwd || paneSpec.working_directory || "~/projects" };
        } else {
          spec = {
            kind: "terminal",
            launch_profile_id: paneSpec.profileId || paneSpec.launch_profile_id || "terminal",
            working_directory: paneSpec.cwd || paneSpec.working_directory || settings.defaultWorkingDirectory,
            command_override: paneSpec.commandOverride || paneSpec.command_override || null,
          };
        }
        return { paneId: paneId, title: "Pane " + paneCounter, spec: spec };
      }

      function makePaneRuntime(paneId, kind) {
        return {
          paneId: paneId,
          runtimeSessionId: nextSessionId(),
          kind: kind || "terminal",
          status: "running",
          lastError: null,
          browserLocation: null,
        };
      }

      // ---- layout helpers ----
      function leafNode(paneId) {
        return { type: "pane", paneId: paneId };
      }

      function splitNode(direction, first, second) {
        return { type: "split", direction: direction, ratio: 500, first: first, second: second };
      }

      function makeTab(paneSpecs) {
        var tabId = nextTabId();
        var panes = paneSpecs.map(function (spec) { return makePaneView(spec); });
        var layout = panes.length === 1
          ? leafNode(panes[0].paneId)
          : panes.reduce(function (acc, p, i) {
              return i === 0 ? leafNode(p.paneId) : splitNode("horizontal", acc, leafNode(p.paneId));
            }, null);
        return { tabId: tabId, title: "Workspace " + tabCounter, layout: layout, panes: panes, activePaneId: panes[0].paneId };
      }

      function rebuildLayout(node, removedPaneId) {
        if (node.type === "pane") {
          return node.paneId === removedPaneId ? null : node;
        }
        var first = rebuildLayout(node.first, removedPaneId);
        var second = rebuildLayout(node.second, removedPaneId);
        if (!first) return second;
        if (!second) return first;
        return { type: "split", direction: node.direction, ratio: node.ratio, first: first, second: second };
      }

      function swapInLayout(node, idA, idB) {
        if (!node) return node;
        if (node.type === "pane") {
          if (node.paneId === idA) return { type: "pane", paneId: idB };
          if (node.paneId === idB) return { type: "pane", paneId: idA };
          return node;
        }
        return {
          type: "split",
          direction: node.direction,
          ratio: node.ratio,
          first: swapInLayout(node.first, idA, idB),
          second: swapInLayout(node.second, idA, idB),
        };
      }

      // ---- initial workspace ----
      var initialPaneSpecs = (opts.initialPanes && opts.initialPanes.length > 0)
        ? opts.initialPanes
        : [{ kind: "terminal" }];

      var workspace = { activeTabId: "", tabs: [] };
      var initialTab = makeTab(initialPaneSpecs);
      workspace = { activeTabId: initialTab.tabId, tabs: [initialTab] };

      var runtimes = {};
      function ensureRuntime(paneId, kind) {
        if (!runtimes[paneId]) {
          runtimes[paneId] = makePaneRuntime(paneId, kind);
        }
        return runtimes[paneId];
      }

      workspace.tabs.forEach(function (tab) {
        tab.panes.forEach(function (p) { ensureRuntime(p.paneId, p.spec.kind); });
      });

      function currentView() {
        return JSON.parse(JSON.stringify(workspace));
      }

      function allRuntimes() {
        return Object.values(runtimes);
      }

      // ---- event system ----
      function emitEvent(eventName, payload) {
        var listeners = eventListeners[eventName] || [];
        listeners.forEach(function (entry) {
          var cb = callbackRegistry[entry.handlerId];
          if (cb) {
            cb({ event: eventName, id: entry.eventId, payload: payload });
          }
        });
      }

      function emitTerminalOutput(paneId) {
        var rt = runtimes[paneId];
        if (!rt || rt.kind !== "terminal") return;
        setTimeout(function () {
          emitEvent("terminal_output_received", {
            paneId: paneId,
            runtimeSessionId: rt.runtimeSessionId,
            chunk: "\\r\\nTerminal ready at " + paneId + "\\r\\n$ ",
          });
        }, 50);
      }

      function emitInitialOutput() {
        workspace.tabs.forEach(function (tab) {
          tab.panes.forEach(function (p) { emitTerminalOutput(p.paneId); });
        });
      }

      // ---- git state (stateful for stage/unstage) ----
      var gitData = opts.git || {};
      var gitFiles = gitData.files ? JSON.parse(JSON.stringify(gitData.files)) : [];
      var gitRepoState = gitData.repoState || null;
      var gitDiff = gitData.diff || null;
      var gitBranches = gitData.branches || [];
      var gitCommits = gitData.commits || [];
      var gitStashes = gitData.stashes || [];

      function handleGitCommand(command) {
        switch (command.kind) {
          case "status":
            return Promise.resolve({ kind: "status", files: JSON.parse(JSON.stringify(gitFiles)) });
          case "repoState":
            return Promise.resolve({ kind: "repoState", state: gitRepoState });
          case "diff":
            if (gitDiff && command.path === gitDiff.filePath) {
              return Promise.resolve({ kind: "diff", diffs: [gitDiff] });
            }
            return Promise.resolve({ kind: "diff", diffs: [] });
          case "branches":
            return Promise.resolve({ kind: "branches", branches: gitBranches });
          case "log":
            return Promise.resolve({ kind: "log", commits: gitCommits });
          case "stashList":
            return Promise.resolve({ kind: "stashList", entries: gitStashes });
          case "blame":
            return Promise.resolve({ kind: "blame", entries: [] });
          case "showCommit":
            return Promise.resolve({ kind: "showCommit", diffs: gitDiff ? [gitDiff] : [] });
          case "stage":
            if (command.paths) {
              command.paths.forEach(function (p) {
                gitFiles.forEach(function (f) {
                  if (f.path === p) { f.indexStatus = "modified"; f.worktreeStatus = "unmodified"; }
                });
              });
            }
            return Promise.resolve({ kind: "stage" });
          case "unstage":
            if (command.paths) {
              command.paths.forEach(function (p) {
                gitFiles.forEach(function (f) {
                  if (f.path === p) { f.indexStatus = "untracked"; f.worktreeStatus = "modified"; }
                });
              });
            }
            return Promise.resolve({ kind: "unstage" });
          default:
            return Promise.resolve({ kind: command.kind });
        }
      }

      // ---- workspace command handler ----
      function handleWorkspaceCommand(command) {
        switch (command.kind) {
          case "openTab": {
            var specs = command.pane_specs || [{ kind: "terminal" }];
            var newTab = makeTab(specs);
            if (command.title) newTab.title = command.title;
            newTab.panes.forEach(function (p) { ensureRuntime(p.paneId, p.spec.kind); });
            workspace = {
              activeTabId: newTab.tabId,
              tabs: workspace.tabs.concat([newTab]),
            };
            newTab.panes.forEach(function (p) { emitTerminalOutput(p.paneId); });
            emitEvent("workspace_projection_updated", { workspace: currentView() });
            return Promise.resolve(currentView());
          }

          case "closeTab": {
            workspace = {
              activeTabId: workspace.activeTabId === command.tab_id
                ? (workspace.tabs.filter(function (t) { return t.tabId !== command.tab_id; })[0] || { tabId: "" }).tabId
                : workspace.activeTabId,
              tabs: workspace.tabs.filter(function (t) { return t.tabId !== command.tab_id; }),
            };
            if (workspace.tabs.length === 0) {
              var fresh = makeTab([{ kind: "terminal" }]);
              fresh.panes.forEach(function (p) { ensureRuntime(p.paneId, "terminal"); });
              workspace = { activeTabId: fresh.tabId, tabs: [fresh] };
              fresh.panes.forEach(function (p) { emitTerminalOutput(p.paneId); });
            }
            emitEvent("workspace_projection_updated", { workspace: currentView() });
            return Promise.resolve(currentView());
          }

          case "setActiveTab": {
            workspace = { activeTabId: command.tab_id, tabs: workspace.tabs };
            return Promise.resolve(currentView());
          }

          case "focusPane": {
            workspace = {
              activeTabId: command.tab_id,
              tabs: workspace.tabs.map(function (t) {
                return t.tabId === command.tab_id
                  ? Object.assign({}, t, { activePaneId: command.pane_id })
                  : t;
              }),
            };
            return Promise.resolve(currentView());
          }

          case "splitPane": {
            var tabIdx = -1;
            for (var i = 0; i < workspace.tabs.length; i++) {
              if (workspace.tabs[i].panes.some(function (p) { return p.paneId === command.pane_id; })) {
                tabIdx = i;
                break;
              }
            }
            if (tabIdx === -1) return Promise.reject("Pane not found");

            var tab = workspace.tabs[tabIdx];
            var newPaneView = makePaneView(command.pane_spec);
            ensureRuntime(newPaneView.paneId, newPaneView.spec.kind);

            var newLayout = splitNode(
              command.direction,
              tab.layout,
              leafNode(newPaneView.paneId),
            );

            var updatedTab = Object.assign({}, tab, {
              layout: newLayout,
              panes: tab.panes.concat([newPaneView]),
            });

            workspace = {
              activeTabId: workspace.activeTabId,
              tabs: workspace.tabs.map(function (t, idx) { return idx === tabIdx ? updatedTab : t; }),
            };

            emitTerminalOutput(newPaneView.paneId);
            emitEvent("workspace_projection_updated", { workspace: currentView() });
            return Promise.resolve(currentView());
          }

          case "closePane": {
            var closedTabIdx = -1;
            for (var j = 0; j < workspace.tabs.length; j++) {
              if (workspace.tabs[j].panes.some(function (p) { return p.paneId === command.pane_id; })) {
                closedTabIdx = j;
                break;
              }
            }
            if (closedTabIdx === -1) return Promise.reject("Pane not found");

            var closedTab = workspace.tabs[closedTabIdx];
            var remainingPanes = closedTab.panes.filter(function (p) { return p.paneId !== command.pane_id; });

            if (remainingPanes.length === 0) {
              workspace = {
                activeTabId: workspace.activeTabId,
                tabs: workspace.tabs.filter(function (_, idx) { return idx !== closedTabIdx; }),
              };
              if (workspace.tabs.length === 0) {
                var autoTab = makeTab([{ kind: "terminal" }]);
                autoTab.panes.forEach(function (p) { ensureRuntime(p.paneId, "terminal"); });
                workspace = { activeTabId: autoTab.tabId, tabs: [autoTab] };
                autoTab.panes.forEach(function (p) { emitTerminalOutput(p.paneId); });
              } else {
                workspace = { activeTabId: workspace.tabs[0].tabId, tabs: workspace.tabs };
              }
            } else {
              var newActivePaneId = closedTab.activePaneId === command.pane_id
                ? remainingPanes[0].paneId
                : closedTab.activePaneId;
              var newClosedLayout = remainingPanes.length === 1
                ? leafNode(remainingPanes[0].paneId)
                : rebuildLayout(closedTab.layout, command.pane_id);

              var updatedClosedTab = Object.assign({}, closedTab, {
                panes: remainingPanes,
                activePaneId: newActivePaneId,
                layout: newClosedLayout,
              });
              workspace = {
                activeTabId: workspace.activeTabId,
                tabs: workspace.tabs.map(function (t, idx) { return idx === closedTabIdx ? updatedClosedTab : t; }),
              };
            }

            delete runtimes[command.pane_id];
            emitEvent("workspace_projection_updated", { workspace: currentView() });
            return Promise.resolve(currentView());
          }

          case "replacePaneSpec": {
            workspace = {
              activeTabId: workspace.activeTabId,
              tabs: workspace.tabs.map(function (t) {
                return Object.assign({}, t, {
                  panes: t.panes.map(function (p) {
                    return p.paneId === command.pane_id
                      ? Object.assign({}, p, { spec: command.pane_spec })
                      : p;
                  }),
                });
              }),
            };
            return Promise.resolve(currentView());
          }

          case "restartPaneRuntime": {
            if (runtimes[command.pane_id]) {
              runtimes[command.pane_id] = makePaneRuntime(command.pane_id, runtimes[command.pane_id].kind);
              emitTerminalOutput(command.pane_id);
              emitEvent("runtime_status_changed", { runtime: runtimes[command.pane_id] });
            }
            return Promise.resolve(currentView());
          }

          case "swapPaneSlots": {
            var swapTabIdx = -1;
            for (var s = 0; s < workspace.tabs.length; s++) {
              var paneIds = workspace.tabs[s].panes.map(function (p) { return p.paneId; });
              if (paneIds.indexOf(command.pane_id_a) !== -1 && paneIds.indexOf(command.pane_id_b) !== -1) {
                swapTabIdx = s;
                break;
              }
            }
            if (swapTabIdx !== -1) {
              var swapTab = workspace.tabs[swapTabIdx];
              var swappedLayout = swapInLayout(swapTab.layout, command.pane_id_a, command.pane_id_b);
              var updatedSwapTab = Object.assign({}, swapTab, { layout: swappedLayout });
              workspace = {
                activeTabId: workspace.activeTabId,
                tabs: workspace.tabs.map(function (t, idx) { return idx === swapTabIdx ? updatedSwapTab : t; }),
              };
            }
            emitEvent("workspace_projection_updated", { workspace: currentView() });
            return Promise.resolve(currentView());
          }

          case "renameTab": {
            workspace = {
              activeTabId: workspace.activeTabId,
              tabs: workspace.tabs.map(function (t) {
                return t.tabId === command.tab_id
                  ? Object.assign({}, t, { title: command.title })
                  : t;
              }),
            };
            emitEvent("workspace_projection_updated", { workspace: currentView() });
            return Promise.resolve(currentView());
          }

          default:
            return Promise.resolve(currentView());
        }
      }

      // ---- Tauri internals mock ----
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
          // ---- plugin:event handlers ----
          if (cmd === "plugin:event|listen") {
            var eventName = args.event;
            var handlerId = args.handler;
            var eid = ++eventIdCounter;
            if (!eventListeners[eventName]) eventListeners[eventName] = [];
            eventListeners[eventName].push({ handlerId: handlerId, eventId: eid });
            if (eventName === "terminal_output_received") {
              setTimeout(emitInitialOutput, 100);
            }
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

          // ---- bootstrap ----
          if (cmd === "bootstrap_shell") {
            bootstrapAttempts++;
            if (bootstrapFailureCount === -1) {
              return Promise.reject("Bootstrap failed (mock)");
            }
            if (bootstrapFailureCount > 0 && bootstrapAttempts <= bootstrapFailureCount) {
              return Promise.reject("Bootstrap failed (mock, attempt " + bootstrapAttempts + ")");
            }
            return Promise.resolve({
              workspace: currentView(),
              settings: JSON.parse(JSON.stringify(settings)),
              profileCatalog: { terminalProfiles: profiles.slice() },
              runtimeProjections: allRuntimes(),
            });
          }

          // ---- workspace commands ----
          if (cmd === "dispatch_workspace_command") {
            return handleWorkspaceCommand(args.command);
          }

          // ---- settings commands ----
          if (cmd === "dispatch_settings_command") {
            var sCmd = args.command;
            if (sCmd.kind === "update") {
              settings = sCmd.settings;
            }
            return Promise.resolve(JSON.parse(JSON.stringify(settings)));
          }

          // ---- runtime commands ----
          if (cmd === "dispatch_runtime_command") {
            var rCmd = args.command;
            if (rCmd.kind === "writeTerminalInput") {
              emitEvent("terminal_output_received", {
                paneId: rCmd.pane_id,
                runtimeSessionId: (runtimes[rCmd.pane_id] || {}).runtimeSessionId || "",
                chunk: rCmd.input,
              });
            }
            return Promise.resolve(null);
          }

          // ---- git commands ----
          if (cmd === "dispatch_git_command") {
            return handleGitCommand(args.command);
          }

          // ---- browser surface commands ----
          if (cmd === "dispatch_browser_surface_command") {
            return Promise.resolve(null);
          }

          return Promise.resolve(null);
        },
      };
    })();
  `;
}
