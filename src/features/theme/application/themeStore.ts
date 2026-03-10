import { create } from "zustand";
import type {
  ThemeColorTokens,
  ThemeDefinition,
  ThemeExportFormat,
  ThemeKind,
} from "../domain/models";
import { BUILT_IN_THEMES, findThemeById } from "../domain/presets";
import { generateThemeId, validateThemeTokens } from "../domain/validation";
import { applyPartialTokens, applyTheme } from "./themeApplicator";

const CUSTOM_THEMES_KEY = "tabby-custom-themes";

function loadCustomThemes(): ThemeDefinition[] {
  try {
    const raw = localStorage.getItem(CUSTOM_THEMES_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as ThemeDefinition[];
    return parsed.map((t) => ({ ...t, builtIn: false }));
  } catch {
    return [];
  }
}

function saveCustomThemes(themes: readonly ThemeDefinition[]): void {
  const custom = themes.filter((t) => !t.builtIn);
  localStorage.setItem(CUSTOM_THEMES_KEY, JSON.stringify(custom));
}

function getMidnight(): ThemeDefinition {
  return BUILT_IN_THEMES[0];
}

function resolveThemeFromList(
  themes: readonly ThemeDefinition[],
  themeId: string,
): ThemeDefinition {
  if (themeId === "system") {
    const prefersDark =
      typeof window !== "undefined" &&
      window.matchMedia("(prefers-color-scheme: dark)").matches;
    const systemId = prefersDark ? "midnight" : "dawn";
    return findThemeById(themes, systemId) ?? getMidnight();
  }
  return findThemeById(themes, themeId) ?? getMidnight();
}

interface ThemeState {
  readonly themes: readonly ThemeDefinition[];
  readonly activeThemeId: string;
  readonly resolvedTheme: ThemeDefinition;
  readonly draft: ThemeDefinition | null;

  initialize: () => void;
  selectTheme: (themeId: string) => void;
  resolveSystemTheme: () => void;

  createTheme: (theme: ThemeDefinition) => void;
  cloneTheme: (sourceId: string, newName: string) => ThemeDefinition;
  updateTheme: (theme: ThemeDefinition) => void;
  deleteTheme: (themeId: string) => void;

  openEditor: (themeId: string | null) => void;
  updateDraft: (tokens: Partial<ThemeColorTokens>) => void;
  updateDraftMeta: (meta: { name?: string; kind?: ThemeKind }) => void;
  saveDraft: () => void;
  discardDraft: () => void;

  importTheme: (json: string) => ThemeDefinition;
  exportTheme: (themeId: string) => string;
}

export const useThemeStore = create<ThemeState>((set, get) => ({
  themes: [],
  activeThemeId: "midnight",
  resolvedTheme: getMidnight(),
  draft: null,

  initialize() {
    const custom = loadCustomThemes();
    const allThemes = [...BUILT_IN_THEMES, ...custom];
    const resolved = resolveThemeFromList(allThemes, get().activeThemeId);
    set({ themes: allThemes, resolvedTheme: resolved });
    applyTheme(resolved);
  },

  selectTheme(themeId: string) {
    const resolved = resolveThemeFromList(get().themes, themeId);
    set({ activeThemeId: themeId, resolvedTheme: resolved });
    applyTheme(resolved);
  },

  resolveSystemTheme() {
    const { activeThemeId, themes } = get();
    if (activeThemeId !== "system") return;
    const resolved = resolveThemeFromList(themes, "system");
    set({ resolvedTheme: resolved });
    applyTheme(resolved);
  },

  createTheme(theme: ThemeDefinition) {
    const newTheme: ThemeDefinition = { ...theme, builtIn: false };
    const nextThemes = [...get().themes, newTheme];
    set({ themes: nextThemes });
    saveCustomThemes(nextThemes);
  },

  cloneTheme(sourceId: string, newName: string): ThemeDefinition {
    const source = findThemeById(get().themes, sourceId);
    if (!source) {
      throw new Error(`Theme not found: ${sourceId}`);
    }
    const cloned: ThemeDefinition = {
      ...source,
      id: generateThemeId(),
      name: newName,
      builtIn: false,
    };
    const nextThemes = [...get().themes, cloned];
    set({ themes: nextThemes });
    saveCustomThemes(nextThemes);
    return cloned;
  },

  updateTheme(theme: ThemeDefinition) {
    const nextThemes = get().themes.map((t) =>
      t.id === theme.id ? { ...theme, builtIn: false } : t,
    );
    set({ themes: nextThemes });
    saveCustomThemes(nextThemes);

    if (get().activeThemeId === theme.id) {
      const resolved = resolveThemeFromList(nextThemes, theme.id);
      set({ resolvedTheme: resolved });
      applyTheme(resolved);
    }
  },

  deleteTheme(themeId: string) {
    const target = findThemeById(get().themes, themeId);
    if (!target || target.builtIn) return;

    const nextThemes = get().themes.filter((t) => t.id !== themeId);
    set({ themes: nextThemes });
    saveCustomThemes(nextThemes);

    if (get().activeThemeId === themeId) {
      const resolved = resolveThemeFromList(nextThemes, "midnight");
      set({
        activeThemeId: "midnight",
        resolvedTheme: resolved,
      });
      applyTheme(resolved);
    }
  },

  openEditor(themeId: string | null) {
    if (themeId === null) {
      const newDraft: ThemeDefinition = {
        id: generateThemeId(),
        name: "New Theme",
        kind: "dark",
        builtIn: false,
        colors: { ...getMidnight().colors },
      };
      set({ draft: newDraft });
      return;
    }
    const source = findThemeById(get().themes, themeId);
    if (!source) return;
    set({ draft: { ...source } });
  },

  updateDraft(tokens: Partial<ThemeColorTokens>) {
    const { draft } = get();
    if (!draft) return;
    const updatedDraft: ThemeDefinition = {
      ...draft,
      colors: { ...draft.colors, ...tokens },
    };
    set({ draft: updatedDraft });
    applyPartialTokens(tokens);
  },

  updateDraftMeta(meta: { name?: string; kind?: ThemeKind }) {
    const { draft } = get();
    if (!draft) return;
    set({
      draft: {
        ...draft,
        ...(meta.name !== undefined ? { name: meta.name } : {}),
        ...(meta.kind !== undefined ? { kind: meta.kind } : {}),
      },
    });
  },

  saveDraft() {
    const { draft, themes } = get();
    if (!draft) return;

    const existing = findThemeById(themes, draft.id);
    if (existing) {
      get().updateTheme(draft);
    } else {
      get().createTheme(draft);
    }
    set({ draft: null });
  },

  discardDraft() {
    set({ draft: null });
    const resolved = resolveThemeFromList(get().themes, get().activeThemeId);
    applyTheme(resolved);
  },

  importTheme(json: string): ThemeDefinition {
    let parsed: ThemeExportFormat;
    try {
      parsed = JSON.parse(json) as ThemeExportFormat;
    } catch {
      throw new Error("Invalid JSON format");
    }

    if (parsed.formatVersion !== 1) {
      throw new Error(
        `Unsupported format version: ${String(parsed.formatVersion)}`,
      );
    }

    if (!parsed.theme || !parsed.theme.colors) {
      throw new Error("Missing theme data");
    }

    const invalidTokens = validateThemeTokens(parsed.theme.colors);
    if (invalidTokens.length > 0) {
      throw new Error(
        `Invalid color tokens: ${invalidTokens.join(", ")}`,
      );
    }

    const imported: ThemeDefinition = {
      id: generateThemeId(),
      name: parsed.theme.name,
      kind: parsed.theme.kind,
      builtIn: false,
      colors: { ...parsed.theme.colors },
    };

    get().createTheme(imported);
    return imported;
  },

  exportTheme(themeId: string): string {
    const theme = findThemeById(get().themes, themeId);
    if (!theme) {
      throw new Error(`Theme not found: ${themeId}`);
    }

    const exportData: ThemeExportFormat = {
      formatVersion: 1,
      theme: {
        id: theme.id,
        name: theme.name,
        kind: theme.kind,
        colors: { ...theme.colors },
      },
    };

    return JSON.stringify(exportData, null, 2);
  },
}));
