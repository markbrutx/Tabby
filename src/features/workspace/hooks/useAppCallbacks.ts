import { useCallback, useState } from "react";
import { selectActiveTab } from "@/features/workspace/selectors";
import { useTauriMenuEvents } from "@/features/workspace/hooks/useTauriMenuEvents";
import { useWorkspaceShortcuts } from "@/features/workspace/useWorkspaceShortcuts";
import type { AppOrchestration } from "./useAppOrchestration";

export function useAppCallbacks(orchestration: AppOrchestration) {
  const {
    workspaceModel,
    settings,
    confirmDialog,
    openSetupWizard,
    setActiveTab,
    focusPane,
    restartPaneRuntime,
    splitPane,
    swapPaneSlots,
    updateSettings,
  } = orchestration;

  const [settingsOpen, setSettingsOpen] = useState(false);
  const [shortcutsOpen, setShortcutsOpen] = useState(false);
  const [splitPopup, setSplitPopup] = useState<{
    paneId: string;
    direction: "horizontal" | "vertical";
  } | null>(null);

  const handleOpenSettings = useCallback(() => setSettingsOpen(true), []);
  const handleOpenShortcuts = useCallback(() => setShortcutsOpen(true), []);
  const handleCloseShortcuts = useCallback(() => setShortcutsOpen(false), []);
  const handleCloseSettings = useCallback(() => setSettingsOpen(false), []);

  useTauriMenuEvents(handleOpenSettings);

  const handleSplitRight = useCallback(
    (paneId: string) => setSplitPopup({ paneId, direction: "horizontal" }),
    [],
  );

  const handleSplitDown = useCallback(
    (paneId: string) => setSplitPopup({ paneId, direction: "vertical" }),
    [],
  );

  const handleSwapPaneSlots = useCallback(
    (paneIdA: string, paneIdB: string) => {
      void swapPaneSlots(paneIdA, paneIdB);
    },
    [swapPaneSlots],
  );

  const handleOpenGitView = useCallback(
    (paneId: string, cwd: string) => {
      void splitPane(paneId, "horizontal", {
        kind: "git",
        workingDirectory: cwd,
      });
    },
    [splitPane],
  );

  const handleZoomIn = useCallback(() => {
    if (!settings) return;
    const next = Math.min(settings.fontSize + 1, 20);
    void updateSettings({ ...settings, fontSize: next });
  }, [settings, updateSettings]);

  const handleZoomOut = useCallback(() => {
    if (!settings) return;
    const next = Math.max(settings.fontSize - 1, 11);
    void updateSettings({ ...settings, fontSize: next });
  }, [settings, updateSettings]);

  const handleZoomReset = useCallback(() => {
    if (!settings) return;
    void updateSettings({ ...settings, fontSize: 14 });
  }, [settings, updateSettings]);

  useWorkspaceShortcuts({
    workspace: workspaceModel,
    onCreateTab: openSetupWizard,
    onCloseTab: confirmDialog.requestCloseTab,
    onClosePane: confirmDialog.requestClosePane,
    onSelectTab: setActiveTab,
    onFocusPane: focusPane,
    onRestartPane: restartPaneRuntime,
    onSplitRight: handleSplitRight,
    onSplitDown: handleSplitDown,
    onOpenSettings: handleOpenSettings,
    onOpenShortcuts: handleOpenShortcuts,
    onZoomIn: handleZoomIn,
    onZoomOut: handleZoomOut,
    onZoomReset: handleZoomReset,
  });

  return {
    settingsOpen,
    shortcutsOpen,
    splitPopup,
    handleOpenSettings,
    handleOpenShortcuts,
    handleCloseShortcuts,
    handleCloseSettings,
    handleSwapPaneSlots,
    handleOpenGitView,
    handleSplitRight,
    handleSplitDown,
    setSplitPopup,
  };
}

export type AppCallbacks = ReturnType<typeof useAppCallbacks>;
