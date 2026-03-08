import { create } from "zustand";
import { asErrorMessage } from "@/app-shell/clients";
import type {
  PaneRuntimeView,
  PaneSpecDto,
  SettingsView,
  WorkspaceCommandDto,
  WorkspaceView,
} from "@/contracts/tauri-bindings";
import {
  CUSTOM_PROFILE_ID,
} from "@/features/workspace/domain/models";
import type {
  PaneSpec,
  SplitDirection,
  WorkspaceReadModel,
} from "@/features/workspace/domain/models";
import { mapPaneSpecToDto, mapWorkspaceFromDto } from "@/features/workspace/application/snapshot-mappers";
import type { WorkspaceClient } from "@/app-shell/clients";
import type { SettingsReadModel } from "@/features/settings/domain/models";
import type { SetupWizardConfig, WizardTab } from "@/features/workspace/store/types";

export interface WorkspaceStore {
  workspace: WorkspaceReadModel | null;
  error: string | null;
  isHydrating: boolean;
  isWorking: boolean;
  wizardTab: WizardTab | null;
  initialize: () => Promise<void>;
  createTabFromWizard: (config: SetupWizardConfig) => Promise<void>;
  openSetupWizard: () => void;
  closeSetupWizard: () => void;
  closeTab: (tabId: string) => Promise<void>;
  setActiveTab: (tabId: string) => Promise<void>;
  focusPane: (tabId: string, paneId: string) => Promise<void>;
  replacePaneSpec: (paneId: string, paneSpec: PaneSpec) => Promise<void>;
  restartPaneRuntime: (paneId: string) => Promise<void>;
  splitPane: (
    paneId: string,
    direction: SplitDirection,
    paneSpec: PaneSpec,
  ) => Promise<void>;
  closePane: (paneId: string) => Promise<void>;
  swapPanes: (paneIdA: string, paneIdB: string) => Promise<void>;
  clearError: () => void;
}

type SetFn = (
  partial:
    | Partial<WorkspaceStore>
    | ((state: WorkspaceStore) => Partial<WorkspaceStore>),
) => void;

function makeWizardTab(workspace?: WorkspaceReadModel | null): WizardTab {
  const nextIndex = (workspace?.tabs.length ?? 0) + 1;
  return {
    id: `__wizard_${Date.now()}__`,
    title: `Workspace ${nextIndex}`,
  };
}

async function runWorkspaceMutation(
  set: SetFn,
  mutation: () => Promise<WorkspaceView>,
  onSuccess?: (workspace: WorkspaceReadModel) => Partial<WorkspaceStore>,
) {
  set({ isWorking: true });
  try {
    const dto = await mutation();
    const workspace = mapWorkspaceFromDto(dto);
    set({
      workspace,
      error: null,
      isWorking: false,
      wizardTab: workspace.tabs.length === 0 ? makeWizardTab(workspace) : null,
      ...(onSuccess?.(workspace) ?? {}),
    });
  } catch (error) {
    set({ error: asErrorMessage(error), isWorking: false });
  }
}

function toPaneSpec(group: SetupWizardConfig["groups"][number]): PaneSpecDto[] {
  const isBrowser = group.mode === "browser";
  return Array.from({ length: group.count }, () =>
    isBrowser
      ? {
          kind: "browser",
          initial_url: group.url || "https://google.com",
        }
      : {
          kind: "terminal",
          launch_profile_id: group.profileId || "terminal",
          working_directory: group.workingDirectory || "~",
          command_override:
            group.profileId === CUSTOM_PROFILE_ID ? group.customCommand?.trim() || null : null,
        },
  );
}

export interface WorkspaceStoreDeps {
  workspaceClient: WorkspaceClient;
  getSettingsStore: () => {
    getState: () => {
      settings: SettingsReadModel | null;
      loadBootstrap: (settings: SettingsView, profiles: readonly { id: string; label: string; description: string; startupCommandTemplate: string | null }[]) => void;
      updateSettings: (settings: SettingsReadModel) => Promise<void>;
    };
  };
  getRuntimeStore: () => {
    getState: () => {
      loadBootstrap: (runtimes: PaneRuntimeView[]) => void;
    };
  };
}

