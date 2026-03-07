import { describe, expect, it } from "vitest";
import { createMockTransport } from "./mockTransport";

async function setupOnboarded() {
  const transport = createMockTransport();
  const defaults = await transport.getAppSettings();
  await transport.updateAppSettings({
    ...defaults,
    hasCompletedOnboarding: true,
    defaultProfileId: "terminal",
    defaultWorkingDirectory: "~",
  });
  return transport;
}

describe("mockTransport", () => {
  it("bootstraps with empty workspace when onboarding not completed", async () => {
    const transport = createMockTransport();
    const result = await transport.bootstrapWorkspace();

    expect(result.workspace.tabs).toHaveLength(0);
    expect(result.settings.hasCompletedOnboarding).toBe(false);
    expect(result.settings.defaultProfileId).toBe("");
    expect(result.settings.defaultWorkingDirectory).toBe("");
    expect(result.profiles).toHaveLength(4);
  });

  it("bootstraps with one tab when onboarding completed", async () => {
    const transport = await setupOnboarded();
    const result = await transport.bootstrapWorkspace();

    expect(result.workspace.tabs).toHaveLength(1);
    expect(result.workspace.activeTabId).toBe(result.workspace.tabs[0].id);
    expect(result.settings.defaultLayout).toBe("1x1");
    expect(result.profiles).toHaveLength(4);
  });

  it("creates a tab with the correct pane count for preset", async () => {
    const transport = await setupOnboarded();
    await transport.bootstrapWorkspace();

    const snapshot = await transport.createTab({
      preset: "1x2",
      cwd: "/projects",
      profileId: "terminal",
      startupCommand: null,
    });

    expect(snapshot.tabs).toHaveLength(2);
    const newTab = snapshot.tabs[1];
    expect(newTab.panes).toHaveLength(2);
    expect(newTab.layout.type).toBe("split");
    expect(snapshot.activeTabId).toBe(newTab.id);
  });

  it("closes a tab and falls back to previous", async () => {
    const transport = await setupOnboarded();
    const bootstrap = await transport.bootstrapWorkspace();
    const firstTabId = bootstrap.workspace.tabs[0].id;

    const afterCreate = await transport.createTab({
      preset: "1x1",
      cwd: null,
      profileId: null,
      startupCommand: null,
    });
    const secondTabId = afterCreate.tabs[1].id;

    const afterClose = await transport.closeTab(secondTabId);

    expect(afterClose.tabs).toHaveLength(1);
    expect(afterClose.activeTabId).toBe(firstTabId);
  });

  it("creates a new tab when closing the last tab", async () => {
    const transport = await setupOnboarded();
    const bootstrap = await transport.bootstrapWorkspace();
    const tabId = bootstrap.workspace.tabs[0].id;

    const afterClose = await transport.closeTab(tabId);

    expect(afterClose.tabs).toHaveLength(1);
    expect(afterClose.tabs[0].id).not.toBe(tabId);
  });

  it("switches active tab", async () => {
    const transport = await setupOnboarded();
    const bootstrap = await transport.bootstrapWorkspace();
    const firstTabId = bootstrap.workspace.tabs[0].id;

    await transport.createTab({
      preset: "1x1",
      cwd: null,
      profileId: null,
      startupCommand: null,
    });

    const afterSwitch = await transport.setActiveTab(firstTabId);
    expect(afterSwitch.activeTabId).toBe(firstTabId);
  });

  it("updates pane profile", async () => {
    const transport = await setupOnboarded();
    const bootstrap = await transport.bootstrapWorkspace();
    const paneId = bootstrap.workspace.tabs[0].panes[0].id;

    const afterUpdate = await transport.updatePaneProfile({
      paneId,
      profileId: "claude",
      startupCommand: null,
    });

    const updatedPane = afterUpdate.tabs[0].panes[0];
    expect(updatedPane.profileId).toBe("claude");
    expect(updatedPane.profileLabel).toBe("Claude Code");
  });

  it("updates pane cwd", async () => {
    const transport = await setupOnboarded();
    const bootstrap = await transport.bootstrapWorkspace();
    const paneId = bootstrap.workspace.tabs[0].panes[0].id;

    const afterUpdate = await transport.updatePaneCwd({
      paneId,
      cwd: "/new/path",
    });

    const updatedPane = afterUpdate.tabs[0].panes[0];
    expect(updatedPane.cwd).toBe("/new/path");
  });

  it("restarts a pane with new session id", async () => {
    const transport = await setupOnboarded();
    const bootstrap = await transport.bootstrapWorkspace();
    const pane = bootstrap.workspace.tabs[0].panes[0];
    const oldSessionId = pane.sessionId;

    const afterRestart = await transport.restartPane(pane.id);
    const restartedPane = afterRestart.tabs[0].panes[0];

    expect(restartedPane.sessionId).not.toBe(oldSessionId);
    expect(restartedPane.status).toBe("running");
  });

  it("updates and retrieves settings", async () => {
    const transport = createMockTransport();
    const original = await transport.getAppSettings();

    const updated = await transport.updateAppSettings({
      ...original,
      fontSize: 16,
      defaultLayout: "3x3",
    });

    expect(updated.fontSize).toBe(16);
    expect(updated.defaultLayout).toBe("3x3");

    const retrieved = await transport.getAppSettings();
    expect(retrieved.fontSize).toBe(16);
  });

  it("emits pty output to listeners", async () => {
    const transport = await setupOnboarded();
    const chunks: string[] = [];

    await transport.listenToPtyOutput((payload) => {
      chunks.push(payload.chunk);
    });

    const bootstrap = await transport.bootstrapWorkspace();
    const pane = bootstrap.workspace.tabs[0].panes[0];

    await transport.writePty(pane.id, "hello");

    expect(chunks.some((c) => c.includes("hello"))).toBe(true);
  });

  it("unlisten stops receiving output", async () => {
    const transport = await setupOnboarded();
    const chunks: string[] = [];

    const unlisten = await transport.listenToPtyOutput((payload) => {
      chunks.push(payload.chunk);
    });

    unlisten();

    const bootstrap = await transport.bootstrapWorkspace();
    const pane = bootstrap.workspace.tabs[0].panes[0];
    await transport.writePty(pane.id, "after-unlisten");

    expect(chunks.every((c) => !c.includes("after-unlisten"))).toBe(true);
  });

  it("custom profile requires startup command", async () => {
    const transport = await setupOnboarded();
    const bootstrap = await transport.bootstrapWorkspace();
    const paneId = bootstrap.workspace.tabs[0].panes[0].id;

    const afterUpdate = await transport.updatePaneProfile({
      paneId,
      profileId: "custom",
      startupCommand: "npm run dev",
    });

    const updatedPane = afterUpdate.tabs[0].panes[0];
    expect(updatedPane.profileId).toBe("custom");
    expect(updatedPane.startupCommand).toBe("npm run dev");
  });

  it("splits a pane and adds new pane to tab", async () => {
    const transport = await setupOnboarded();
    const bootstrap = await transport.bootstrapWorkspace();
    const paneId = bootstrap.workspace.tabs[0].panes[0].id;

    const afterSplit = await transport.splitPane({
      paneId,
      direction: "horizontal",
      profileId: null,
      startupCommand: null,
      cwd: null,
    });

    expect(afterSplit.tabs[0].panes).toHaveLength(2);
    expect(afterSplit.tabs[0].layout.type).toBe("split");
  });

  it("creates a tab with per-pane configs", async () => {
    const transport = await setupOnboarded();
    await transport.bootstrapWorkspace();

    const snapshot = await transport.createTab({
      preset: "1x2",
      cwd: null,
      profileId: null,
      startupCommand: null,
      paneConfigs: [
        { profileId: "terminal", cwd: "/projects/a", startupCommand: null },
        { profileId: "claude", cwd: "/projects/b", startupCommand: null },
      ],
    });

    const tab = snapshot.tabs[1];
    expect(tab.panes).toHaveLength(2);
    expect(tab.panes[0].profileId).toBe("terminal");
    expect(tab.panes[0].cwd).toBe("/projects/a");
    expect(tab.panes[1].profileId).toBe("claude");
    expect(tab.panes[1].cwd).toBe("/projects/b");
  });

  it("creates a tab with uniform config when paneConfigs is absent", async () => {
    const transport = await setupOnboarded();
    await transport.bootstrapWorkspace();

    const snapshot = await transport.createTab({
      preset: "1x2",
      cwd: "/uniform",
      profileId: "terminal",
      startupCommand: null,
    });

    const tab = snapshot.tabs[1];
    expect(tab.panes).toHaveLength(2);
    expect(tab.panes[0].cwd).toBe("/uniform");
    expect(tab.panes[1].cwd).toBe("/uniform");
    expect(tab.panes[0].profileId).toBe("terminal");
    expect(tab.panes[1].profileId).toBe("terminal");
  });

  it("creates a tab with arbitrary paneConfigs count (e.g. 5)", async () => {
    const transport = await setupOnboarded();
    await transport.bootstrapWorkspace();

    const snapshot = await transport.createTab({
      preset: "1x1",
      cwd: null,
      profileId: null,
      startupCommand: null,
      paneConfigs: [
        { profileId: "terminal", cwd: "/a", startupCommand: null },
        { profileId: "terminal", cwd: "/a", startupCommand: null },
        { profileId: "claude", cwd: "/b", startupCommand: null },
        { profileId: "claude", cwd: "/b", startupCommand: null },
        { profileId: "claude", cwd: "/b", startupCommand: null },
      ],
    });

    const tab = snapshot.tabs[1];
    expect(tab.panes).toHaveLength(5);
    expect(tab.panes[0].cwd).toBe("/a");
    expect(tab.panes[4].cwd).toBe("/b");
    expect(tab.layout.type).toBe("split");
  });

  it("closes a pane and collapses layout", async () => {
    const transport = await setupOnboarded();
    const bootstrap = await transport.bootstrapWorkspace();
    const paneId = bootstrap.workspace.tabs[0].panes[0].id;

    await transport.splitPane({
      paneId,
      direction: "horizontal",
      profileId: null,
      startupCommand: null,
      cwd: null,
    });

    const firstPaneId = bootstrap.workspace.tabs[0].panes[0].id;
    const afterClose = await transport.closePane(firstPaneId);

    expect(afterClose.tabs[0].panes).toHaveLength(1);
    expect(afterClose.tabs[0].layout.type).toBe("pane");
  });
});
