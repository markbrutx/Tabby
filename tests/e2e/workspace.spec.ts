import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    const profiles = [
      {
        id: "terminal",
        label: "Terminal",
        description: "Pure login shell",
        startupCommand: null,
      },
      {
        id: "claude",
        label: "Claude Code",
        description: "Open Claude Code in a fresh shell",
        startupCommand: "claude",
      },
      {
        id: "codex",
        label: "Codex",
        description: "Open Codex in a fresh shell",
        startupCommand: "codex",
      },
      {
        id: "custom",
        label: "Custom",
        description: "Run an arbitrary shell command",
        startupCommand: null,
      },
    ];

    const paneCountMap = {
      "1x1": 1,
      "1x2": 2,
      "2x2": 4,
      "2x3": 6,
      "3x3": 9,
    };

    let settings = {
      defaultLayout: "2x2",
      defaultProfileId: "terminal",
      defaultWorkingDirectory: "/Users/mark/workspaces/tabby",
      defaultCustomCommand: "",
      fontSize: 13,
      theme: "system",
      launchFullscreen: true,
      hasCompletedOnboarding: true,
    };

    let tabCounter = 1;
    let paneCounter = 1;
    let sessionCounter = 1;
    const listeners = new Set();

    function resolveProfile(profileId, startupCommand) {
      const profile = profiles.find((candidate) => candidate.id === profileId);
      if (!profile) {
        throw new Error(`Unknown profile ${profileId}`);
      }

      return {
        id: profile.id,
        label: profile.label,
        startupCommand:
          profileId === "custom"
            ? startupCommand || "npm run dev"
            : profile.startupCommand,
      };
    }

    function createPane(profileId, cwd, startupCommand, index) {
      const profile = resolveProfile(profileId, startupCommand);

      return {
        id: `pane-${paneCounter++}`,
        sessionId: `session-${sessionCounter++}`,
        title: `Pane ${index + 1}`,
        cwd,
        profileId: profile.id,
        profileLabel: profile.label,
        startupCommand: profile.startupCommand,
        status: "running",
      };
    }

    function createTab(preset, profileId, cwd, startupCommand) {
      const paneCount = paneCountMap[preset];
      const panes = Array.from({ length: paneCount }, (_, index) =>
        createPane(profileId, cwd, startupCommand, index),
      );

      return {
        id: `tab-${tabCounter}`,
        title: `Workspace ${tabCounter++}`,
        preset,
        panes,
        activePaneId: panes[0].id,
      };
    }

    let workspace = {
      activeTabId: "",
      tabs: [],
    };

    function emitBootstrapChunks(tab) {
      window.setTimeout(() => {
        tab.panes.forEach((pane) => {
          listeners.forEach((listener) =>
            listener({
              paneId: pane.id,
              sessionId: pane.sessionId,
              chunk: `\r\n${pane.profileLabel} ready at ${pane.cwd}\r\n$ `,
            }),
          );
        });
      }, 0);
    }

    function ensureInitialWorkspace() {
      if (workspace.tabs.length > 0) {
        return;
      }

      const firstTab = createTab(
        settings.defaultLayout,
        settings.defaultProfileId,
        settings.defaultWorkingDirectory,
        settings.defaultCustomCommand,
      );

      workspace = {
        activeTabId: firstTab.id,
        tabs: [firstTab],
      };
    }

    window.__TABBY_MOCK__ = {
      async bootstrapWorkspace() {
        ensureInitialWorkspace();
        emitBootstrapChunks(workspace.tabs[0]);
        return {
          workspace,
          settings,
          profiles,
        };
      },
      async createTab(request) {
        const nextTab = createTab(
          request.preset,
          request.profileId || settings.defaultProfileId,
          request.cwd || settings.defaultWorkingDirectory,
          request.startupCommand || settings.defaultCustomCommand,
        );
        workspace = {
          activeTabId: nextTab.id,
          tabs: [...workspace.tabs, nextTab],
        };
        emitBootstrapChunks(nextTab);
        return workspace;
      },
      async closeTab(tabId) {
        const nextTabs = workspace.tabs.filter((tab) => tab.id !== tabId);
        workspace = {
          activeTabId: nextTabs[0]?.id ?? "",
          tabs: nextTabs,
        };
        return workspace;
      },
      async setActiveTab(tabId) {
        workspace = {
          ...workspace,
          activeTabId: tabId,
        };
        return workspace;
      },
      async focusPane(tabId, paneId) {
        workspace = {
          ...workspace,
          activeTabId: tabId,
          tabs: workspace.tabs.map((tab) =>
            tab.id === tabId ? { ...tab, activePaneId: paneId } : tab,
          ),
        };
        return workspace;
      },
      async updatePaneProfile(request) {
        workspace = {
          ...workspace,
          tabs: workspace.tabs.map((tab) => ({
            ...tab,
            panes: tab.panes.map((pane) => {
              if (pane.id !== request.paneId) {
                return pane;
              }

              const profile = resolveProfile(
                request.profileId,
                request.startupCommand,
              );

              return {
                ...pane,
                profileId: profile.id,
                profileLabel: profile.label,
                startupCommand: profile.startupCommand,
                sessionId: `session-${sessionCounter++}`,
                status: "running",
              };
            }),
          })),
        };

        return workspace;
      },
      async updatePaneCwd(request) {
        workspace = {
          ...workspace,
          tabs: workspace.tabs.map((tab) => ({
            ...tab,
            panes: tab.panes.map((pane) =>
              pane.id === request.paneId
                ? {
                    ...pane,
                    cwd: request.cwd,
                    sessionId: `session-${sessionCounter++}`,
                    status: "running",
                  }
                : pane,
            ),
          })),
        };

        return workspace;
      },
      async restartPane(paneId) {
        workspace = {
          ...workspace,
          tabs: workspace.tabs.map((tab) => ({
            ...tab,
            panes: tab.panes.map((pane) =>
              pane.id === paneId
                ? {
                    ...pane,
                    sessionId: `session-${sessionCounter++}`,
                    status: "running",
                  }
                : pane,
            ),
          })),
        };

        return workspace;
      },
      async writePty(paneId, data) {
        const pane = workspace.tabs
          .flatMap((tab) => tab.panes)
          .find((candidate) => candidate.id === paneId);

        if (!pane) {
          return;
        }

        listeners.forEach((listener) =>
          listener({
            paneId,
            sessionId: pane.sessionId,
            chunk: data,
          }),
        );
      },
      async resizePty() {},
      async getAppSettings() {
        return settings;
      },
      async updateAppSettings(nextSettings) {
        settings = nextSettings;
        return settings;
      },
      async listenToPtyOutput(handler) {
        listeners.add(handler);
        return () => listeners.delete(handler);
      },
    };
  });

  await page.goto("/");
});

