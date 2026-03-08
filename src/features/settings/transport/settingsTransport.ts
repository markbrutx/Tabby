import type { WorkspaceSettings } from "@/features/workspace/domain";

export interface SettingsTransport {
  getAppSettings: () => Promise<WorkspaceSettings>;
  updateAppSettings: (settings: WorkspaceSettings) => Promise<WorkspaceSettings>;
  resetAppSettings: () => Promise<WorkspaceSettings>;
}
