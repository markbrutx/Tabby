import { asErrorMessage } from "@/app-shell/clients";
import type { WorkspaceClient } from "@/app-shell/clients";
import type { WorkspaceBootstrapView } from "@/contracts/tauri-bindings";

export interface BootstrapableWorkspaceStore {
  getState: () => {
    beginBootstrap: () => void;
    loadBootstrap: (payload: WorkspaceBootstrapView) => Promise<void>;
    setBootstrapError: (message: string) => void;
  };
}

export interface BootstrapableSettingsStore {
  getState: () => {
    loadBootstrap: (
      settings: WorkspaceBootstrapView["settings"],
      profiles: WorkspaceBootstrapView["profileCatalog"]["terminalProfiles"],
    ) => void;
  };
}

export interface BootstrapableRuntimeStore {
  getState: () => {
    loadBootstrap: (runtimes: WorkspaceBootstrapView["runtimeProjections"]) => void;
  };
}

export interface AppBootstrapCoordinatorDeps {
  workspaceClient: WorkspaceClient;
  workspaceStore: BootstrapableWorkspaceStore;
  settingsStore: BootstrapableSettingsStore;
  runtimeStore: BootstrapableRuntimeStore;
}

export interface AppBootstrapCoordinator {
  initialize: () => Promise<void>;
}

export function createAppBootstrapCoordinator(
  deps: AppBootstrapCoordinatorDeps,
): AppBootstrapCoordinator {
  return {
    async initialize() {
      deps.workspaceStore.getState().beginBootstrap();

      try {
        const payload = await deps.workspaceClient.bootstrap();

        deps.settingsStore.getState().loadBootstrap(
          payload.settings,
          payload.profileCatalog.terminalProfiles,
        );

        deps.runtimeStore.getState().loadBootstrap(payload.runtimeProjections);

        await deps.workspaceStore.getState().loadBootstrap(payload);
      } catch (error) {
        deps.workspaceStore.getState().setBootstrapError(asErrorMessage(error));
      }
    },
  };
}
