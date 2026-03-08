import { describe, expect, it, vi } from "vitest";
import type { SettingsView } from "@/contracts/tauri-bindings";
import type { SettingsClient } from "@/app-shell/clients";
import { createSettingsStore } from "./store";

function makeSettingsView(overrides?: Partial<SettingsView>): SettingsView {
  return {
    defaultLayout: "1x1",
    defaultTerminalProfileId: "terminal",
    defaultWorkingDirectory: "~",
    defaultCustomCommand: "",
    fontSize: 14,
    theme: "midnight",
    launchFullscreen: false,
    hasCompletedOnboarding: true,
    lastWorkingDirectory: null,
    ...overrides,
  };
}

function makeMockSettingsClient(
  overrides?: Partial<SettingsClient>,
): SettingsClient {
  return {
    dispatch: vi.fn().mockResolvedValue(makeSettingsView()),
    listenProjectionUpdated: vi.fn().mockResolvedValue(() => {}),
    ...overrides,
  };
}

describe("createSettingsStore", () => {
  it("loads settings and profiles from bootstrap data", () => {
    const client = makeMockSettingsClient();
    const store = createSettingsStore(client);

    expect(store.getState().settings).toBeNull();
    expect(store.getState().profiles).toEqual([]);

    const settingsDto = makeSettingsView({ fontSize: 16 });
    const profiles = [
      {
        id: "zsh",
        label: "Zsh",
        description: "Z shell",
        startupCommandTemplate: null,
      },
    ];

    store.getState().loadBootstrap(settingsDto, profiles);

    expect(store.getState().settings).not.toBeNull();
    expect(store.getState().settings?.fontSize).toBe(16);
    expect(store.getState().profiles).toHaveLength(1);
    expect(store.getState().profiles[0].id).toBe("zsh");
  });

  it("dispatches update command through injected client", async () => {
    const updatedView = makeSettingsView({ fontSize: 18 });
    const client = makeMockSettingsClient({
      dispatch: vi.fn().mockResolvedValue(updatedView),
    });
    const store = createSettingsStore(client);

    store.getState().loadBootstrap(makeSettingsView(), []);

    const currentSettings = store.getState().settings;
    if (!currentSettings) throw new Error("settings should be loaded");

    await store.getState().updateSettings({ ...currentSettings, fontSize: 18 });

    expect(client.dispatch).toHaveBeenCalledWith(
      expect.objectContaining({ kind: "update" }),
    );
    expect(store.getState().settings?.fontSize).toBe(18);
  });

  it("dispatches reset command through injected client", async () => {
    const defaultView = makeSettingsView({ fontSize: 14 });
    const client = makeMockSettingsClient({
      dispatch: vi.fn().mockResolvedValue(defaultView),
    });
    const store = createSettingsStore(client);

    store.getState().loadBootstrap(makeSettingsView({ fontSize: 20 }), []);

    await store.getState().resetSettings();

    expect(client.dispatch).toHaveBeenCalledWith({ kind: "reset" });
    expect(store.getState().settings?.fontSize).toBe(14);
  });

  it("throws on dispatch failure with descriptive message", async () => {
    const client = makeMockSettingsClient({
      dispatch: vi.fn().mockRejectedValue(new Error("network timeout")),
    });
    const store = createSettingsStore(client);

    store.getState().loadBootstrap(makeSettingsView(), []);
    const settings = store.getState().settings;
    if (!settings) throw new Error("settings should be loaded");

    await expect(store.getState().updateSettings(settings)).rejects.toThrow(
      "network timeout",
    );
  });

  it("maps multiple profiles from bootstrap data", () => {
    const client = makeMockSettingsClient();
    const store = createSettingsStore(client);

    const profiles = [
      { id: "zsh", label: "Zsh", description: "Z shell", startupCommandTemplate: null },
      { id: "bash", label: "Bash", description: "Bash shell", startupCommandTemplate: "/bin/bash" },
      { id: "claude", label: "Claude", description: "AI agent", startupCommandTemplate: "claude --project ." },
    ];

    store.getState().loadBootstrap(makeSettingsView(), profiles);

    expect(store.getState().profiles).toHaveLength(3);
    expect(store.getState().profiles[0].id).toBe("zsh");
    expect(store.getState().profiles[1].startupCommandTemplate).toBe("/bin/bash");
    expect(store.getState().profiles[2].label).toBe("Claude");
  });

  it("registers event listener on bootstrap via initializeListeners", () => {
    const client = makeMockSettingsClient();
    const store = createSettingsStore(client);

    store.getState().loadBootstrap(makeSettingsView(), []);

    expect(client.listenProjectionUpdated).toHaveBeenCalledOnce();
  });

  it("throws on reset failure with descriptive message", async () => {
    const client = makeMockSettingsClient({
      dispatch: vi.fn().mockRejectedValue(new Error("reset failed")),
    });
    const store = createSettingsStore(client);

    store.getState().loadBootstrap(makeSettingsView(), []);

    await expect(store.getState().resetSettings()).rejects.toThrow(
      "reset failed",
    );
  });

  describe("isolation", () => {
    it("can be instantiated and tested with no cross-feature dependencies", () => {
      const client = makeMockSettingsClient();
      const store = createSettingsStore(client);

      expect(store.getState().settings).toBeNull();
      expect(store.getState().profiles).toEqual([]);

      store.getState().loadBootstrap(makeSettingsView(), []);

      expect(store.getState().settings).not.toBeNull();
    });
  });
});
