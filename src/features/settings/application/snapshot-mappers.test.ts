import { describe, expect, it } from "vitest";
import type {
  ProfileCatalogView,
  ProfileView,
  SettingsView,
} from "@/contracts/tauri-bindings";
import {
  mapProfileFromDto,
  mapProfileCatalogFromDto,
  mapSettingsFromDto,
} from "./snapshot-mappers";

describe("mapProfileFromDto", () => {
  it("maps a ProfileView to ProfileReadModel", () => {
    const dto: ProfileView = {
      id: "zsh-default",
      label: "Zsh",
      description: "Default Zsh terminal",
      startupCommandTemplate: null,
    };

    const result = mapProfileFromDto(dto);

    expect(result).toEqual({
      id: "zsh-default",
      label: "Zsh",
      description: "Default Zsh terminal",
      startupCommandTemplate: null,
    });
  });

  it("preserves a non-null startupCommandTemplate", () => {
    const dto: ProfileView = {
      id: "claude",
      label: "Claude Code",
      description: "Claude AI agent",
      startupCommandTemplate: "claude --project .",
    };

    const result = mapProfileFromDto(dto);

    expect(result.startupCommandTemplate).toBe("claude --project .");
  });

  it("produces a read-only compatible result", () => {
    const dto: ProfileView = {
      id: "test",
      label: "Test",
      description: "Test profile",
      startupCommandTemplate: null,
    };

    const result = mapProfileFromDto(dto);

    expect(result.id).toBe("test");
    expect(typeof result.label).toBe("string");
    expect(typeof result.description).toBe("string");
  });
});

describe("mapProfileCatalogFromDto", () => {
  it("maps a ProfileCatalogView with multiple profiles", () => {
    const dto: ProfileCatalogView = {
      terminalProfiles: [
        {
          id: "zsh",
          label: "Zsh",
          description: "Default shell",
          startupCommandTemplate: null,
        },
        {
          id: "bash",
          label: "Bash",
          description: "Bash shell",
          startupCommandTemplate: "/bin/bash",
        },
      ],
    };

    const result = mapProfileCatalogFromDto(dto);

    expect(result.terminalProfiles).toHaveLength(2);
    expect(result.terminalProfiles[0].id).toBe("zsh");
    expect(result.terminalProfiles[1].startupCommandTemplate).toBe("/bin/bash");
  });

  it("maps an empty catalog", () => {
    const dto: ProfileCatalogView = { terminalProfiles: [] };

    const result = mapProfileCatalogFromDto(dto);

    expect(result.terminalProfiles).toHaveLength(0);
  });

  it("does not mutate the original DTO", () => {
    const dto: ProfileCatalogView = {
      terminalProfiles: [
        {
          id: "zsh",
          label: "Zsh",
          description: "Shell",
          startupCommandTemplate: null,
        },
      ],
    };

    const original = JSON.stringify(dto);
    mapProfileCatalogFromDto(dto);

    expect(JSON.stringify(dto)).toBe(original);
  });
});

describe("mapSettingsFromDto", () => {
  it("maps a full SettingsView to SettingsReadModel", () => {
    const dto: SettingsView = {
      defaultLayout: "1x2",
      defaultTerminalProfileId: "zsh",
      defaultWorkingDirectory: "~",
      defaultCustomCommand: "",
      fontSize: 14,
      theme: "midnight",
      launchFullscreen: false,
      hasCompletedOnboarding: true,
      lastWorkingDirectory: "/Users/dev",
    };

    const result = mapSettingsFromDto(dto);

    expect(result).toEqual({
      defaultLayout: "1x2",
      defaultTerminalProfileId: "zsh",
      defaultWorkingDirectory: "~",
      defaultCustomCommand: "",
      fontSize: 14,
      theme: "midnight",
      launchFullscreen: false,
      hasCompletedOnboarding: true,
      lastWorkingDirectory: "/Users/dev",
    });
  });

  it("handles null lastWorkingDirectory", () => {
    const dto: SettingsView = {
      defaultLayout: "1x1",
      defaultTerminalProfileId: "default",
      defaultWorkingDirectory: "~",
      defaultCustomCommand: "",
      fontSize: 12,
      theme: "system",
      launchFullscreen: false,
      hasCompletedOnboarding: false,
      lastWorkingDirectory: null,
    };

    const result = mapSettingsFromDto(dto);

    expect(result.lastWorkingDirectory).toBeNull();
    expect(result.hasCompletedOnboarding).toBe(false);
  });

  it("result contains only camelCase field names", () => {
    const dto: SettingsView = {
      defaultLayout: "2x2",
      defaultTerminalProfileId: "bash",
      defaultWorkingDirectory: "/tmp",
      defaultCustomCommand: "echo hi",
      fontSize: 16,
      theme: "dawn",
      launchFullscreen: true,
      hasCompletedOnboarding: true,
      lastWorkingDirectory: null,
    };

    const result = mapSettingsFromDto(dto);
    const keys = Object.keys(result);

    for (const key of keys) {
      expect(key).not.toContain("_");
    }
  });
});
