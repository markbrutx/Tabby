import { describe, expect, it } from "vitest";
import { BUILT_IN_THEMES, findThemeById } from "./presets";
import { validateThemeTokens } from "./validation";

describe("BUILT_IN_THEMES", () => {
  it("all built-in themes have valid color tokens", () => {
    for (const theme of BUILT_IN_THEMES) {
      const invalid = validateThemeTokens(theme.colors);
      expect(invalid, `Theme "${theme.name}" has invalid tokens: ${invalid.join(", ")}`).toEqual([]);
    }
  });

  it("all built-in themes have unique IDs", () => {
    const ids = BUILT_IN_THEMES.map((t) => t.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("all built-in themes have builtIn: true", () => {
    for (const theme of BUILT_IN_THEMES) {
      expect(theme.builtIn, `Theme "${theme.name}" should be builtIn`).toBe(true);
    }
  });

  it("dawn is light, all others are dark", () => {
    for (const theme of BUILT_IN_THEMES) {
      if (theme.id === "dawn") {
        expect(theme.kind).toBe("light");
      } else {
        expect(theme.kind, `Theme "${theme.name}" should be dark`).toBe("dark");
      }
    }
  });

  it("midnight preset has correct key colors", () => {
    const midnight = findThemeById(BUILT_IN_THEMES, "midnight");
    expect(midnight).toBeDefined();
    expect(midnight!.colors.bg).toBe("#120b08");
    expect(midnight!.colors.text).toBe("#f8ece2");
    expect(midnight!.colors.accent).toBe("#f2a084");
  });

  it("dawn preset has correct key colors", () => {
    const dawn = findThemeById(BUILT_IN_THEMES, "dawn");
    expect(dawn).toBeDefined();
    expect(dawn!.colors.bg).toBe("#ead4c3");
    expect(dawn!.colors.text).toBe("#5f463b");
    expect(dawn!.colors.accent).toBe("#ee8f72");
  });
});

describe("findThemeById", () => {
  it("returns the correct theme for a known ID", () => {
    const result = findThemeById(BUILT_IN_THEMES, "dracula");
    expect(result).toBeDefined();
    expect(result!.name).toBe("Dracula");
  });

  it("returns undefined for an unknown ID", () => {
    const result = findThemeById(BUILT_IN_THEMES, "nonexistent-theme");
    expect(result).toBeUndefined();
  });
});
