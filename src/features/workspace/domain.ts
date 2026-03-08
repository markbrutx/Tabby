export type {
  BrowserLocationObservedEvent,
  BrowserSurfaceBoundsDto as BrowserBounds,
  BrowserSurfaceCommandDto,
  LayoutPresetDto as LayoutPreset,
  PaneRuntimeView,
  PaneSpecDto,
  PaneView,
  ProfileCatalogView,
  ProfileView as PaneProfile,
  RuntimeCommandDto,
  RuntimeStatusChangedEvent,
  RuntimeStatusDto as PaneRuntimeStatus,
  SettingsCommandDto,
  SettingsProjectionUpdatedEvent,
  SettingsView as WorkspaceSettings,
  SplitDirectionDto as SplitDirection,
  SplitNodeDto as SplitNode,
  TabView,
  TerminalOutputEvent,
  ThemeModeDto as ThemeMode,
  WorkspaceBootstrapView,
  WorkspaceCommandDto,
  WorkspaceProjectionUpdatedEvent,
  WorkspaceView,
} from "@/contracts/tauri-bindings";

export const CUSTOM_PROFILE_ID = "custom" as const;
export const BROWSER_PROFILE_ID = "browser" as const;
export const DEFAULT_BROWSER_URL = "https://google.com" as const;
