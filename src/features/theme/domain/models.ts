export type ThemeKind = "dark" | "light";

export interface ThemeColorTokens {
  readonly bg: string;
  readonly surface: string;
  readonly text: string;
  readonly textSoft: string;
  readonly textMuted: string;
  readonly accent: string;
  readonly accentStrong: string;
  readonly accentSoft: string;
  readonly border: string;
  readonly borderStrong: string;
  readonly danger: string;
  readonly dangerStrong: string;
  readonly dangerSoft: string;
  readonly warning: string;
  readonly surfaceOverlay: string;
  readonly surfaceHover: string;
  readonly scrollbar: string;
  readonly tokenKeyword: string;
  readonly tokenString: string;
  readonly tokenComment: string;
  readonly tokenNumber: string;
  readonly tokenType: string;
  readonly tokenPunctuation: string;
}

export interface ThemeDefinition {
  readonly id: string;
  readonly name: string;
  readonly kind: ThemeKind;
  readonly builtIn: boolean;
  readonly colors: ThemeColorTokens;
}

export interface ThemeExportFormat {
  readonly formatVersion: 1;
  readonly theme: Omit<ThemeDefinition, "builtIn">;
}
