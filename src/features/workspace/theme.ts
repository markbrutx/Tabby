import { useEffect, useState } from "react";
import type { ITheme } from "xterm";
import type { ThemeMode } from "@/features/workspace/domain";

export type ResolvedTheme = Exclude<ThemeMode, "system">;

const TERMINAL_THEMES: Record<ResolvedTheme, ITheme> = {
  dawn: {
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
  midnight: {
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

function getSystemPrefersDark(): boolean {
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
    return true;
  }

  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

export function resolveThemeMode(
  themeMode: ThemeMode | undefined,
  prefersDark: boolean,
): ResolvedTheme {
  if (!themeMode || themeMode === "system") {
    return prefersDark ? "midnight" : "dawn";
  }

  return themeMode;
}

export function applyResolvedTheme(theme: ResolvedTheme): void {
  if (typeof document === "undefined") {
    return;
  }

  const root = document.documentElement;
  root.dataset.theme = theme;
  root.style.colorScheme = theme === "dawn" ? "light" : "dark";
}

export function getTerminalTheme(theme: ResolvedTheme): ITheme {
  return TERMINAL_THEMES[theme];
}

export function useResolvedTheme(themeMode: ThemeMode | undefined): ResolvedTheme {
  const [prefersDark, setPrefersDark] = useState(getSystemPrefersDark);

  useEffect(() => {
    if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
      return;
    }

    if (themeMode && themeMode !== "system") {
      return;
    }

    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    const handleChange = () => setPrefersDark(mediaQuery.matches);

    handleChange();

    mediaQuery.addEventListener("change", handleChange);
    return () => mediaQuery.removeEventListener("change", handleChange);
  }, [themeMode]);

  return resolveThemeMode(themeMode, prefersDark);
}
