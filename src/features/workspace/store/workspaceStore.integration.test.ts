import { describe, expect, it } from "vitest";
import { createWorkspaceStore } from "./workspaceStore";
import { createMockTransport } from "@/lib/bridge/mockTransport";

describe("workspaceStore + mockTransport integration", () => {
  function setup() {
    const transport = createMockTransport();
    const store = createWorkspaceStore(transport);
    return { store, transport };
  }

  it("initializes with a default 1x1 tab", async () => {
    const { store } = setup();
    await store.getState().initialize();

    const { workspace, settings, profiles, isHydrating } = store.getState();
    expect(isHydrating).toBe(false);
    expect(workspace).not.toBeNull();
    expect(workspace!.tabs).toHaveLength(1);
    expect(workspace!.tabs[0].panes).toHaveLength(1);
    expect(workspace!.tabs[0].layout.type).toBe("pane");
    expect(settings?.defaultLayout).toBe("1x1");
    expect(profiles).toHaveLength(4);
  });

  it("creates a new tab and switches to it", async () => {
    const { store } = setup();
    await store.getState().initialize();
    await store.getState().createTab("1x2");

    const { workspace } = store.getState();
    expect(workspace!.tabs).toHaveLength(2);
    expect(workspace!.tabs[1].panes).toHaveLength(2);
    expect(workspace!.activeTabId).toBe(workspace!.tabs[1].id);
  });

  it("closes a tab and falls back", async () => {
    const { store } = setup();
    await store.getState().initialize();
    await store.getState().createTab("1x1");

    const secondTabId = store.getState().workspace!.tabs[1].id;
    const firstTabId = store.getState().workspace!.tabs[0].id;

    await store.getState().closeTab(secondTabId);

    const { workspace } = store.getState();
    expect(workspace!.tabs).toHaveLength(1);
    expect(workspace!.activeTabId).toBe(firstTabId);
  });

  it("closing last tab auto-creates a fresh one", async () => {
    const { store } = setup();
    await store.getState().initialize();
    const tabId = store.getState().workspace!.tabs[0].id;

    await store.getState().closeTab(tabId);

    const { workspace } = store.getState();
    expect(workspace!.tabs).toHaveLength(1);
    expect(workspace!.tabs[0].id).not.toBe(tabId);
  });

  it("switches tabs with setActiveTab", async () => {
    const { store } = setup();
    await store.getState().initialize();
    const firstTabId = store.getState().workspace!.tabs[0].id;

    await store.getState().createTab("1x1");
    await store.getState().setActiveTab(firstTabId);

    expect(store.getState().workspace!.activeTabId).toBe(firstTabId);
  });

  it("focuses pane within active tab", async () => {
    const { store } = setup();
    await store.getState().initialize();
    await store.getState().createTab("1x2");

    const tab = store.getState().workspace!.tabs[1];
    const secondPane = tab.panes[1];

    await store.getState().focusPane(tab.id, secondPane.id);

    const updatedTab = store.getState().workspace!.tabs[1];
    expect(updatedTab.activePaneId).toBe(secondPane.id);
  });

  it("updates pane profile to claude", async () => {
    const { store } = setup();
    await store.getState().initialize();

    const paneId = store.getState().workspace!.tabs[0].panes[0].id;

    await store.getState().updatePaneProfile({
      paneId,
      profileId: "claude",
      startupCommand: null,
    });

    const pane = store.getState().workspace!.tabs[0].panes[0];
    expect(pane.profileId).toBe("claude");
    expect(pane.profileLabel).toBe("Claude Code");
  });

  it("updates pane working directory", async () => {
    const { store } = setup();
    await store.getState().initialize();

    const paneId = store.getState().workspace!.tabs[0].panes[0].id;

    await store.getState().updatePaneCwd({
      paneId,
      cwd: "/new/project",
    });

    const pane = store.getState().workspace!.tabs[0].panes[0];
    expect(pane.cwd).toBe("/new/project");
  });

  it("restarts pane with a new session", async () => {
    const { store } = setup();
    await store.getState().initialize();

    const pane = store.getState().workspace!.tabs[0].panes[0];
    const oldSessionId = pane.sessionId;

    await store.getState().restartPane(pane.id);

    const restarted = store.getState().workspace!.tabs[0].panes[0];
    expect(restarted.sessionId).not.toBe(oldSessionId);
    expect(restarted.status).toBe("running");
  });

  it("saves and applies settings", async () => {
    const { store } = setup();
    await store.getState().initialize();

    const current = store.getState().settings!;
    await store.getState().updateSettings({
      ...current,
      fontSize: 18,
      defaultLayout: "3x3",
    });

    const updated = store.getState().settings!;
    expect(updated.fontSize).toBe(18);
    expect(updated.defaultLayout).toBe("3x3");
  });

  it("creates custom profile tab with startup command", async () => {
    const { store } = setup();
    await store.getState().initialize();

    const paneId = store.getState().workspace!.tabs[0].panes[0].id;
    await store.getState().updatePaneProfile({
      paneId,
      profileId: "custom",
      startupCommand: "npm run dev",
    });

    const pane = store.getState().workspace!.tabs[0].panes[0];
    expect(pane.profileId).toBe("custom");
    expect(pane.startupCommand).toBe("npm run dev");
  });

  it("clears error state", async () => {
    const { store } = setup();
    await store.getState().initialize();

    store.getState().clearError();
    expect(store.getState().error).toBeNull();
  });

  it("handles multiple tab create/close cycles", async () => {
    const { store } = setup();
    await store.getState().initialize();

    await store.getState().createTab("1x1");
    await store.getState().createTab("1x2");
    await store.getState().createTab("2x3");

    expect(store.getState().workspace!.tabs).toHaveLength(4);

    const tab2Id = store.getState().workspace!.tabs[1].id;
    await store.getState().closeTab(tab2Id);

    expect(store.getState().workspace!.tabs).toHaveLength(3);
  });

  it("splits a pane horizontally", async () => {
    const { store } = setup();
    await store.getState().initialize();

    const paneId = store.getState().workspace!.tabs[0].panes[0].id;
    await store.getState().splitPane({
      paneId,
      direction: "horizontal",
      profileId: null,
      startupCommand: null,
      cwd: null,
    });

    const tab = store.getState().workspace!.tabs[0];
    expect(tab.panes).toHaveLength(2);
    expect(tab.layout.type).toBe("split");
  });

  it("closes a pane (not the last one)", async () => {
    const { store } = setup();
    await store.getState().initialize();

    const paneId = store.getState().workspace!.tabs[0].panes[0].id;
    await store.getState().splitPane({
      paneId,
      direction: "horizontal",
      profileId: null,
      startupCommand: null,
      cwd: null,
    });

    const firstPaneId = store.getState().workspace!.tabs[0].panes[0].id;
    await store.getState().closePane(firstPaneId);

    const tab = store.getState().workspace!.tabs[0];
    expect(tab.panes).toHaveLength(1);
  });
});
