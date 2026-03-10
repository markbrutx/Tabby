import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useThemeStore } from "./themeStore";
import { BUILT_IN_THEMES } from "../domain/presets";
import type { ThemeDefinition, ThemeExportFormat } from "../domain/models";

function makeMockMatchMedia(prefersDark: boolean) {
  return vi.fn().mockImplementation((query: string) => ({
    matches: query === "(prefers-color-scheme: dark)" ? prefersDark : false,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
  }));
}

function makeCustomTheme(overrides?: Partial<ThemeDefinition>): ThemeDefinition {
  return {
    id: "custom-test-1",
    name: "Test Custom",
    kind: "dark",
    builtIn: false,
    colors: { ...BUILT_IN_THEMES[0].colors },
    ...overrides,
  };
}

describe("themeStore", () => {
  beforeEach(() => {
    localStorage.clear();
    Object.defineProperty(window, "matchMedia", {
      writable: true,
      value: makeMockMatchMedia(true),
    });
    // Reset the store to initial state
    useThemeStore.setState({
      themes: [],
      activeThemeId: "midnight",
      resolvedTheme: BUILT_IN_THEMES[0],
      draft: null,
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("initialize", () => {
    it("loads built-in themes", () => {
      useThemeStore.getState().initialize();
      const state = useThemeStore.getState();
      expect(state.themes.length).toBeGreaterThanOrEqual(BUILT_IN_THEMES.length);
      for (const builtIn of BUILT_IN_THEMES) {
        expect(state.themes.find((t) => t.id === builtIn.id)).toBeDefined();
      }
    });

    it("loads custom themes from localStorage", () => {
      const custom = makeCustomTheme();
      localStorage.setItem("tabby-custom-themes", JSON.stringify([custom]));

      useThemeStore.getState().initialize();
      const state = useThemeStore.getState();
      expect(state.themes.find((t) => t.id === "custom-test-1")).toBeDefined();
    });
  });

  describe("selectTheme", () => {
    it("sets activeThemeId and resolvedTheme", () => {
      useThemeStore.getState().initialize();
      useThemeStore.getState().selectTheme("dracula");
      const state = useThemeStore.getState();
      expect(state.activeThemeId).toBe("dracula");
      expect(state.resolvedTheme.id).toBe("dracula");
    });

    it("with 'system' resolves to midnight when prefers dark", () => {
      Object.defineProperty(window, "matchMedia", {
        writable: true,
        value: makeMockMatchMedia(true),
      });
      useThemeStore.getState().initialize();
      useThemeStore.getState().selectTheme("system");
      const state = useThemeStore.getState();
      expect(state.activeThemeId).toBe("system");
      expect(state.resolvedTheme.id).toBe("midnight");
    });

    it("with 'system' resolves to dawn when prefers light", () => {
      Object.defineProperty(window, "matchMedia", {
        writable: true,
        value: makeMockMatchMedia(false),
      });
      useThemeStore.getState().initialize();
      useThemeStore.getState().selectTheme("system");
      const state = useThemeStore.getState();
      expect(state.activeThemeId).toBe("system");
      expect(state.resolvedTheme.id).toBe("dawn");
    });

    it("falls back to midnight for unknown ID", () => {
      useThemeStore.getState().initialize();
      useThemeStore.getState().selectTheme("nonexistent");
      const state = useThemeStore.getState();
      expect(state.resolvedTheme.id).toBe("midnight");
    });
  });

  describe("createTheme", () => {
    it("adds to themes array and persists to localStorage", () => {
      useThemeStore.getState().initialize();
      const custom = makeCustomTheme();
      useThemeStore.getState().createTheme(custom);

      const state = useThemeStore.getState();
      expect(state.themes.find((t) => t.id === custom.id)).toBeDefined();

      const stored = JSON.parse(localStorage.getItem("tabby-custom-themes")!) as ThemeDefinition[];
      expect(stored.find((t) => t.id === custom.id)).toBeDefined();
    });
  });

  describe("cloneTheme", () => {
    it("creates new theme with unique ID and given name", () => {
      useThemeStore.getState().initialize();
      const cloned = useThemeStore.getState().cloneTheme("midnight", "My Midnight");

      expect(cloned.id).not.toBe("midnight");
      expect(cloned.id.startsWith("custom-")).toBe(true);
      expect(cloned.name).toBe("My Midnight");
      expect(cloned.builtIn).toBe(false);
      expect(cloned.colors.bg).toBe(BUILT_IN_THEMES[0].colors.bg);

      const state = useThemeStore.getState();
      expect(state.themes.find((t) => t.id === cloned.id)).toBeDefined();
    });
  });

  describe("updateTheme", () => {
    it("modifies existing custom theme", () => {
      useThemeStore.getState().initialize();
      const custom = makeCustomTheme();
      useThemeStore.getState().createTheme(custom);

      const updated: ThemeDefinition = {
        ...custom,
        name: "Updated Name",
        colors: { ...custom.colors, bg: "#222222" },
      };
      useThemeStore.getState().updateTheme(updated);

      const state = useThemeStore.getState();
      const found = state.themes.find((t) => t.id === custom.id);
      expect(found).toBeDefined();
      expect(found!.name).toBe("Updated Name");
      expect(found!.colors.bg).toBe("#222222");
    });
  });

  describe("deleteTheme", () => {
    it("removes custom theme", () => {
      useThemeStore.getState().initialize();
      const custom = makeCustomTheme();
      useThemeStore.getState().createTheme(custom);

      useThemeStore.getState().deleteTheme(custom.id);

      const state = useThemeStore.getState();
      expect(state.themes.find((t) => t.id === custom.id)).toBeUndefined();
    });

    it("does not remove built-in theme", () => {
      useThemeStore.getState().initialize();
      const countBefore = useThemeStore.getState().themes.length;
      useThemeStore.getState().deleteTheme("midnight");
      expect(useThemeStore.getState().themes.length).toBe(countBefore);
    });
  });

  describe("draft editing", () => {
    it("openEditor with null sets a new draft", () => {
      useThemeStore.getState().initialize();
      useThemeStore.getState().openEditor(null);
      const state = useThemeStore.getState();
      expect(state.draft).not.toBeNull();
      expect(state.draft!.id.startsWith("custom-")).toBe(true);
      expect(state.draft!.name).toBe("New Theme");
    });

    it("openEditor with theme ID loads that theme as draft", () => {
      useThemeStore.getState().initialize();
      useThemeStore.getState().openEditor("dracula");
      const state = useThemeStore.getState();
      expect(state.draft).not.toBeNull();
      expect(state.draft!.id).toBe("dracula");
    });

    it("updateDraft updates draft colors", () => {
      useThemeStore.getState().initialize();
      useThemeStore.getState().openEditor(null);
      useThemeStore.getState().updateDraft({ bg: "#ff0000" });
      expect(useThemeStore.getState().draft!.colors.bg).toBe("#ff0000");
    });

    it("updateDraftMeta updates draft name and kind", () => {
      useThemeStore.getState().initialize();
      useThemeStore.getState().openEditor(null);
      useThemeStore.getState().updateDraftMeta({ name: "Renamed", kind: "light" });
      const draft = useThemeStore.getState().draft!;
      expect(draft.name).toBe("Renamed");
      expect(draft.kind).toBe("light");
    });

    it("saveDraft commits draft to store as new theme", () => {
      useThemeStore.getState().initialize();
      useThemeStore.getState().openEditor(null);
      const draftId = useThemeStore.getState().draft!.id;
      useThemeStore.getState().saveDraft();

      expect(useThemeStore.getState().draft).toBeNull();
      expect(useThemeStore.getState().themes.find((t) => t.id === draftId)).toBeDefined();
    });

    it("discardDraft clears draft", () => {
      useThemeStore.getState().initialize();
      useThemeStore.getState().openEditor(null);
      expect(useThemeStore.getState().draft).not.toBeNull();
      useThemeStore.getState().discardDraft();
      expect(useThemeStore.getState().draft).toBeNull();
    });
  });

  describe("import/export", () => {
    it("importTheme parses valid JSON and creates theme", () => {
      useThemeStore.getState().initialize();
      const exportData: ThemeExportFormat = {
        formatVersion: 1,
        theme: {
          id: "imported",
          name: "Imported Theme",
          kind: "dark",
          colors: { ...BUILT_IN_THEMES[0].colors },
        },
      };
      const imported = useThemeStore.getState().importTheme(JSON.stringify(exportData));

      expect(imported.name).toBe("Imported Theme");
      expect(imported.builtIn).toBe(false);
      expect(imported.id.startsWith("custom-")).toBe(true);
      expect(useThemeStore.getState().themes.find((t) => t.id === imported.id)).toBeDefined();
    });

    it("importTheme rejects invalid JSON", () => {
      useThemeStore.getState().initialize();
      expect(() => useThemeStore.getState().importTheme("not json")).toThrow("Invalid JSON format");
    });

    it("importTheme rejects unsupported format version", () => {
      useThemeStore.getState().initialize();
      const bad = JSON.stringify({ formatVersion: 99, theme: {} });
      expect(() => useThemeStore.getState().importTheme(bad)).toThrow("Unsupported format version");
    });

    it("importTheme rejects missing theme data", () => {
      useThemeStore.getState().initialize();
      const bad = JSON.stringify({ formatVersion: 1 });
      expect(() => useThemeStore.getState().importTheme(bad)).toThrow("Missing theme data");
    });

    it("exportTheme produces valid ThemeExportFormat JSON", () => {
      useThemeStore.getState().initialize();
      const json = useThemeStore.getState().exportTheme("midnight");
      const parsed = JSON.parse(json) as ThemeExportFormat;

      expect(parsed.formatVersion).toBe(1);
      expect(parsed.theme.id).toBe("midnight");
      expect(parsed.theme.name).toBe("Midnight");
      expect(parsed.theme.colors.bg).toBe("#120b08");
    });

    it("exportTheme throws for unknown theme", () => {
      useThemeStore.getState().initialize();
      expect(() => useThemeStore.getState().exportTheme("nonexistent")).toThrow("Theme not found");
    });
  });
});
