export type PaneGroupConfig =
  | { mode: "terminal"; profileId: string; workingDirectory: string; customCommand: string; count: number }
  | { mode: "browser"; url: string; count: number }
  | { mode: "git"; workingDirectory: string; count: number };

export interface SetupWizardConfig {
  groups: PaneGroupConfig[];
  layoutVariantId: string | null;
}

export interface WizardTab {
  id: string;
  title: string;
}
