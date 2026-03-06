export type LayoutPreset = "1x1" | "1x2" | "2x2" | "2x3" | "3x3";
export type ThemeMode = "system" | "dawn" | "midnight";

export interface GridDefinition {
  preset: LayoutPreset;
  rows: number;
  columns: number;
  paneCount: number;
}

export interface WorkspaceSettings {
  defaultLayout: LayoutPreset;
  defaultProfileId: string;
  defaultWorkingDirectory: string;
  defaultCustomCommand: string;
  fontSize: number;
  theme: ThemeMode;
  launchFullscreen: boolean;
}

export interface PaneProfile {
  id: string;
  label: string;
  description: string;
  startupCommand: string | null;
}

export interface PaneSnapshot {
  id: string;
  sessionId: string;
  title: string;
  cwd: string;
  profileId: string;
  profileLabel: string;
  startupCommand: string | null;
}

export interface TabSnapshot {
  id: string;
  title: string;
  preset: LayoutPreset;
  panes: PaneSnapshot[];
  activePaneId: string;
}

export interface WorkspaceSnapshot {
  activeTabId: string;
  tabs: TabSnapshot[];
}

export interface BootstrapSnapshot {
  workspace: WorkspaceSnapshot;
  settings: WorkspaceSettings;
  profiles: PaneProfile[];
}

export interface NewTabRequest {
  preset: LayoutPreset;
  cwd?: string | null;
  profileId?: string | null;
  startupCommand?: string | null;
}

export interface UpdatePaneProfileRequest {
  paneId: string;
  profileId: string;
  startupCommand?: string | null;
}

export interface UpdatePaneCwdRequest {
  paneId: string;
  cwd: string;
}

export interface PtyResizeRequest {
  paneId: string;
  cols: number;
  rows: number;
}

export interface PtyOutputEvent {
  paneId: string;
  sessionId: string;
  chunk: string;
}
