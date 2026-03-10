import type { ThemeColorTokens, ThemeDefinition } from "../domain/models";

const TOKEN_TO_CSS_VAR: Record<keyof ThemeColorTokens, string> = {
  bg: "--color-bg",
  surface: "--color-surface",
  text: "--color-text",
  textSoft: "--color-text-soft",
  textMuted: "--color-text-muted",
  accent: "--color-accent",
  accentStrong: "--color-accent-strong",
  accentSoft: "--color-accent-soft",
  border: "--color-border",
  borderStrong: "--color-border-strong",
  danger: "--color-danger",
  dangerStrong: "--color-danger-strong",
  dangerSoft: "--color-danger-soft",
  warning: "--color-warning",
  surfaceOverlay: "--color-surface-overlay",
  surfaceHover: "--color-surface-hover",
  scrollbar: "--color-scrollbar",
  tokenKeyword: "--color-token-keyword",
  tokenString: "--color-token-string",
  tokenComment: "--color-token-comment",
  tokenNumber: "--color-token-number",
  tokenType: "--color-token-type",
  tokenPunctuation: "--color-token-punctuation",
};

export function applyTheme(theme: ThemeDefinition): void {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  for (const [key, cssVar] of Object.entries(TOKEN_TO_CSS_VAR)) {
    root.style.setProperty(cssVar, theme.colors[key as keyof ThemeColorTokens]);
  }
  root.style.colorScheme = theme.kind === "light" ? "light" : "dark";
}

export function applyPartialTokens(tokens: Partial<ThemeColorTokens>): void {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  for (const [key, value] of Object.entries(tokens)) {
    const cssVar = TOKEN_TO_CSS_VAR[key as keyof ThemeColorTokens];
    if (cssVar && value) {
      root.style.setProperty(cssVar, value);
    }
  }
}

export { TOKEN_TO_CSS_VAR };
