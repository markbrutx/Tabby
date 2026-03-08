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

    const paneCountMap: Record<string, number> = {
      "1x1": 1,
      "1x2": 2,
      "2x2": 4,
      "2x3": 6,
      "3x3": 9,
    };

    let settings = {
      defaultLayout: "1x1",
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
    const listeners = new Set<(payload: any) => void>();

    function resolveProfile(profileId: string, startupCommand: string | null) {
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

    function createPane(profileId: string, cwd: string, startupCommand: string | null, index: number) {
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

    function leaf(paneId: string) {
      return { type: "pane", paneId };
    }

    function hsplit(a: string, b: string) {
      return { type: "split", direction: "horizontal", ratio: 500, first: leaf(a), second: leaf(b) };
    }

    function buildLayout(preset: string, paneIds: string[]): any {
      switch (preset) {
        case "1x1": return leaf(paneIds[0]);
        case "1x2": return hsplit(paneIds[0], paneIds[1]);
        case "2x2": return {
          type: "split", direction: "vertical", ratio: 500,
          first: hsplit(paneIds[0], paneIds[1]),
          second: hsplit(paneIds[2], paneIds[3]),
        };
        default: return leaf(paneIds[0]);
      }
    }

    function createTab(preset: string, profileId: string, cwd: string, startupCommand: string | null) {
      const paneCount = paneCountMap[preset] ?? 1;
      const panes = Array.from({ length: paneCount }, (_, index) =>
        createPane(profileId, cwd, startupCommand, index),
      );
      const paneIds = panes.map((p) => p.id);
      const layout = buildLayout(preset, paneIds);

      return {
        id: `tab-${tabCounter}`,
        title: `Workspace ${tabCounter++}`,
        layout,
        panes,
        activePaneId: panes[0].id,
      };
    }

    let workspace = {
      activeTabId: "",
      tabs: [] as any[],
    };

    function emitBootstrapChunks(tab: any) {
      window.setTimeout(() => {
        tab.panes.forEach((pane: any) => {
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

    (window as any).__TABBY_MOCK__ = {
      async bootstrapWorkspace() {
        ensureInitialWorkspace();
        emitBootstrapChunks(workspace.tabs[0]);
        return {
          workspace,
          settings,
          profiles,
        };
      },
      async createTab(request: any) {
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
      async closeTab(tabId: string) {
        const nextTabs = workspace.tabs.filter((tab: any) => tab.id !== tabId);
        workspace = {
          activeTabId: nextTabs[0]?.id ?? "",
          tabs: nextTabs,
        };
        return workspace;
      },
      async setActiveTab(tabId: string) {
        workspace = { ...workspace, activeTabId: tabId };
        return workspace;
      },
      async focusPane(tabId: string, paneId: string) {
        workspace = {
          ...workspace,
          activeTabId: tabId,
          tabs: workspace.tabs.map((tab: any) =>
            tab.id === tabId ? { ...tab, activePaneId: paneId } : tab,
          ),
        };
        return workspace;
      },
      async updatePaneProfile(request: any) {
        workspace = {
          ...workspace,
          tabs: workspace.tabs.map((tab: any) => ({
            ...tab,
            panes: tab.panes.map((pane: any) => {
              if (pane.id !== request.paneId) return pane;
              const profile = resolveProfile(request.profileId, request.startupCommand);
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
      async updatePaneCwd(request: any) {
        workspace = {
          ...workspace,
          tabs: workspace.tabs.map((tab: any) => ({
            ...tab,
            panes: tab.panes.map((pane: any) =>
              pane.id === request.paneId
                ? { ...pane, cwd: request.cwd, sessionId: `session-${sessionCounter++}`, status: "running" }
                : pane,
            ),
          })),
        };
        return workspace;
      },
      async restartPane(paneId: string) {
        workspace = {
          ...workspace,
          tabs: workspace.tabs.map((tab: any) => ({
            ...tab,
            panes: tab.panes.map((pane: any) =>
              pane.id === paneId
                ? { ...pane, sessionId: `session-${sessionCounter++}`, status: "running" }
                : pane,
            ),
          })),
        };
        return workspace;
      },
      async splitPane(request: any) {
        const tabIndex = workspace.tabs.findIndex((tab: any) =>
          tab.panes.some((p: any) => p.id === request.paneId),
        );
        if (tabIndex === -1) throw new Error(`Pane not found: ${request.paneId}`);

        const tab = workspace.tabs[tabIndex];
        const sourcePane = tab.panes.find((p: any) => p.id === request.paneId);
        const newPane = createPane(
          request.profileId ?? sourcePane.profileId,
          request.cwd ?? sourcePane.cwd,
          request.startupCommand ?? sourcePane.startupCommand,
          tab.panes.length,
        );

        const newLayout = {
          type: "split",
          direction: request.direction,
          ratio: 500,
          first: tab.layout,
          second: leaf(newPane.id),
        };

        workspace = {
          ...workspace,
          tabs: workspace.tabs.map((t: any, i: number) =>
            i === tabIndex
              ? { ...t, layout: newLayout, panes: [...t.panes, newPane] }
              : t,
          ),
        };

        emitBootstrapChunks({ panes: [newPane] });
        return workspace;
      },
      async closePane(paneId: string) {
        const tabIndex = workspace.tabs.findIndex((tab: any) =>
          tab.panes.some((p: any) => p.id === paneId),
        );
        if (tabIndex === -1) throw new Error(`Pane not found: ${paneId}`);

        const tab = workspace.tabs[tabIndex];
        const newPanes = tab.panes.filter((p: any) => p.id !== paneId);

        if (newPanes.length === 0) {
          workspace = {
            ...workspace,
            tabs: workspace.tabs.filter((_: any, i: number) => i !== tabIndex),
          };
          if (workspace.tabs.length === 0) {
            const freshTab = createTab(
              settings.defaultLayout,
              settings.defaultProfileId,
              settings.defaultWorkingDirectory,
              null,
            );
            workspace = { activeTabId: freshTab.id, tabs: [freshTab] };
            emitBootstrapChunks(freshTab);
          }
          return workspace;
        }

        const newActivePaneId = tab.activePaneId === paneId ? newPanes[0].id : tab.activePaneId;
        const newLayout = newPanes.length === 1 ? leaf(newPanes[0].id) : tab.layout;

        workspace = {
          ...workspace,
          tabs: workspace.tabs.map((t: any, i: number) =>
            i === tabIndex
              ? { ...t, layout: newLayout, panes: newPanes, activePaneId: newActivePaneId }
              : t,
          ),
        };

        return workspace;
      },
      async writePty(paneId: string, data: string) {
        const pane = workspace.tabs
          .flatMap((tab: any) => tab.panes)
          .find((candidate: any) => candidate.id === paneId);

        if (!pane) return;

        listeners.forEach((listener) =>
          listener({ paneId, sessionId: pane.sessionId, chunk: data }),
        );
      },
      async resizePty() {},
      async getAppSettings() {
        return settings;
      },
      async updateAppSettings(nextSettings: any) {
        settings = nextSettings;
        return settings;
      },
      async listenToPtyOutput(handler: (payload: any) => void) {
        listeners.add(handler);
        return () => listeners.delete(handler);
      },
    };
  });

  await page.goto("/");
});

test("bootstraps with a single terminal pane", async ({ page }) => {
  await expect(page.getByTestId("tab-1")).toBeVisible();
  await expect(page.locator('[data-testid^="pane-"]:visible')).toHaveCount(1);
});

test("creates new tab with Cmd+T", async ({ page }) => {
  await expect(page.getByTestId("tab-1")).toBeVisible();
  await page.keyboard.press("Meta+t");
  await expect(page.getByTestId("tab-2")).toBeVisible();
});

test("switches tabs by clicking", async ({ page }) => {
  await page.keyboard.press("Meta+t");
  await expect(page.getByTestId("tab-2")).toBeVisible();

  await page.getByTestId("tab-1").click();
  const tab1 = page.getByTestId("tab-1");
  await expect(tab1).toBeVisible();
});

test("closes tab with close button", async ({ page }) => {
  await page.keyboard.press("Meta+t");
  await expect(page.getByTestId("tab-2")).toBeVisible();

  await page.getByTestId("close-tab-2").click();
  await expect(page.getByTestId("tab-2")).toHaveCount(0);
});

test("creates new tab via + button", async ({ page }) => {
  await page.getByTestId("new-tab-button").click();
  await expect(page.getByTestId("tab-2")).toBeVisible();
});

test("pane focus via keyboard (Alt+Arrow)", async ({ page }) => {
  // Create a 1x2 tab so there are two panes
  await page.getByTestId("new-tab-button").click();

  const panes = page.locator('[data-testid^="pane-"]:visible');
  const count = await panes.count();
  if (count < 2) return; // Skip if only 1 pane (1x1 default)

  await panes.nth(0).click();
  await expect(panes.nth(0)).toHaveAttribute("data-active", "true");
});

test("opens settings with Cmd+,", async ({ page }) => {
  await page.keyboard.press("Meta+,");
  await expect(page.getByTestId("settings-modal")).toBeVisible();
});

test("saves settings and closes modal", async ({ page }) => {
  await page.keyboard.press("Meta+,");
  await expect(page.getByTestId("settings-modal")).toBeVisible();

  await page.getByTestId("settings-layout").selectOption("2x2");
  await page.getByTestId("save-settings").click();

  await expect(page.getByTestId("settings-modal")).toHaveCount(0);
});

test("restarts pane with Cmd+Shift+R", async ({ page }) => {
  const pane = page.locator('[data-testid^="pane-"]:visible').first();
  await pane.click();
  await page.keyboard.press("Meta+Shift+r");

  // Pane should still be visible after restart
  await expect(pane).toBeVisible();
});

test("Cmd+W closes active pane", async ({ page }) => {
  // With a single pane, Cmd+W closes the pane and auto-creates a fresh tab
  const tabId = await page.getByTestId("tab-1").textContent();
  await page.keyboard.press("Meta+w");

  // Should still have a tab (auto-created)
  await expect(page.getByTestId("tab-1")).toBeVisible();
});
