import { useCallback, useMemo } from "react";
import { useShallow } from "zustand/react/shallow";
import { useWorkspaceStore, useSettingsStore, useRuntimeStore } from "@/contexts/stores";
import { useConfirmAction } from "@/features/workspace/hooks/useConfirmAction";
import { useWizardState } from "@/features/workspace/hooks/useWizardState";
import { selectActiveTab } from "@/features/workspace/selectors";
import { useResolvedTheme } from "@/features/workspace/theme";
import { useThemeStore } from "@/features/theme/application/themeStore";
import { buildWorkspaceSnapshotModel } from "@/features/workspace/model/workspaceSnapshot";

export function useAppOrchestration() {
  const {
    workspace,
    error,
    isHydrating,
    createTabFromWizard,
    closeTab,
    setActiveTab,
    focusPane,
    restartPaneRuntime,
    splitPane,
    closePane,
    swapPaneSlots,
    renameTab,
    clearError,
  } = useWorkspaceStore(
    useShallow((state) => ({
      workspace: state.workspace,
      error: state.error,
      isHydrating: state.isHydrating,
      createTabFromWizard: state.createTabFromWizard,
      closeTab: state.closeTab,
      setActiveTab: state.setActiveTab,
      focusPane: state.focusPane,
      restartPaneRuntime: state.restartPaneRuntime,
      splitPane: state.splitPane,
      closePane: state.closePane,
      swapPaneSlots: state.swapPaneSlots,
      renameTab: state.renameTab,
      clearError: state.clearError,
    })),
  );

  const { wizardTab, openSetupWizard, closeSetupWizard } = useWizardState(workspace);

  const runtimes = useRuntimeStore((state) => state.runtimes);

  const {
    settings,
    profiles,
    updateSettings,
    resetSettings,
  } = useSettingsStore(
    useShallow((state) => ({
      settings: state.settings,
      profiles: state.profiles,
      updateSettings: state.updateSettings,
      resetSettings: state.resetSettings,
    })),
  );

  const workspaceModel = useMemo(
    () => buildWorkspaceSnapshotModel(workspace, runtimes, profiles),
    [workspace, runtimes, profiles],
  );

  const initializeThemes = useThemeStore((s) => s.initialize);
  const resolvedTheme = useResolvedTheme(settings?.theme);

  const closePaneWithCleanup = useCallback(
    async (paneId: string) => {
      await closePane(paneId);
    },
    [closePane],
  );

  const closeTabWithCleanup = useCallback(
    async (tabId: string) => {
      await closeTab(tabId);
    },
    [closeTab],
  );

  const confirmDialog = useConfirmAction({
    workspace: workspaceModel ?? { tabs: [] },
    closePane: closePaneWithCleanup,
    closeTab: closeTabWithCleanup,
  });

  return {
    workspace,
    workspaceModel,
    error,
    isHydrating,
    wizardTab,
    settings,
    profiles,
    resolvedTheme,
    confirmDialog,
    initializeThemes,
    createTabFromWizard,
    openSetupWizard,
    closeSetupWizard,
    setActiveTab,
    focusPane,
    restartPaneRuntime,
    splitPane,
    swapPaneSlots,
    renameTab,
    clearError,
    updateSettings,
    resetSettings,
  };
}

export type AppOrchestration = ReturnType<typeof useAppOrchestration>;
