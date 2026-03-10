import { useEffect } from "react";
import type { ITheme } from "xterm";
import type { ThemeKind, ThemeDefinition } from "@/features/theme/domain/models";
import { applyTheme } from "@/features/theme/application/themeApplicator";
import { useThemeStore } from "@/features/theme/application/themeStore";

const TERMINAL_THEMES: Record<ThemeKind, ITheme> = {
  light: {
    background: "#fff7f1",
    foreground: "#5f463b",
    cursor: "#db735b",
    selectionBackground: "rgba(219, 115, 91, 0.22)",
    black: "#3b2e2a",
    red: "#d66f61",
    green: "#7f9f89",
    yellow: "#c38b59",
    blue: "#7ea5c7",
    magenta: "#d996a7",
    cyan: "#7abac1",
    white: "#fffdfa",
    brightBlack: "#8b7568",
    brightRed: "#e98978",
    brightGreen: "#96b09c",
    brightYellow: "#d7a571",
    brightBlue: "#9bb8d5",
    brightMagenta: "#e6b0bf",
    brightCyan: "#98cad0",
    brightWhite: "#ffffff",
  },
  dark: {
    background: "#130c08",
    foreground: "#f8ece2",
    cursor: "#e97d61",
    selectionBackground: "rgba(233, 125, 97, 0.24)",
    black: "#1a1210",
    red: "#f08475",
    green: "#8eb79a",
    yellow: "#d6a06f",
    blue: "#9db6d7",
    magenta: "#e1a6b5",
    cyan: "#8dc8d0",
    white: "#f8ece2",
    brightBlack: "#8f776a",
    brightRed: "#f4a08e",
    brightGreen: "#a4c3a9",
    brightYellow: "#e3b486",
    brightBlue: "#b4c7e2",
    brightMagenta: "#eab9c5",
    brightCyan: "#a9d4db",
    brightWhite: "#fffaf5",
  },
};

export function getTerminalTheme(kind: ThemeKind): ITheme {
  return TERMINAL_THEMES[kind];
}

export function applyResolvedTheme(theme: ThemeDefinition): void {
  applyTheme(theme);
}

export function useResolvedTheme(themeId: string | undefined): ThemeDefinition {
  const selectTheme = useThemeStore((s) => s.selectTheme);
  const resolveSystemTheme = useThemeStore((s) => s.resolveSystemTheme);
  const resolvedTheme = useThemeStore((s) => s.resolvedTheme);

  useEffect(() => {
    if (themeId) {
      selectTheme(themeId);
    }
  }, [themeId, selectTheme]);

  useEffect(() => {
    if (!themeId || themeId !== "system") return;
    if (typeof window === "undefined") return;

    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    const handleChange = () => resolveSystemTheme();

    mediaQuery.addEventListener("change", handleChange);
    return () => mediaQuery.removeEventListener("change", handleChange);
  }, [themeId, resolveSystemTheme]);

  return resolvedTheme;
}
