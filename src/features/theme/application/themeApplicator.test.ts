import { beforeEach, describe, expect, it } from "vitest";
import { applyPartialTokens, applyTheme, TOKEN_TO_CSS_VAR } from "./themeApplicator";
import { BUILT_IN_THEMES } from "../domain/presets";
import type { ThemeColorTokens } from "../domain/models";

describe("applyTheme", () => {
  beforeEach(() => {
    // Clear all inline styles on document root
    document.documentElement.removeAttribute("style");
  });

  it("sets all CSS variables on document root", () => {
    const midnight = BUILT_IN_THEMES[0];
    applyTheme(midnight);

    const root = document.documentElement;
    for (const [tokenKey, cssVar] of Object.entries(TOKEN_TO_CSS_VAR)) {
      const expected = midnight.colors[tokenKey as keyof ThemeColorTokens];
      expect(
        root.style.getPropertyValue(cssVar),
        `CSS var ${cssVar} should be "${expected}"`,
      ).toBe(expected);
    }
  });

  it("sets colorScheme to 'dark' for dark themes", () => {
    const midnight = BUILT_IN_THEMES[0];
    expect(midnight.kind).toBe("dark");
    applyTheme(midnight);
    expect(document.documentElement.style.colorScheme).toBe("dark");
  });

  it("sets colorScheme to 'light' for light themes", () => {
    const dawn = BUILT_IN_THEMES[1];
    expect(dawn.kind).toBe("light");
    applyTheme(dawn);
    expect(document.documentElement.style.colorScheme).toBe("light");
  });
});

describe("applyPartialTokens", () => {
  beforeEach(() => {
    document.documentElement.removeAttribute("style");
  });

  it("only sets specified tokens", () => {
    applyPartialTokens({ bg: "#ff0000", text: "#00ff00" });

    const root = document.documentElement;
    expect(root.style.getPropertyValue("--color-bg")).toBe("#ff0000");
    expect(root.style.getPropertyValue("--color-text")).toBe("#00ff00");
    // Unset tokens should remain empty
    expect(root.style.getPropertyValue("--color-accent")).toBe("");
  });
});
