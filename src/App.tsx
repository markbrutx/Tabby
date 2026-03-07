import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { RecoveryScreen } from "@/components/RecoveryScreen";
import { SplitTreeRenderer } from "@/features/workspace/components/SplitTreeRenderer";
import { TabBar } from "@/features/workspace/components/TabBar";
import { SettingsModal } from "@/features/workspace/components/SettingsModal";
import { ShortcutsModal } from "@/features/workspace/components/ShortcutsModal";
import { SplitPopup } from "@/features/workspace/components/SplitPopup";
import { WorkspaceSetupWizard } from "@/features/workspace/components/WorkspaceSetupWizard";
import { selectActiveTab, selectActivePane } from "@/features/workspace/selectors";
import { useWorkspaceStore } from "@/features/workspace/store/workspaceStore";
import type { SplitDirection } from "@/features/workspace/domain";
import type { SetupWizardConfig, WizardTab } from "@/features/workspace/store/workspaceStore";
import {
  applyResolvedTheme,
  useResolvedTheme,
} from "@/features/workspace/theme";
import { useWorkspaceShortcuts } from "@/features/workspace/useWorkspaceShortcuts";
import { isTauriRuntime } from "@/lib/runtime";

function App() {
  const {
    workspace,
    settings,
    profiles,
    error,
    isHydrating,
    wizardTab,
    initialize,
    createTabFromWizard,
    openSetupWizard,
    closeSetupWizard,
    closeTab,
    setActiveTab,
    focusPane,
    restartPane,
    splitPane,
    closePane,
    updateSettings,
    resetSettings,
    clearError,
  } = useWorkspaceStore();

  const [settingsOpen, setSettingsOpen] = useState(false);
  const [shortcutsOpen, setShortcutsOpen] = useState(false);
  const [splitPopup, setSplitPopup] = useState<{
    paneId: string;
    direction: SplitDirection;
  } | null>(null);
  const resolvedTheme = useResolvedTheme(settings?.theme);

  useEffect(() => {
    void initialize();
  }, [initialize]);

  useEffect(() => {
    applyResolvedTheme(resolvedTheme);
  }, [resolvedTheme]);

  useEffect(() => {
    if (!isTauriRuntime()) return;

    let cancelled = false;
    let unlisten: (() => void) | undefined;

    void listen("menu-open-settings", () => {
      setSettingsOpen(true);
    }).then((fn) => {
      if (cancelled) {
        fn();
      } else {
        unlisten = fn;
      }
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  const handleSplitHorizontal = useCallback(
    (paneId: string) => setSplitPopup({ paneId, direction: "horizontal" }),
    [],
  );

  const handleSplitVertical = useCallback(
    (paneId: string) => setSplitPopup({ paneId, direction: "vertical" }),
    [],
  );

  const handleOpenSettings = useCallback(() => setSettingsOpen(true), []);
  const handleOpenShortcuts = useCallback(() => setShortcutsOpen(true), []);
  const handleCloseShortcuts = useCallback(() => setShortcutsOpen(false), []);

  useWorkspaceShortcuts({
    workspace,
    onCreateTab: openSetupWizard,
    onCloseTab: closeTab,
    onClosePane: closePane,
    onSelectTab: setActiveTab,
    onFocusPane: focusPane,
    onRestartPane: restartPane,
    onSplitHorizontal: handleSplitHorizontal,
    onSplitVertical: handleSplitVertical,
    onOpenSettings: handleOpenSettings,
    onOpenShortcuts: handleOpenShortcuts,
  });

  if (isHydrating) {
    return (
      <div className="flex h-screen items-center justify-center bg-[var(--color-bg)] text-[var(--color-text)]">
        <p className="text-sm text-[var(--color-text-muted)]">Starting...</p>
      </div>
    );
  }

  if (!workspace || !settings) {
    return (
      <RecoveryScreen
        title="Workspace unavailable"
        message={error ?? "Tabby could not bootstrap the workspace."}
        onRetry={() => void initialize()}
      />
    );
  }

  const activeTab = selectActiveTab(workspace);

  // No real tabs and no wizard — recovery screen.
  if (!activeTab && !wizardTab) {
    return (
      <RecoveryScreen
        title="No active workspace"
        message={error ?? "All workspaces have been closed."}
        onRetry={() => void initialize()}
      />
    );
  }

  const activePane = selectActivePane(workspace);

  // Build display tabs: real tabs + phantom wizard tab at the end.
  const displayTabs = wizardTab
    ? [...workspace.tabs, { id: wizardTab.id, title: wizardTab.title }]
    : workspace.tabs;

  // Wizard tab is always active when open; otherwise use real active tab.
  const displayActiveTabId = wizardTab
    ? wizardTab.id
    : workspace.activeTabId;

  const isWizardActive = wizardTab !== null;

  return (
    <div className="flex h-screen flex-col bg-[var(--color-bg)] text-[var(--color-text)]">
      <TabBar
        tabs={displayTabs}
        activeTabId={displayActiveTabId}
        onSelect={(tabId) => {
          if (wizardTab && tabId !== wizardTab.id) {
            // Clicking a real tab while wizard is open — close wizard, switch tab.
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
            void closeTab(tabId);
          }
        }}
        onNewTab={openSetupWizard}
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
        {workspace.tabs.map((tab) => {
          const isActive = tab.id === workspace.activeTabId && !isWizardActive;

          return (
            <div key={tab.id} className={`h-full ${isActive ? "block" : "hidden"}`}>
              <SplitTreeRenderer
                tab={tab}
                fontSize={settings.fontSize}
                theme={resolvedTheme}
                visible={isActive}
                onFocus={focusPane}
                onRestart={restartPane}
              />
            </div>
          );
        })}

        {isWizardActive ? (
          <WorkspaceSetupWizard
            profiles={profiles}
            settings={settings}
            isFirstLaunch={workspace.tabs.length === 0 && !settings.hasCompletedOnboarding}
            onComplete={(config: SetupWizardConfig) => {
              void createTabFromWizard(config);
            }}
            onCancel={workspace.tabs.length > 0 ? closeSetupWizard : undefined}
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
          defaultProfileId={activePane.profileId}
          defaultCwd={activePane.cwd}
          onConfirm={(profileId, cwd, startupCommand) => {
            void splitPane({
              paneId: splitPopup.paneId,
              direction: splitPopup.direction,
              profileId,
              cwd,
              startupCommand,
            });
            setSplitPopup(null);
          }}
          onCancel={() => setSplitPopup(null)}
        />
      ) : null}
    </div>
  );
}

export default App;
