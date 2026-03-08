import { shellClients } from "@/app-shell/clients";
import { createAppBootstrapCoordinator } from "@/app-shell/AppBootstrapCoordinator";
import { createWorkspaceStore } from "@/features/workspace/application/store";
import { createSettingsStore } from "@/features/settings/application/store";
import { createRuntimeStore } from "@/features/runtime/application/store";
import {
  initDispatcher,
  teardownDispatcher,
} from "@/features/terminal/ptyOutputDispatcher";

export const useSettingsStore = createSettingsStore(shellClients.settings);

export const useRuntimeStore = createRuntimeStore({
  runtimeClient: shellClients.runtime,
  initTerminalDispatcher: initDispatcher,
  teardownTerminalDispatcher: teardownDispatcher,
});

const coordinatorDeps = {
  workspaceClient: shellClients.workspace,
  settingsStore: useSettingsStore,
  runtimeStore: useRuntimeStore,
};

// Coordinator is created first so workspace store can reference it via closure.
// The workspaceStore dep is assigned immediately after store creation.
let coordinator: ReturnType<typeof createAppBootstrapCoordinator>;

export const useWorkspaceStore = createWorkspaceStore({
  workspaceClient: shellClients.workspace,
  onWizardComplete: () => {
    void coordinator.completeOnboarding();
  },
});

coordinator = createAppBootstrapCoordinator({
  ...coordinatorDeps,
  workspaceStore: useWorkspaceStore,
});

export const bootstrapCoordinator = coordinator;
