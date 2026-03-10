import { create } from "zustand";
import { asErrorMessage } from "@/app-shell/clients";
import type {
  PaneSpecDto,
  WorkspaceCommandDto,
  WorkspaceView,
} from "@/contracts/tauri-bindings";
import {
  CUSTOM_PROFILE_ID,
  DEFAULT_BROWSER_URL,
} from "@/features/workspace/domain/models";
import type {
  PaneSpec,
  SplitDirection,
  WorkspaceReadModel,
} from "@/features/workspace/domain/models";
import { mapPaneSpecToDto, mapSplitNodeToDto, mapWorkspaceFromDto } from "@/features/workspace/application/snapshot-mappers";
import { getLayoutVariants } from "@/features/workspace/layoutVariants";
import type { WorkspaceClient } from "@/app-shell/clients";
import type { SetupWizardConfig } from "@/features/workspace/store/types";

export interface WorkspaceStore {
  workspace: WorkspaceReadModel | null;
  error: string | null;
  isHydrating: boolean;
  isWorking: boolean;
  beginBootstrap: () => void;
  loadBootstrap: (workspace: WorkspaceReadModel) => Promise<void>;
  setBootstrapError: (message: string) => void;
  createTabFromWizard: (config: SetupWizardConfig) => Promise<void>;
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
  swapPaneSlots: (paneIdA: string, paneIdB: string) => Promise<void>;
  renameTab: (tabId: string, title: string) => Promise<void>;
  clearError: () => void;
}

type SetFn = (
  partial:
    | Partial<WorkspaceStore>
    | ((state: WorkspaceStore) => Partial<WorkspaceStore>),
) => void;

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
      ...(onSuccess?.(workspace) ?? {}),
    });
  } catch (error) {
    set({ error: asErrorMessage(error), isWorking: false });
  }
}

function toPaneSpec(group: SetupWizardConfig["groups"][number]): PaneSpecDto[] {
  return Array.from({ length: group.count }, (): PaneSpecDto => {
    switch (group.mode) {
      case "browser":
        return { kind: "browser", initial_url: group.url || DEFAULT_BROWSER_URL };
      case "git":
        return { kind: "git", working_directory: group.workingDirectory || "~" };
      case "terminal":
        return {
          kind: "terminal",
          launch_profile_id: group.profileId || "terminal",
          working_directory: group.workingDirectory || "~",
          command_override:
            group.profileId === CUSTOM_PROFILE_ID ? group.customCommand.trim() || null : null,
        };
    }
  });
}

export interface WorkspaceStoreDeps {
  workspaceClient: WorkspaceClient;
  onWizardComplete: () => void;
}

export function createWorkspaceStore(deps: WorkspaceStoreDeps) {
  let workspaceListenersReady: Promise<void> | null = null;

  return create<WorkspaceStore>((set, get) => ({
    workspace: null,
    error: null,
    isHydrating: true,
    isWorking: false,

    beginBootstrap() {
      set({ isHydrating: true, error: null });
    },

    async loadBootstrap(workspace) {
      if (!workspaceListenersReady) {
        workspaceListenersReady = deps.workspaceClient
          .listenProjectionUpdated((dto) => {
            const mapped = mapWorkspaceFromDto(dto);
            set({ workspace: mapped, error: null });
          })
          .then(() => undefined);
      }
      await workspaceListenersReady;

      set({ workspace, error: null, isHydrating: false });
    },

    setBootstrapError(message) {
      set({ error: message, isHydrating: false });
    },

    async createTabFromWizard(config) {
      const customTitle = config.title;

      const paneSpecs = config.groups.flatMap(toPaneSpec);
      const totalPanes = paneSpecs.length;
      const variants = getLayoutVariants(totalPanes);
      const selectedVariant = config.layoutVariantId
        ? variants.find((v) => v.id === config.layoutVariantId)
        : null;

      let layoutTree = null;
      let autoLayout = true;
      if (selectedVariant) {
        const dummyIds = Array.from({ length: totalPanes }, (_, i) => `p${i}`);
        layoutTree = mapSplitNodeToDto(selectedVariant.build(dummyIds));
        autoLayout = false;
      }

      await runWorkspaceMutation(
        set,
        () =>
          deps.workspaceClient.dispatch({
            kind: "openTab",
            layout: null,
            auto_layout: autoLayout,
            layout_tree: layoutTree,
            pane_specs: paneSpecs,
          } satisfies WorkspaceCommandDto),
        (workspace) => {
          deps.onWizardComplete();

          if (customTitle && workspace.activeTabId) {
            const activeTab = workspace.tabs.find((t) => t.tabId === workspace.activeTabId);
            if (activeTab && activeTab.title !== customTitle) {
              void deps.workspaceClient.dispatch({
                kind: "renameTab",
                tab_id: workspace.activeTabId,
                title: customTitle,
              } satisfies WorkspaceCommandDto);
            }
          }

          return {};
        },
      );
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

    async swapPaneSlots(paneIdA, paneIdB) {
      await runWorkspaceMutation(set, () =>
        deps.workspaceClient.dispatch({
          kind: "swapPaneSlots",
          pane_id_a: paneIdA,
          pane_id_b: paneIdB,
        } satisfies WorkspaceCommandDto),
      );
    },

    async renameTab(tabId, title) {
      await runWorkspaceMutation(set, () =>
        deps.workspaceClient.dispatch({
          kind: "renameTab",
          tab_id: tabId,
          title,
        } satisfies WorkspaceCommandDto),
      );
    },

    clearError() {
      set({ error: null });
    },
  }));
}