test("creates a new pair workspace from the sidebar launchpad", async ({ page }) => {
  await expect(page.getByTestId("active-workspace-title")).toHaveText("Workspace 1");
  await page.getByTestId("toggle-sidebar").click();
  await page.getByTestId("launchpad-1x2").click();

  await expect(page.getByTestId("active-workspace-title")).toHaveText("Workspace 2");
  await expect(page.locator('[data-testid^="pane-"]:visible')).toHaveCount(2);
});

test("switches tabs and closes the active tab", async ({ page }) => {
  await page.getByTestId("toggle-sidebar").click();
  await page.getByTestId("launchpad-1x2").click();
  await page.getByTestId("tab-1").click();

  await expect(page.getByTestId("active-workspace-title")).toHaveText("Workspace 1");
  await expect(page.locator('[data-testid^="pane-"]:visible')).toHaveCount(4);

  await page.getByTestId("tab-2").click();
  await page.getByTestId("close-tab-2").click();

  await expect(page.getByTestId("active-workspace-title")).toHaveText("Workspace 1");
  await expect(page.getByTestId("tab-2")).toHaveCount(0);
});

test("updates workspace defaults through the settings drawer", async ({ page }) => {
  await page.getByTestId("toggle-sidebar").click();
  await page.getByTestId("open-settings").click();
  await expect(page.getByTestId("settings-drawer")).toBeVisible();

  await page.getByTestId("settings-layout").selectOption("3x3");
  await page.getByTestId("settings-profile").selectOption("codex");
  await page
    .getByTestId("settings-working-directory")
    .fill("/Users/mark/workspaces/mega-grid");
  await page.getByTestId("save-settings").click();

  await page.getByRole("button", { name: "New workspace" }).click();

  await expect(page.getByTestId("active-workspace-title")).toHaveText("Workspace 2");
  await expect(page.locator('[data-testid^="pane-"]:visible')).toHaveCount(9);
  await expect(page.getByText("codex default profile")).toBeVisible();
});

