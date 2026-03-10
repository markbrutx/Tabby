import { describe, expect, it } from "vitest";
import { generateThemeId, isValidCssColor, validateThemeTokens } from "./validation";
import type { ThemeColorTokens } from "./models";

describe("isValidCssColor", () => {
  it("accepts 3-digit hex", () => {
    expect(isValidCssColor("#fff")).toBe(true);
    expect(isValidCssColor("#abc")).toBe(true);
  });

  it("accepts 6-digit hex", () => {
    expect(isValidCssColor("#ffffff")).toBe(true);
    expect(isValidCssColor("#1a2b3c")).toBe(true);
  });

  it("accepts 8-digit hex (with alpha)", () => {
    expect(isValidCssColor("#ffffffff")).toBe(true);
    expect(isValidCssColor("#1a2b3c80")).toBe(true);
  });

  it("accepts rgb format", () => {
    expect(isValidCssColor("rgb(255, 0, 128)")).toBe(true);
  });

  it("accepts rgba format", () => {
    expect(isValidCssColor("rgba(255, 0, 128, 0.5)")).toBe(true);
    expect(isValidCssColor("rgba(0, 0, 0, 1)")).toBe(true);
    expect(isValidCssColor("rgba(0, 0, 0, 0)")).toBe(true);
  });

  it("accepts hsl format", () => {
    expect(isValidCssColor("hsl(120, 50%, 50%)")).toBe(true);
  });

  it("accepts hsla format", () => {
    expect(isValidCssColor("hsla(120, 50%, 50%, 0.8)")).toBe(true);
  });

  it("rejects invalid strings", () => {
    expect(isValidCssColor("not-a-color")).toBe(false);
    expect(isValidCssColor("")).toBe(false);
    expect(isValidCssColor("123")).toBe(false);
    expect(isValidCssColor("#gg0000")).toBe(false);
    expect(isValidCssColor("red")).toBe(false);
  });
});

function makeValidTokens(): ThemeColorTokens {
  return {
    bg: "#000000",
    surface: "#111111",
    text: "#ffffff",
    textSoft: "#cccccc",
    textMuted: "#999999",
    accent: "#ff0000",
    accentStrong: "#cc0000",
    accentSoft: "rgba(255, 0, 0, 0.2)",
    border: "#333333",
    borderStrong: "#555555",
    danger: "#ff0000",
    dangerStrong: "#cc0000",
    dangerSoft: "rgba(255, 0, 0, 0.1)",
    warning: "#ffaa00",
    surfaceOverlay: "rgba(255, 255, 255, 0.05)",
    surfaceHover: "rgba(255, 255, 255, 0.1)",
    scrollbar: "rgba(255, 255, 255, 0.2)",
    tokenKeyword: "#c792ea",
    tokenString: "#c3e88d",
    tokenComment: "#637777",
    tokenNumber: "#f78c6c",
    tokenType: "#ffcb6b",
    tokenPunctuation: "#89ddff",
  };
}

describe("validateThemeTokens", () => {
  it("returns empty array for valid tokens", () => {
    expect(validateThemeTokens(makeValidTokens())).toEqual([]);
  });

  it("returns token names for invalid colors", () => {
    const tokens: ThemeColorTokens = {
      ...makeValidTokens(),
      bg: "invalid",
      text: "also-bad",
    };
    const result = validateThemeTokens(tokens);
    expect(result).toContain("bg");
    expect(result).toContain("text");
    expect(result).toHaveLength(2);
  });
});

describe("generateThemeId", () => {
  it("returns string starting with 'custom-'", () => {
    const id = generateThemeId();
    expect(id.startsWith("custom-")).toBe(true);
  });

  it("returns unique IDs on repeated calls", () => {
    const ids = new Set(Array.from({ length: 20 }, () => generateThemeId()));
    expect(ids.size).toBe(20);
  });
});
