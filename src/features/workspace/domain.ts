export type {
  AppSettings as WorkspaceSettings,
  BootstrapSnapshot,
  GridDefinition,
  LayoutPreset,
  NewTabRequest,
  PaneProfile,
  PaneRuntimeStatus,
  PaneSnapshot,
  PtyOutputEvent,
  PtyResizeRequest,
  TabSnapshot,
  ThemeMode,
  UpdatePaneCwdRequest,
  UpdatePaneProfileRequest,
  WorkspaceSnapshot,
} from "@/lib/tauri-bindings";

export const CUSTOM_PROFILE_ID = "custom" as const;