export function createWorkspaceStore(deps: WorkspaceStoreDeps) {
  let workspaceListenersReady: Promise<void> | null = null;

  return create<WorkspaceStore>((set, get) => ({
    workspace: null,
    error: null,
    isHydrating: true,
    isWorking: false,
    wizardTab: null,

    async initialize() {
      set({ isHydrating: true, error: null });

      try {
        const payload = await deps.workspaceClient.bootstrap();
        deps.getSettingsStore().getState().loadBootstrap(
          payload.settings,
          payload.profileCatalog.terminalProfiles,
        );
        deps.getRuntimeStore().getState().loadBootstrap(payload.runtimeProjections);

        if (!workspaceListenersReady) {
          workspaceListenersReady = deps.workspaceClient
            .listenProjectionUpdated((dto) => {
              const workspace = mapWorkspaceFromDto(dto);
              set({
                workspace,
                error: null,
                wizardTab: workspace.tabs.length === 0 ? makeWizardTab(workspace) : null,
              });
            })
            .then(() => undefined);
        }
        await workspaceListenersReady;

        const workspace = mapWorkspaceFromDto(payload.workspace);
        const shouldShowWizard = workspace.tabs.length === 0;
        set({
          workspace,
          error: null,
          isHydrating: false,
          wizardTab: shouldShowWizard ? makeWizardTab(workspace) : null,
        });
      } catch (error) {
        set({
          error: asErrorMessage(error),
          isHydrating: false,
        });
      }
    },

    async createTabFromWizard(config) {
      await runWorkspaceMutation(
        set,
        () =>
          deps.workspaceClient.dispatch({
            kind: "openTab",
            layout: null,
            auto_layout: true,
            pane_specs: config.groups.flatMap(toPaneSpec),
          } satisfies WorkspaceCommandDto),
        () => {
          const settingsState = deps.getSettingsStore().getState();
          const settings = settingsState.settings;
          if (settings && !settings.hasCompletedOnboarding) {
            void settingsState.updateSettings({ ...settings, hasCompletedOnboarding: true });
          }
          return {};
        },
      );
    },

    openSetupWizard() {
      set({ wizardTab: makeWizardTab(get().workspace) });
    },

    closeSetupWizard() {
      const workspace = get().workspace;
      if (workspace && workspace.tabs.length === 0) {
        return;
      }
      set({ wizardTab: null });
    },

    async closeTab(tabId) {
      await runWorkspaceMutation(set, () =>
        deps.workspaceClient.dispatch({
          kind: "closeTab",
          tab_id: tabId,
        } satisfies WorkspaceCommandDto),
      );
    },

    async setActiveTab(tabId) {
      await runWorkspaceMutation(set, () =>
        deps.workspaceClient.dispatch({
          kind: "setActiveTab",
          tab_id: tabId,
        } satisfies WorkspaceCommandDto),
      );
    },

    async focusPane(tabId, paneId) {
      await runWorkspaceMutation(set, () =>
        deps.workspaceClient.dispatch({
          kind: "focusPane",
          tab_id: tabId,
          pane_id: paneId,
        } satisfies WorkspaceCommandDto),
      );
    },

    async replacePaneSpec(paneId, paneSpec) {
      await runWorkspaceMutation(set, () =>
        deps.workspaceClient.dispatch({
          kind: "replacePaneSpec",
          pane_id: paneId,
          pane_spec: mapPaneSpecToDto(paneSpec),
        } satisfies WorkspaceCommandDto),
      );
    },

    async restartPaneRuntime(paneId) {
      set({ isWorking: true });
      try {
        await deps.workspaceClient.dispatch({
          kind: "restartPaneRuntime",
          pane_id: paneId,
        } satisfies WorkspaceCommandDto);
        set({ isWorking: false, error: null });
      } catch (error) {
        set({ error: asErrorMessage(error), isWorking: false });
      }
    },

    async splitPane(paneId, direction, paneSpec) {
      await runWorkspaceMutation(set, () =>
        deps.workspaceClient.dispatch({
          kind: "splitPane",
          pane_id: paneId,
          direction,
          pane_spec: mapPaneSpecToDto(paneSpec),
        } satisfies WorkspaceCommandDto),
      );
    },

    async closePane(paneId) {
      await runWorkspaceMutation(set, () =>
        deps.workspaceClient.dispatch({
          kind: "closePane",
          pane_id: paneId,
        } satisfies WorkspaceCommandDto),
      );
    },

    async swapPanes(paneIdA, paneIdB) {
      await runWorkspaceMutation(set, () =>
        deps.workspaceClient.dispatch({
          kind: "swapPaneSlots",
          pane_id_a: paneIdA,
          pane_id_b: paneIdB,
        } satisfies WorkspaceCommandDto),
      );
    },

    clearError() {
      set({ error: null });
    },
  }));
}
