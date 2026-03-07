export type {
  AppSettings as WorkspaceSettings,
  BootstrapSnapshot,
  LayoutPreset,
  NewTabRequest,
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
