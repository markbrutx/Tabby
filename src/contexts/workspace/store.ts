import { create } from "zustand";
import { asErrorMessage, shellClients } from "@/app-shell/clients";
import {
  BROWSER_PROFILE_ID,
  CUSTOM_PROFILE_ID,
  type LayoutPreset,
  type PaneSpecDto,
  type SplitDirection,
  type WorkspaceCommandDto,
  type WorkspaceView,
} from "@/features/workspace/domain";
import { useRuntimeStore } from "@/contexts/runtime/store";
import { useSettingsStore } from "@/contexts/settings/store";
import type { SetupWizardConfig, WizardTab } from "@/features/workspace/store/types";

interface WorkspaceStore {
  workspace: WorkspaceView | null;
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
  replacePaneSpec: (paneId: string, paneSpec: PaneSpecDto) => Promise<void>;
  restartPaneRuntime: (paneId: string) => Promise<void>;
  splitPane: (
    paneId: string,
    direction: SplitDirection,
    paneSpec: PaneSpecDto,
  ) => Promise<void>;
  closePane: (paneId: string) => Promise<void>;
  swapPanes: (paneIdA: string, paneIdB: string) => Promise<void>;
  trackTerminalWorkingDirectory: (paneId: string, workingDirectory: string) => Promise<void>;
  clearError: () => void;
}

type SetFn = (
  partial:
    | Partial<WorkspaceStore>
    | ((state: WorkspaceStore) => Partial<WorkspaceStore>),
) => void;

let workspaceListenersReady: Promise<void> | null = null;

function makeWizardTab(workspace?: WorkspaceView | null): WizardTab {
  const nextIndex = (workspace?.tabs.length ?? 0) + 1;
  return {
    id: `__wizard_${Date.now()}__`,
    title: `Workspace ${nextIndex}`,
  };
}

async function runWorkspaceMutation(
  set: SetFn,
  mutation: () => Promise<WorkspaceView>,
  onSuccess?: (workspace: WorkspaceView) => Partial<WorkspaceStore>,
) {
  set({ isWorking: true });
  try {
    const workspace = await mutation();
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

function createWorkspaceStoreState() {
  return (set: SetFn, get: () => WorkspaceStore): WorkspaceStore => ({
    workspace: null,
    error: null,
    isHydrating: true,
    isWorking: false,
    wizardTab: null,

    async initialize() {
      set({ isHydrating: true, error: null });

      try {
        const payload = await shellClients.workspace.bootstrap();
        useSettingsStore.getState().loadBootstrap(
          payload.settings,
          payload.profileCatalog.terminalProfiles,
        );
        useRuntimeStore.getState().loadBootstrap(payload.runtimeProjections);

        if (!workspaceListenersReady) {
          workspaceListenersReady = shellClients.workspace
            .listenProjectionUpdated((workspace) => {
              set({
                workspace,
                error: null,
                wizardTab: workspace.tabs.length === 0 ? makeWizardTab(workspace) : null,
              });
            })
            .then(() => undefined);
        }
        await workspaceListenersReady;

        const shouldShowWizard = payload.workspace.tabs.length === 0;
        set({
          workspace: payload.workspace,
          error: null,
          isHydrating: false,
          wizardTab: shouldShowWizard ? makeWizardTab(payload.workspace) : null,
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
          shellClients.workspace.dispatch({
            kind: "openTab",
            layout: null,
            auto_layout: true,
            pane_specs: config.groups.flatMap(toPaneSpec),
          } satisfies WorkspaceCommandDto),
        () => {
          const settings = useSettingsStore.getState().settings;
          if (settings && !settings.hasCompletedOnboarding) {
            void useSettingsStore
              .getState()
              .updateSettings({ ...settings, hasCompletedOnboarding: true });
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
        shellClients.workspace.dispatch({
          kind: "closeTab",
          tab_id: tabId,
        } satisfies WorkspaceCommandDto),
      );
    },

    async setActiveTab(tabId) {
      await runWorkspaceMutation(set, () =>
        shellClients.workspace.dispatch({
          kind: "setActiveTab",
          tab_id: tabId,
        } satisfies WorkspaceCommandDto),
      );
    },

    async focusPane(tabId, paneId) {
      await runWorkspaceMutation(set, () =>
        shellClients.workspace.dispatch({
          kind: "focusPane",
          tab_id: tabId,
          pane_id: paneId,
        } satisfies WorkspaceCommandDto),
      );
    },

    async replacePaneSpec(paneId, paneSpec) {
      await runWorkspaceMutation(set, () =>
        shellClients.workspace.dispatch({
          kind: "replacePaneSpec",
          pane_id: paneId,
          pane_spec: paneSpec,
        } satisfies WorkspaceCommandDto),
      );
    },

    async restartPaneRuntime(paneId) {
      await runWorkspaceMutation(set, async () => {
        await shellClients.workspace.dispatch({
          kind: "restartPaneRuntime",
          pane_id: paneId,
        } satisfies WorkspaceCommandDto);
        return get().workspace ?? { activeTabId: "", tabs: [] };
      });
    },

    async splitPane(paneId, direction, paneSpec) {
      await runWorkspaceMutation(set, () =>
        shellClients.workspace.dispatch({
          kind: "splitPane",
          pane_id: paneId,
          direction,
          pane_spec: paneSpec,
        } satisfies WorkspaceCommandDto),
      );
    },

    async closePane(paneId) {
      await runWorkspaceMutation(set, () =>
        shellClients.workspace.dispatch({
          kind: "closePane",
          pane_id: paneId,
        } satisfies WorkspaceCommandDto),
      );
    },

    async swapPanes(paneIdA, paneIdB) {
      await runWorkspaceMutation(set, () =>
        shellClients.workspace.dispatch({
          kind: "swapPaneSlots",
          pane_id_a: paneIdA,
          pane_id_b: paneIdB,
        } satisfies WorkspaceCommandDto),
      );
    },

    async trackTerminalWorkingDirectory(paneId, workingDirectory) {
      await shellClients.workspace.dispatch({
        kind: "trackTerminalWorkingDirectory",
        pane_id: paneId,
        working_directory: workingDirectory,
      } satisfies WorkspaceCommandDto);
    },

    clearError() {
      set({ error: null });
    },
  });
}

export const useWorkspaceStore = create<WorkspaceStore>(createWorkspaceStoreState());
