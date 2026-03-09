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
    settings: { readonly hasCompletedOnboarding: boolean } | null;
    loadBootstrap: (
      settings: WorkspaceBootstrapView["settings"],
      profiles: WorkspaceBootstrapView["profileCatalog"]["terminalProfiles"],
    ) => void;
    markOnboardingComplete: () => Promise<void>;
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

interface AppBootstrapCoordinator {
  initialize: () => Promise<void>;
  completeOnboarding: () => Promise<void>;
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

    async completeOnboarding() {
      const settingsState = deps.settingsStore.getState();
      const settings = settingsState.settings;
      if (settings && !settings.hasCompletedOnboarding) {
        await settingsState.markOnboardingComplete();
      }
    },
  };
}
