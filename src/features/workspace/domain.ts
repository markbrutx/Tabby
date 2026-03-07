export type {
  AppSettings as WorkspaceSettings,
  BootstrapSnapshot,
  BrowserUrlChangedEvent,
  LayoutPreset,
  NewTabRequest,
  PaneKind,
  PaneLifecycleEvent,
  PaneProfile,
  PaneSnapshot,
  PtyOutputEvent,
  PtyResizeRequest,
  SplitDirection,
  SplitNode,
  SplitPaneRequest,
  TabSnapshot,
  ThemeMode,
  UpdatePaneCwdRequest,
  UpdatePaneProfileRequest,
  WorkspaceSnapshot,
} from "@/lib/tauri-bindings";

export const CUSTOM_PROFILE_ID = "custom" as const;
export const BROWSER_PROFILE_ID = "browser" as const;
export const DEFAULT_BROWSER_URL = "https://google.com" as const;
