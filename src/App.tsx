import { useCallback, useEffect, useMemo, useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { RecoveryScreen } from "@/components/RecoveryScreen";
import { ConfirmDialog } from "@/features/workspace/components/ConfirmDialog";
import { SplitTreeRenderer } from "@/features/workspace/components/SplitTreeRenderer";
import { TabBar } from "@/features/workspace/components/TabBar";
import { SettingsModal } from "@/features/settings/components/SettingsModal";
import { ShortcutsModal } from "@/features/settings/components/ShortcutsModal";
import { SplitPopup } from "@/features/workspace/components/SplitPopup";
import { WorkspaceSetupWizard } from "@/features/workspace/components/WorkspaceSetupWizard";
import { useConfirmAction } from "@/features/workspace/hooks/useConfirmAction";
import { useTauriMenuEvents } from "@/features/workspace/hooks/useTauriMenuEvents";
import { selectActivePane, selectActiveTab } from "@/features/workspace/selectors";
import { shellClients } from "@/app-shell/clients";
import { useWorkspaceStore, useSettingsStore, useRuntimeStore, bootstrapCoordinator } from "@/contexts/stores";
import type { SetupWizardConfig } from "@/features/workspace/store/types";
import { applyResolvedTheme, useResolvedTheme } from "@/features/workspace/theme";
import { useWorkspaceShortcuts } from "@/features/workspace/useWorkspaceShortcuts";
import { buildWorkspaceSnapshotModel } from "@/features/workspace/model/workspaceSnapshot";

function App() {
  const {
    workspace,
    error,
    isHydrating,
    wizardTab,
    createTabFromWizard,
    openSetupWizard,
    closeSetupWizard,
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
      wizardTab: state.wizardTab,
      createTabFromWizard: state.createTabFromWizard,
      openSetupWizard: state.openSetupWizard,
      closeSetupWizard: state.closeSetupWizard,
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

  const [settingsOpen, setSettingsOpen] = useState(false);
  const [shortcutsOpen, setShortcutsOpen] = useState(false);
  const [splitPopup, setSplitPopup] = useState<{
    paneId: string;
    direction: "horizontal" | "vertical";
  } | null>(null);

  const resolvedTheme = useResolvedTheme(settings?.theme);

  const confirmDialog = useConfirmAction({
    workspace: workspaceModel ?? { tabs: [] },
    closePane,
    closeTab,
  });

  useEffect(() => {
    void bootstrapCoordinator.initialize();
  }, []);

  useEffect(() => {
    applyResolvedTheme(resolvedTheme);
  }, [resolvedTheme]);

  useEffect(() => {
    if (settings?.fontSize) {
      document.documentElement.style.setProperty(
        "--ui-font-size",
        `${settings.fontSize}px`,
      );
    }
  }, [settings?.fontSize]);

  const handleOpenSettings = useCallback(() => setSettingsOpen(true), []);
  const handleOpenShortcuts = useCallback(() => setShortcutsOpen(true), []);
  const handleCloseShortcuts = useCallback(() => setShortcutsOpen(false), []);

  useTauriMenuEvents(handleOpenSettings, confirmDialog.requestQuitApp);

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

  if (isHydrating) {
    return (
      <div className="flex h-screen items-center justify-center bg-[var(--color-bg)] text-[var(--color-text)]">
        <p className="text-sm text-[var(--color-text-muted)]">Starting...</p>
      </div>
    );
  }

  if (!workspaceModel || !settings) {
    return (
      <RecoveryScreen
        title="Workspace unavailable"
        message={error ?? "Tabby could not bootstrap the workspace."}
        onRetry={() => void bootstrapCoordinator.initialize()}
      />
    );
  }

  const activeTab = selectActiveTab(workspaceModel);

  if (!activeTab && !wizardTab) {
    return (
      <RecoveryScreen
        title="No active workspace"
        message={error ?? "All workspaces have been closed."}
        onRetry={() => void bootstrapCoordinator.initialize()}
      />
    );
  }

  const activePane = selectActivePane(workspaceModel);
  const modalOpen =
    splitPopup !== null ||
    confirmDialog.action !== null ||
    settingsOpen ||
    shortcutsOpen;

  const displayTabs = wizardTab
    ? [...workspaceModel.tabs, { id: wizardTab.id, title: wizardTab.title }]
    : workspaceModel.tabs;

  const displayActiveTabId = wizardTab
    ? wizardTab.id
    : workspaceModel.activeTabId;

  const isWizardActive = wizardTab !== null;

  return (
    <div className="flex h-screen flex-col bg-[var(--color-bg)] text-[var(--color-text)]">
      <TabBar
        tabs={displayTabs}
        activeTabId={displayActiveTabId}
        onSelect={(tabId) => {
          if (wizardTab && tabId !== wizardTab.id) {
            closeSetupWizard();
            void setActiveTab(tabId);
          } else if (!wizardTab) {
            void setActiveTab(tabId);
          }
        }}
        onClose={(tabId) => {
          if (wizardTab && tabId === wizardTab.id) {
            closeSetupWizard();
          } else {
            confirmDialog.requestCloseTab(tabId);
          }
        }}
        onRename={(tabId, title) => void renameTab(tabId, title)}
        onNewTab={openSetupWizard}
        showNewTab={!isWizardActive}
        onOpenSettings={handleOpenSettings}
        onOpenShortcuts={handleOpenShortcuts}
      />

      {error ? (
        <div className="flex items-center justify-between gap-4 border-b border-[var(--color-border)] bg-[var(--color-danger-soft)] px-4 py-1.5 text-xs">
          <span>{error}</span>
          <button
            className="text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
            onClick={clearError}
          >
            dismiss
          </button>
        </div>
      ) : null}

      <div className="min-h-0 flex-1">
        {workspaceModel.tabs.map((tab) => {
          const isActive = tab.id === workspaceModel.activeTabId && !isWizardActive;

          return (
            <div key={tab.id} className={`h-full ${isActive ? "block" : "hidden"}`}>
              <SplitTreeRenderer
                tab={tab}
                theme={resolvedTheme}
                visible={isActive}
                modalOpen={modalOpen}
                gitClient={shellClients.git}
                onFocus={focusPane}
                onRestart={restartPaneRuntime}
                onClosePane={confirmDialog.requestClosePane}
                onSwapPaneSlots={handleSwapPaneSlots}
                onOpenGitView={handleOpenGitView}
              />
            </div>
          );
        })}

        {isWizardActive ? (
          <WorkspaceSetupWizard
            profiles={profiles}
            settings={settings}
            isFirstLaunch={
              workspaceModel.tabs.length === 0 && !settings.hasCompletedOnboarding
            }
            onComplete={(config: SetupWizardConfig) => {
              void createTabFromWizard(config);
            }}
            onCancel={workspaceModel.tabs.length > 0 ? closeSetupWizard : undefined}
          />
        ) : null}
      </div>

      {settingsOpen ? (
        <SettingsModal
          settings={settings}
          profiles={profiles}
          onClose={() => setSettingsOpen(false)}
          onSave={updateSettings}
          onReset={resetSettings}
        />
      ) : null}

      {shortcutsOpen ? (
        <ShortcutsModal onClose={handleCloseShortcuts} />
      ) : null}

      {splitPopup && activePane ? (
        <SplitPopup
          direction={splitPopup.direction}
          profiles={profiles}
          defaultSpec={activePane.spec}
          onConfirm={(paneSpec) => {
            void splitPane(splitPopup.paneId, splitPopup.direction, paneSpec);
            setSplitPopup(null);
          }}
          onCancel={() => setSplitPopup(null)}
        />
      ) : null}

      {confirmDialog.message ? (
        <ConfirmDialog
          {...confirmDialog.message}
          onConfirm={confirmDialog.confirm}
          onCancel={confirmDialog.cancel}
        />
      ) : null}
    </div>
  );
}

export default App;
