import { useEffect, useMemo, useRef } from "react";
import type { StoreApi, UseBoundStore } from "zustand";
import type { PaneSnapshotModel } from "@/features/workspace/model/workspaceSnapshot";
import type { GitClient } from "@/app-shell/clients";
import {
  createGitPaneStore,
  type GitActiveView,
  type GitPaneState,
} from "@/features/git/application/useGitPaneStore";
import { FileTreePanel } from "./FileTreePanel";
import { DiffViewer, type StagingCallbacks } from "./DiffViewer";
import { CommitPanel } from "./CommitPanel";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

export interface GitPaneProps {
  readonly pane: PaneSnapshotModel;
  readonly gitClient: GitClient;
}

// ---------------------------------------------------------------------------
// View tab buttons
// ---------------------------------------------------------------------------

const VIEW_TABS: readonly { readonly key: GitActiveView; readonly label: string }[] = [
  { key: "changes", label: "Changes" },
  { key: "history", label: "History" },
  { key: "branches", label: "Branches" },
  { key: "stash", label: "Stash" },
];

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

function LoadingSkeleton() {
  return (
    <div
      className="flex h-full items-center justify-center"
      data-testid="git-loading"
    >
      <div className="flex flex-col items-center gap-3">
        <div className="h-6 w-6 animate-spin rounded-full border-2 border-[var(--color-border)] border-t-[var(--color-accent)]" />
        <span className="text-sm text-[var(--color-text-soft)]">
          Loading repository...
        </span>
      </div>
    </div>
  );
}

function ErrorState({ message }: { readonly message: string }) {
  return (
    <div
      className="flex h-full items-center justify-center"
      data-testid="git-error"
    >
      <div className="flex flex-col items-center gap-2 text-center">
        <span className="text-sm font-medium text-red-400">Error</span>
        <span className="max-w-xs text-sm text-[var(--color-text-soft)]">
          {message}
        </span>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

export function GitPane({ pane, gitClient }: GitPaneProps) {
  const storeRef = useRef<UseBoundStore<StoreApi<GitPaneState>> | null>(null);

  if (storeRef.current === null) {
    storeRef.current = createGitPaneStore({ gitClient, paneId: pane.id });
  }

  const store = storeRef.current;
  const loading = store((s) => s.loading);
  const error = store((s) => s.error);
  const activeView = store((s) => s.activeView);
  const files = store((s) => s.files);
  const selectedFile = store((s) => s.selectedFile);
  const diffContent = store((s) => s.diffContent);
  const repoState = store((s) => s.repoState);
  const refreshStatus = store((s) => s.refreshStatus);
  const setActiveView = store((s) => s.setActiveView);
  const selectFile = store((s) => s.selectFile);
  const stageFiles = store((s) => s.stageFiles);
  const unstageFiles = store((s) => s.unstageFiles);
  const discardChanges = store((s) => s.discardChanges);
  const stageLines = store((s) => s.stageLines);
  const unstageLines = store((s) => s.unstageLines);
  const stageHunk = store((s) => s.stageHunk);
  const unstageHunk = store((s) => s.unstageHunk);
  const stagedLinesSet = store((s) => s.stagedLines);
  const commitAction = store((s) => s.commit);
  const fetchLastCommitInfo = store((s) => s.fetchLastCommitInfo);

  const stagingCallbacks: StagingCallbacks = useMemo(() => ({
    onStageLines: (filePath: string, lineRanges: string[]) => void stageLines(filePath, lineRanges),
    onUnstageLines: (filePath: string, lineRanges: string[]) => void unstageLines(filePath, lineRanges),
    onStageHunk: (filePath: string, hunkIndex: number) => void stageHunk(filePath, hunkIndex),
    onUnstageHunk: (filePath: string, hunkIndex: number) => void unstageHunk(filePath, hunkIndex),
  }), [stageLines, unstageLines, stageHunk, unstageHunk]);

  useEffect(() => {
    void refreshStatus();
  }, [refreshStatus]);

  if (loading) {
    return (
      <div className="flex h-full flex-col bg-[var(--color-bg)]">
        <LoadingSkeleton />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-full flex-col bg-[var(--color-bg)]">
        <ErrorState message={error} />
      </div>
    );
  }

  return (
    <div
      className="flex h-full flex-col bg-[var(--color-bg)]"
      data-testid="git-pane"
    >
      {/* Header with branch info and view tabs */}
      <div className="flex items-center gap-2 border-b border-[var(--color-border)] px-3 py-1.5">
        {repoState?.headBranch && (
          <span className="text-xs font-medium text-[var(--color-text)]">
            {repoState.headBranch}
          </span>
        )}
        <div className="ml-auto flex gap-1">
          {VIEW_TABS.map((tab) => (
            <button
              key={tab.key}
              type="button"
              className={`rounded px-2 py-0.5 text-xs transition-colors ${
                activeView === tab.key
                  ? "bg-[var(--color-accent)] text-white"
                  : "text-[var(--color-text-soft)] hover:text-[var(--color-text)]"
              }`}
              onClick={() => setActiveView(tab.key)}
            >
              {tab.label}
            </button>
          ))}
        </div>
      </div>

      {/* Main layout: file list | diff | commit area */}
      <div className="flex min-h-0 flex-1">
        {/* Left panel — file tree */}
        <div
          className="flex w-56 flex-col border-r border-[var(--color-border)]"
          data-testid="git-file-list"
        >
          <FileTreePanel
            files={files}
            selectedFile={selectedFile}
            onSelectFile={(path) => void selectFile(path)}
            onStageFiles={(paths) => void stageFiles(paths)}
            onUnstageFiles={(paths) => void unstageFiles(paths)}
            onDiscardChanges={(paths) => void discardChanges(paths)}
          />
        </div>

        {/* Center — diff content */}
        <div className="flex min-w-0 flex-1 flex-col">
          <div
            className="min-h-0 flex-1"
            data-testid="git-diff-area"
          >
            <DiffViewer diffContent={diffContent} staging={stagingCallbacks} stagedLines={stagedLinesSet} />
          </div>

          {/* Bottom — commit area */}
          <div
            className="border-t border-[var(--color-border)] p-3"
            data-testid="git-commit-area"
          >
            <CommitPanel
              files={files}
              onCommit={commitAction}
              onFetchLastCommitInfo={fetchLastCommitInfo}
              onCommitSuccess={async () => { await refreshStatus(); }}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