test("reconfigures an active pane to a custom command", async ({ page }) => {
  const firstPane = page.locator('[data-testid^="pane-"]:visible').first();
  await firstPane.click();

  const paneTestId = await firstPane.getAttribute("data-testid");
  if (!paneTestId) {
    throw new Error("visible pane should expose a data-testid");
  }

  const paneId = paneTestId.replace("pane-", "");
  await page.getByTestId(`profile-select-${paneId}`).selectOption("custom");
  await page.getByTestId(`command-input-${paneId}`).fill("npm run lint");
  await page.getByRole("button", { name: "Launch" }).click();

  await expect(page.getByTestId(`profile-badge-${paneId}`)).toHaveText("Custom");
});

test("moves pane focus with keyboard shortcuts", async ({ page }) => {
  const panes = page.locator('[data-testid^="pane-"]:visible');
  const firstPaneId = (await panes.nth(0).getAttribute("data-testid"))?.replace("pane-", "");
  const secondPaneId = (await panes.nth(1).getAttribute("data-testid"))?.replace("pane-", "");

  if (!firstPaneId || !secondPaneId) {
    throw new Error("expected at least two visible panes");
  }

  await panes.nth(0).click();
  await expect(page.getByTestId(`pane-${firstPaneId}`)).toHaveAttribute("data-active", "true");

  await page.keyboard.press("Meta+Alt+ArrowRight");

  await expect(page.getByTestId(`pane-${secondPaneId}`)).toHaveAttribute("data-active", "true");
  await expect(page.getByTestId(`pane-${firstPaneId}`)).toHaveAttribute("data-active", "false");
});

test("creates new workspace with Cmd+T shortcut", async ({ page }) => {
  await expect(page.getByTestId("tab-1")).toBeVisible();

  await page.keyboard.press("Meta+t");

  await expect(page.getByTestId("tab-2")).toBeVisible();
  await expect(page.getByTestId("active-workspace-title")).toHaveText("Workspace 2");
});

test("closes workspace with Cmd+W shortcut", async ({ page }) => {
  await page.keyboard.press("Meta+t");
  await expect(page.getByTestId("tab-2")).toBeVisible();

  await page.keyboard.press("Meta+w");

  await expect(page.getByTestId("tab-2")).toHaveCount(0);
  await expect(page.getByTestId("active-workspace-title")).toHaveText("Workspace 1");
});

test("launches 3x3 war room from sidebar", async ({ page }) => {
  await page.getByTestId("toggle-sidebar").click();
  await page.getByTestId("launchpad-3x3").click();

  await expect(page.getByTestId("active-workspace-title")).toHaveText("Workspace 2");
  await expect(page.locator('[data-testid^="pane-"]:visible')).toHaveCount(9);
});

test("restarts a pane with Cmd+Shift+R", async ({ page }) => {
  const firstPane = page.locator('[data-testid^="pane-"]:visible').first();
  await firstPane.click();

  const paneTestId = await firstPane.getAttribute("data-testid");
  if (!paneTestId) {
    throw new Error("visible pane should expose a data-testid");
  }

  const paneId = paneTestId.replace("pane-", "");
  const badgeBefore = await page.getByTestId(`profile-badge-${paneId}`).textContent();

  await page.keyboard.press("Meta+Shift+r");

  await expect(page.getByTestId(`profile-badge-${paneId}`)).toHaveText(badgeBefore ?? "Terminal");
});

test("shows onboarding on first launch and proceeds to main UI after completion", async ({
  page,
}) => {
  await page.addInitScript(() => {
    const mock = window.__TABBY_MOCK__;
    if (mock) {
      const originalGetSettings = mock.getAppSettings.bind(mock);
      const originalUpdateSettings = mock.updateAppSettings.bind(mock);
      const originalBootstrap = mock.bootstrapWorkspace.bind(mock);

      mock.getAppSettings = async () => {
        const s = await originalGetSettings();
        return { ...s, hasCompletedOnboarding: false };
      };

      mock.bootstrapWorkspace = async () => {
        const result = await originalBootstrap();
        return {
          ...result,
          settings: { ...result.settings, hasCompletedOnboarding: false },
        };
      };

      mock.updateAppSettings = async (nextSettings) => {
        const result = await originalUpdateSettings(nextSettings);
        return result;
      };
    }
  });

  await page.reload();

  await expect(page.getByTestId("onboarding-wizard")).toBeVisible();
  await expect(page.getByText("Pick your deck")).toBeVisible();

  await page.getByTestId("onboarding-layout-1x2").click();
  await page.getByTestId("onboarding-next").click();

  await expect(page.getByText("Choose your shell")).toBeVisible();
  await page.getByTestId("onboarding-next").click();

  await expect(page.getByText("Make it yours")).toBeVisible();
  await page.getByTestId("onboarding-finish").click();

  await expect(page.getByTestId("active-workspace-title")).toBeVisible();
});
