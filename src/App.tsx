import { useEffect } from "react";
import { AlertTriangle, ChevronRight } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { AppSidebar } from "@/features/workspace/components/AppSidebar";
import { PaneGrid } from "@/features/workspace/components/PaneGrid";
import { SettingsDrawer } from "@/features/workspace/components/SettingsDrawer";
import { TabStrip } from "@/features/workspace/components/TabStrip";
import { useWorkspaceStore } from "@/features/workspace/store/workspaceStore";
import { useWorkspaceShortcuts } from "@/features/workspace/useWorkspaceShortcuts";

function App() {
  const {
    workspace,
    settings,
    profiles,
    error,
    isHydrating,
    isWorking,
    settingsOpen,
    initialize,
    createTab,
    closeTab,
    setActiveTab,
    focusPane,
    updatePaneProfile,
    updatePaneCwd,
    restartPane,
    updateSettings,
    setSettingsOpen,
    clearError,
  } = useWorkspaceStore();

  useEffect(() => {
    void initialize();
  }, [initialize]);

  useWorkspaceShortcuts({
    workspace,
    defaultLayout: settings?.defaultLayout ?? "2x2",
    onCreateTab: async () => {
      if (!settings) {
        return;
      }

      await createTab(settings.defaultLayout);
    },
    onCloseTab: closeTab,
    onSelectTab: setActiveTab,
  });

  if (isHydrating) {
    return (
      <div className="flex min-h-screen items-center justify-center p-8 text-[var(--color-text)]">
        <div className="surface-panel w-full max-w-xl rounded-[32px] p-8 text-center">
          <p className="text-xs uppercase tracking-[0.35em] text-[var(--color-text-muted)]">
            Booting
          </p>
          <h1 className="mt-4 text-4xl font-semibold">Preparing your terminal deck</h1>
          <p className="mt-3 text-sm text-[var(--color-text-soft)]">
            Initializing PTY managers, workspace defaults and Handy-style chrome.
          </p>
        </div>
      </div>
    );
  }

  if (!workspace || !settings) {
    return (
      <div className="flex min-h-screen items-center justify-center p-8 text-[var(--color-text)]">
        <div className="surface-panel w-full max-w-xl rounded-[32px] p-8">
          <div className="flex items-center gap-3 text-[var(--color-warning)]">
            <AlertTriangle size={20} />
            <p className="text-sm uppercase tracking-[0.25em]">Workspace unavailable</p>
          </div>
          <p className="mt-4 text-sm text-[var(--color-text-soft)]">
            {error ?? "Tabby could not bootstrap the workspace."}
          </p>
          <Button className="mt-6" onClick={() => void initialize()}>
            Retry bootstrap
          </Button>
        </div>
      </div>
    );
  }

  const activeTab =
    workspace.tabs.find((tab) => tab.id === workspace.activeTabId) ?? workspace.tabs[0];

  return (
    <div className="min-h-screen p-4 text-[var(--color-text)]">
      <div className="grid h-[calc(100vh-2rem)] grid-cols-[320px_minmax(0,1fr)] gap-4">
        <AppSidebar
          settings={settings}
          isWorking={isWorking}
          onCreateTab={(preset) => void createTab(preset)}
          onOpenSettings={() => setSettingsOpen(true)}
        />

        <main className="surface-panel flex min-h-0 flex-col rounded-[28px] p-4">
          <header className="mb-4 flex items-center justify-between gap-4">
            <div>
              <p className="text-xs uppercase tracking-[0.28em] text-[var(--color-text-muted)]">
                Active workspace
              </p>
              <div className="mt-2 flex items-center gap-2">
                <h2 data-testid="active-workspace-title" className="text-2xl font-semibold">
                  {activeTab.title}
                </h2>
                <ChevronRight size={16} className="text-[var(--color-text-muted)]" />
                <span className="rounded-full bg-[var(--color-accent-soft)] px-3 py-1 text-xs uppercase tracking-[0.2em] text-[var(--color-accent)]">
                  {activeTab.preset}
                </span>
              </div>
            </div>

            <div className="flex items-center gap-3">
              <div className="rounded-2xl border border-[var(--color-border)] bg-white/3 px-4 py-3 text-right text-sm">
                <p className="text-[var(--color-text-soft)]">
                  {activeTab.panes.length} panes, {settings.fontSize}px font
                </p>
                <p className="mt-1 text-xs uppercase tracking-[0.2em] text-[var(--color-text-muted)]">
                  {settings.defaultProfileId} default profile
                </p>
              </div>
              <Button onClick={() => void createTab(settings.defaultLayout)} disabled={isWorking}>
                New workspace
              </Button>
            </div>
          </header>

          {error ? (
            <div className="mb-4 flex items-center justify-between gap-4 rounded-2xl border border-[#c9555d]/40 bg-[#c9555d]/10 px-4 py-3 text-sm">
              <span>{error}</span>
              <Button variant="ghost" size="sm" onClick={clearError}>
                Dismiss
              </Button>
            </div>
          ) : null}

          <TabStrip
            tabs={workspace.tabs}
            activeTabId={workspace.activeTabId}
            isWorking={isWorking}
            onSelect={(tabId) => void setActiveTab(tabId)}
            onClose={(tabId) => void closeTab(tabId)}
          />

          <div className="mt-4 min-h-0 flex-1">
            {workspace.tabs.map((tab) => {
              const isActive = tab.id === workspace.activeTabId;

              return (
                <div key={tab.id} className={`h-full ${isActive ? "block" : "hidden"}`}>
                  <PaneGrid
                    tab={tab}
                    profiles={profiles}
                    fontSize={settings.fontSize}
                    visible={isActive}
                    onFocus={focusPane}
                    onUpdateProfile={(paneId, profileId, startupCommand) =>
                      updatePaneProfile({ paneId, profileId, startupCommand })
                    }
                    onUpdateCwd={(paneId, cwd) => updatePaneCwd({ paneId, cwd })}
                    onRestart={restartPane}
                  />
                </div>
              );
            })}
          </div>
        </main>
      </div>

      {settingsOpen ? (
        <SettingsDrawer
          settings={settings}
          profiles={profiles}
          onClose={() => setSettingsOpen(false)}
          onSave={updateSettings}
        />
      ) : null}
    </div>
  );
}

export default App;
