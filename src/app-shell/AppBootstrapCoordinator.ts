import { asErrorMessage } from "@/app-shell/clients";
import type { WorkspaceClient } from "@/app-shell/clients";
import type { WorkspaceReadModel } from "@/features/workspace/domain/models";
import type { ProfileReadModel, SettingsReadModel } from "@/features/settings/domain/models";
import type { RuntimeReadModel } from "@/features/runtime/domain/models";
import { mapWorkspaceFromDto } from "@/features/workspace/application/snapshot-mappers";
import { mapProfileFromDto, mapSettingsFromDto } from "@/features/settings/application/snapshot-mappers";
import { mapRuntimeFromDto } from "@/features/runtime/application/snapshot-mappers";

export interface BootstrapableWorkspaceStore {
  getState: () => {
    beginBootstrap: () => void;
    loadBootstrap: (workspace: WorkspaceReadModel) => Promise<void>;
    setBootstrapError: (message: string) => void;
  };
}

export interface BootstrapableSettingsStore {
  getState: () => {
    settings: { readonly hasCompletedOnboarding: boolean } | null;
    loadBootstrap: (
      settings: SettingsReadModel,
      profiles: readonly ProfileReadModel[],
    ) => void;
    markOnboardingComplete: () => Promise<void>;
  };
}

export interface BootstrapableRuntimeStore {
  getState: () => {
    loadBootstrap: (runtimes: readonly RuntimeReadModel[]) => void;
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

        const settingsReadModel = mapSettingsFromDto(payload.settings);
        const profileReadModels = payload.profileCatalog.terminalProfiles.map(mapProfileFromDto);
        deps.settingsStore.getState().loadBootstrap(settingsReadModel, profileReadModels);

        const runtimeReadModels = payload.runtimeProjections.map(mapRuntimeFromDto);
        deps.runtimeStore.getState().loadBootstrap(runtimeReadModels);

        const workspaceReadModel = mapWorkspaceFromDto(payload.workspace);
        await deps.workspaceStore.getState().loadBootstrap(workspaceReadModel);
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
