import { RecoveryScreen } from "@/components/RecoveryScreen";
import { TitleBarDragRegion } from "@/components/TitleBarDragRegion";
import { ConfirmDialog } from "@/features/workspace/components/ConfirmDialog";
import { SplitTreeRenderer } from "@/features/workspace/components/SplitTreeRenderer";
import { TabBar } from "@/features/workspace/components/TabBar";
import { SettingsModal } from "@/features/settings/components/SettingsModal";
import { ShortcutsModal } from "@/features/settings/components/ShortcutsModal";
import { SplitPopup } from "@/features/workspace/components/SplitPopup";
import { WorkspaceSetupWizard } from "@/features/workspace/components/WorkspaceSetupWizard";
import { selectActivePane, selectActiveTab } from "@/features/workspace/selectors";
import { shellClients } from "@/app-shell/clients";
import type { SetupWizardConfig } from "@/features/workspace/store/types";
import type { AppOrchestration } from "@/features/workspace/hooks/useAppOrchestration";
import type { AppCallbacks } from "@/features/workspace/hooks/useAppCallbacks";

interface AppLayoutProps {
  readonly orchestration: AppOrchestration;
  readonly callbacks: AppCallbacks;
}

export function AppLayout({ orchestration, callbacks }: AppLayoutProps) {
  const {
    workspaceModel,
    error,
    wizardTab,
    settings,
    profiles,
    resolvedTheme,
    collapseStore,
    confirmDialog,
    createTabFromWizard,
    openSetupWizard,
    closeSetupWizard,
    setActiveTab,
    focusPane,
    restartPaneRuntime,
    splitPane,
    renameTab,
    clearError,
    updateSettings,
    resetSettings,
  } = orchestration;

  const {
    settingsOpen,
    shortcutsOpen,
    splitPopup,
    handleOpenSettings,
    handleOpenShortcuts,
    handleCloseShortcuts,
    handleCloseSettings,
    handleSwapPaneSlots,
    handleOpenGitView,
    handleToggleCollapse,
    setSplitPopup,
  } = callbacks;

  // These are guaranteed non-null by the time AppLayout renders (App.tsx guards)
  if (!workspaceModel || !settings) return null;

  const activeTab = selectActiveTab(workspaceModel);
  const activePane = selectActivePane(workspaceModel);
  const modalOpen =
    splitPopup !== null ||
    confirmDialog.action !== null ||
    settingsOpen ||
    shortcutsOpen;

  const isWizardActive = wizardTab !== null;

  const displayTabs = wizardTab
    ? [...workspaceModel.tabs, { id: wizardTab.id, title: wizardTab.title, isWizard: true }]
    : workspaceModel.tabs;

  const displayActiveTabId = wizardTab
    ? wizardTab.id
    : workspaceModel.activeTabId;

  if (!activeTab && !wizardTab) {
    return (
      <div className="flex h-screen flex-col bg-[var(--color-bg)]">
        <TitleBarDragRegion />
        <RecoveryScreen
          title="No active workspace"
          message={error ?? "All workspaces have been closed."}
          onRetry={() => window.location.reload()}
        />
      </div>
    );
  }

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
          onClose={handleCloseSettings}
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
            const tab = workspaceModel ? selectActiveTab(workspaceModel) : null;
            if (tab) {
              collapseStore.expandPane(tab.id, splitPopup.paneId);
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
