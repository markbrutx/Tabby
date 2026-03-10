import { useCallback, useEffect, useMemo, useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { RecoveryScreen } from "@/components/RecoveryScreen";
import { TitleBarDragRegion } from "@/components/TitleBarDragRegion";
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
import { useResolvedTheme } from "@/features/workspace/theme";
import { applyTheme } from "@/features/theme/application/themeApplicator";
import { useThemeStore } from "@/features/theme/application/themeStore";
import { useWorkspaceShortcuts } from "@/features/workspace/useWorkspaceShortcuts";
import { buildWorkspaceSnapshotModel } from "@/features/workspace/model/workspaceSnapshot";
import { useCollapseStore } from "@/features/workspace/application/collapseStore";
import { collectPaneIds, findNextPane } from "@/features/workspace/layoutReadModel";

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

  const collapseStore = useCollapseStore();
  const initializeThemes = useThemeStore((s) => s.initialize);

  const [settingsOpen, setSettingsOpen] = useState(false);
  const [shortcutsOpen, setShortcutsOpen] = useState(false);
  const [splitPopup, setSplitPopup] = useState<{
    paneId: string;
    direction: "horizontal" | "vertical";
  } | null>(null);

  const resolvedTheme = useResolvedTheme(settings?.theme);

  const closePaneWithCleanup = useCallback(
    async (paneId: string) => {
      const activeTab = workspaceModel ? selectActiveTab(workspaceModel) : null;
      if (activeTab) {
        collapseStore.cleanupPane(activeTab.id, paneId);
      }
      await closePane(paneId);
    },
    [closePane, workspaceModel, collapseStore],
  );

  const closeTabWithCleanup = useCallback(
    async (tabId: string) => {
      collapseStore.cleanupTab(tabId);
      await closeTab(tabId);
    },
    [closeTab, collapseStore],
  );

  const confirmDialog = useConfirmAction({
    workspace: workspaceModel ?? { tabs: [] },
    closePane: closePaneWithCleanup,
    closeTab: closeTabWithCleanup,
  });

  useEffect(() => {
    void bootstrapCoordinator.initialize();
  }, []);

  useEffect(() => {
    initializeThemes();
  }, [initializeThemes]);

  useEffect(() => {
    applyTheme(resolvedTheme);
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
      const activeTab = workspaceModel ? selectActiveTab(workspaceModel) : null;
      if (activeTab) {
        collapseStore.expandPane(activeTab.id, paneId);
      }
      void splitPane(paneId, "horizontal", {
        kind: "git",
        workingDirectory: cwd,
      });
    },
    [splitPane, workspaceModel, collapseStore],
  );

  const handleToggleCollapse = useCallback(
    (paneId: string) => {
      if (!workspaceModel) return;
      const activeTab = selectActiveTab(workspaceModel);
      if (!activeTab) return;

      const allPaneIds = collectPaneIds(activeTab.layout);
      const wasCollapsed = collapseStore.isCollapsed(activeTab.id, paneId);
      const didCollapse = collapseStore.toggleCollapse(activeTab.id, paneId, allPaneIds);

      // If we just collapsed the active pane, focus the next expanded pane
      if (didCollapse && !wasCollapsed && activeTab.activePaneId === paneId) {
        const collapsedSet = collapseStore.getCollapsedSet(activeTab.id);
        const expandedIds = allPaneIds.filter((id) => !collapsedSet.has(id));
        if (expandedIds.length > 0) {
          // Find next pane in DFS order that is expanded
          let candidate = findNextPane(activeTab.layout, paneId);
          let attempts = allPaneIds.length;
          while (candidate && collapsedSet.has(candidate) && attempts > 0) {
            candidate = findNextPane(activeTab.layout, candidate);
            attempts--;
          }
          if (candidate && !collapsedSet.has(candidate)) {
            void focusPane(activeTab.id, candidate);
          }
        }
      }
    },
    [workspaceModel, collapseStore, focusPane],
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
    onToggleCollapsePane: handleToggleCollapse,
  });

  if (isHydrating) {
    return (
      <div className="flex h-screen flex-col bg-[var(--color-bg)] text-[var(--color-text)]">
        <TitleBarDragRegion />
        <div className="flex flex-1 items-center justify-center">
          <p className="text-sm text-[var(--color-text-muted)]">Starting...</p>
        </div>
      </div>
    );
  }

  if (!workspaceModel || !settings) {
    return (
      <div className="flex h-screen flex-col bg-[var(--color-bg)]">
        <TitleBarDragRegion />
        <RecoveryScreen
          title="Workspace unavailable"
          message={error ?? "Tabby could not bootstrap the workspace."}
          onRetry={() => void bootstrapCoordinator.initialize()}
        />
      </div>
    );
  }

  const activeTab = selectActiveTab(workspaceModel);

  if (!activeTab && !wizardTab) {
    return (
      <div className="flex h-screen flex-col bg-[var(--color-bg)]">
        <TitleBarDragRegion />
        <RecoveryScreen
          title="No active workspace"
          message={error ?? "All workspaces have been closed."}
          onRetry={() => void bootstrapCoordinator.initialize()}
        />
      </div>
    );
  }

  const activePane = selectActivePane(workspaceModel);
  const modalOpen =
    splitPopup !== null ||
    confirmDialog.action !== null ||
    settingsOpen ||
    shortcutsOpen;

  const displayTabs = wizardTab
    ? [...workspaceModel.tabs, { id: wizardTab.id, title: wizardTab.title, isWizard: true }]
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
                collapsedPaneIds={collapseStore.getCollapsedSet(tab.id)}
                onFocus={focusPane}
                onRestart={restartPaneRuntime}
                onClosePane={confirmDialog.requestClosePane}
                onSwapPaneSlots={handleSwapPaneSlots}
                onOpenGitView={handleOpenGitView}
                onToggleCollapse={handleToggleCollapse}
              />
            </div>
          );
        })}

        {isWizardActive ? (
          <WorkspaceSetupWizard
            profiles={profiles}
            settings={settings}
            defaultTitle={`Workspace ${workspaceModel.tabs.length + 1}`}
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
            const activeTab = workspaceModel ? selectActiveTab(workspaceModel) : null;
            if (activeTab) {
              collapseStore.expandPane(activeTab.id, splitPopup.paneId);
            }
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
