import { describe, expect, it, vi } from "vitest";
import type { SettingsView } from "@/contracts/tauri-bindings";
import type { SettingsClient } from "@/app-shell/clients";
import { createSettingsStore } from "./store";

function makeSettingsView(overrides?: Partial<SettingsView>): SettingsView {
  return {
    defaultLayout: "single",
    defaultTerminalProfileId: "terminal",
    defaultWorkingDirectory: "~",
    defaultCustomCommand: "",
    fontSize: 14,
    theme: "dark",
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
});
