import { useEffect, useState } from "react";
import { ChevronRight, PanelLeft } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { RecoveryScreen } from "@/components/RecoveryScreen";
import { OnboardingWizard } from "@/features/onboarding/OnboardingWizard";
import { AppSidebar } from "@/features/workspace/components/AppSidebar";
import { PaneGrid } from "@/features/workspace/components/PaneGrid";
import { SettingsDrawer } from "@/features/workspace/components/SettingsDrawer";
import { TabStrip } from "@/features/workspace/components/TabStrip";
import {
  selectActiveTab,
  selectWorkspaceSummary,
} from "@/features/workspace/selectors";
import { useWorkspaceStore } from "@/features/workspace/store/workspaceStore";
import { CUSTOM_PROFILE_ID } from "@/features/workspace/domain";
import {
  applyResolvedTheme,
  useResolvedTheme,
} from "@/features/workspace/theme";
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

  const [sidebarOpen, setSidebarOpen] = useState(false);
  const resolvedTheme = useResolvedTheme(settings?.theme);

  useEffect(() => {
    void initialize();
  }, [initialize]);

  useEffect(() => {
    applyResolvedTheme(resolvedTheme);
  }, [resolvedTheme]);

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
    onFocusPane: focusPane,
    onRestartPane: restartPane,
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
            Initializing PTY managers and workspace defaults.
          </p>
        </div>
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

  if (!settings.hasCompletedOnboarding) {
    return (
      <OnboardingWizard
        initialSettings={settings}
        profiles={profiles}
        onComplete={async (nextSettings) => {
          const existingTabIds = workspace.tabs.map((tab) => tab.id);

          // Create the new tab FIRST so the workspace is ready before transition
          await createTab(nextSettings.defaultLayout, {
            cwd: nextSettings.defaultWorkingDirectory,
            profileId: nextSettings.defaultProfileId,
            startupCommand:
              nextSettings.defaultProfileId === CUSTOM_PROFILE_ID
                ? nextSettings.defaultCustomCommand
                : undefined,
          });

          // Close old bootstrap tabs sequentially
          for (const tabId of existingTabIds) {
            await closeTab(tabId);
          }

          // Update settings LAST — this flips hasCompletedOnboarding and
          // triggers the transition from onboarding to workspace UI.
          await updateSettings(nextSettings);
        }}
      />
    );
  }

  const activeTab = selectActiveTab(workspace);
  const workspaceSummary = selectWorkspaceSummary(workspace, activeTab);

  if (!activeTab) {
    return (
      <RecoveryScreen
        title="No active workspace"
        message={error ?? "All workspaces have been closed. Pick a layout to get back to work."}
        onRetry={() => void initialize()}
        onCreateTab={(preset) => void createTab(preset)}
      />
    );
  }

  return (
    <div className="min-h-screen p-4 text-[var(--color-text)]">
      <div className="flex h-[calc(100vh-2rem)] flex-col gap-4">
        <main className="surface-panel flex min-h-0 flex-1 flex-col rounded-[28px] p-4">
          <header className="mb-4 flex items-center justify-between gap-4">
            <div className="flex items-center gap-3">
              <Button
                data-testid="toggle-sidebar"
                variant="ghost"
                size="sm"
                onClick={() => setSidebarOpen(true)}
                aria-label="Open sidebar"
              >
                <PanelLeft size={18} />
              </Button>
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
            </div>

            <div className="flex items-center gap-3">
              <div className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface-overlay)] px-4 py-3 text-right text-sm">
                <p className="text-[var(--color-text-soft)]">
                  {workspaceSummary.paneCount} panes, {settings.fontSize}px font
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
            <div className="mb-4 flex items-center justify-between gap-4 rounded-2xl border border-[var(--color-danger)] bg-[var(--color-danger-soft)] px-4 py-3 text-sm">
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
                    theme={resolvedTheme}
                    visible={isActive}
                    onFocus={focusPane}
                    onUpdateProfile={(paneId, profileId, startupCommand) =>
                      updatePaneProfile({
                        paneId,
                        profileId,
                        startupCommand: startupCommand ?? null,
                      })
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

      {sidebarOpen ? (
        <AppSidebar
          settings={settings}
          isWorking={isWorking}
          onCreateTab={(preset) => void createTab(preset)}
          onOpenSettings={() => setSettingsOpen(true)}
          onClose={() => setSidebarOpen(false)}
        />
      ) : null}

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
