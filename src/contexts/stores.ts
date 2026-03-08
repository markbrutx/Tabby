import { shellClients } from "@/app-shell/clients";
import { createWorkspaceStore } from "@/contexts/workspace/store";
import { createSettingsStore } from "@/contexts/settings/store";
import { createRuntimeStore } from "@/contexts/runtime/store";

export const useSettingsStore = createSettingsStore(shellClients.settings);

export const useRuntimeStore = createRuntimeStore(shellClients.runtime);

export const useWorkspaceStore = createWorkspaceStore({
  workspaceClient: shellClients.workspace,
  getSettingsStore: () => useSettingsStore,
  getRuntimeStore: () => useRuntimeStore,
});
