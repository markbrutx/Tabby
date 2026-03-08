import { shellClients } from "@/app-shell/clients";
import { createWorkspaceStore } from "@/features/workspace/application/store";
import { createSettingsStore } from "@/features/settings/application/store";
import { createRuntimeStore } from "@/features/runtime/application/store";

export const useSettingsStore = createSettingsStore(shellClients.settings);

export const useRuntimeStore = createRuntimeStore(shellClients.runtime);

export const useWorkspaceStore = createWorkspaceStore({
  workspaceClient: shellClients.workspace,
  getSettingsStore: () => useSettingsStore,
  getRuntimeStore: () => useRuntimeStore,
});
