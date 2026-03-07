import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { RecoveryScreen } from "@/components/RecoveryScreen";
import { SplitTreeRenderer } from "@/features/workspace/components/SplitTreeRenderer";
import { TabBar } from "@/features/workspace/components/TabBar";
import { SettingsModal } from "@/features/workspace/components/SettingsModal";
import { SplitPopup } from "@/features/workspace/components/SplitPopup";
import { selectActiveTab, selectActivePane } from "@/features/workspace/selectors";
import { useWorkspaceStore } from "@/features/workspace/store/workspaceStore";
import type { SplitDirection } from "@/features/workspace/domain";
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
    initialize,
    createTab,
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

  const handleCreateTab = useCallback(async () => {
    if (!settings) return;
    await createTab(settings.defaultLayout);
  }, [settings, createTab]);

  const handleSplitHorizontal = useCallback(
    (paneId: string) => setSplitPopup({ paneId, direction: "horizontal" }),
    [],
  );

  const handleSplitVertical = useCallback(
    (paneId: string) => setSplitPopup({ paneId, direction: "vertical" }),
    [],
  );

  const handleOpenSettings = useCallback(() => setSettingsOpen(true), []);

  useWorkspaceShortcuts({
    workspace,
    onCreateTab: handleCreateTab,
    onCloseTab: closeTab,
    onClosePane: closePane,
    onSelectTab: setActiveTab,
    onFocusPane: focusPane,
    onRestartPane: restartPane,
    onSplitHorizontal: handleSplitHorizontal,
    onSplitVertical: handleSplitVertical,
    onOpenSettings: handleOpenSettings,
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

  if (!activeTab) {
    return (
      <RecoveryScreen
        title="No active workspace"
        message={error ?? "All workspaces have been closed."}
        onRetry={() => void initialize()}
      />
    );
  }

  const activePane = selectActivePane(workspace);

  return (
    <div className="flex h-screen flex-col bg-[var(--color-bg)] text-[var(--color-text)]">
      <TabBar
        tabs={workspace.tabs}
        activeTabId={workspace.activeTabId}
        onSelect={(tabId) => void setActiveTab(tabId)}
        onClose={(tabId) => void closeTab(tabId)}
        onNewTab={() => void createTab(settings.defaultLayout)}
        onOpenSettings={handleOpenSettings}
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
          const isActive = tab.id === workspace.activeTabId;

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
