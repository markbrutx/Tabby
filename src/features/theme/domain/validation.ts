import type { ThemeColorTokens } from "./models";

const HEX_COLOR_REGEX =
  /^#([0-9a-fA-F]{3}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/;
const RGB_REGEX =
  /^rgba?\(\s*\d{1,3}\s*,\s*\d{1,3}\s*,\s*\d{1,3}\s*(,\s*(0|1|0?\.\d+))?\s*\)$/;
const HSL_REGEX =
  /^hsla?\(\s*\d{1,3}\s*,\s*\d{1,3}%\s*,\s*\d{1,3}%\s*(,\s*(0|1|0?\.\d+))?\s*\)$/;

const THEME_TOKEN_KEYS: readonly (keyof ThemeColorTokens)[] = [
  "bg",
  "surface",
  "text",
  "textSoft",
  "textMuted",
  "accent",
  "accentStrong",
  "accentSoft",
  "border",
  "borderStrong",
  "danger",
  "dangerStrong",
  "dangerSoft",
  "warning",
  "surfaceOverlay",
  "surfaceHover",
  "scrollbar",
  "tokenKeyword",
  "tokenString",
  "tokenComment",
  "tokenNumber",
  "tokenType",
  "tokenPunctuation",
];

export function isValidCssColor(value: string): boolean {
  return (
    HEX_COLOR_REGEX.test(value) ||
    RGB_REGEX.test(value) ||
    HSL_REGEX.test(value)
  );
}

export function validateThemeTokens(tokens: ThemeColorTokens): string[] {
  return THEME_TOKEN_KEYS.filter((key) => !isValidCssColor(tokens[key]));
}

export function generateThemeId(): string {
  return `custom-${crypto.randomUUID().slice(0, 8)}`;
}
