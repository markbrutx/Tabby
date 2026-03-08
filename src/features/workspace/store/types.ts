export interface PaneGroupConfig {
  mode: "terminal" | "browser";
  profileId: string;
  workingDirectory: string;
  customCommand?: string;
  url?: string;
  count: number;
}

export interface SetupWizardConfig {
  groups: PaneGroupConfig[];
}

export interface WizardTab {
  id: string;
  title: string;
}
