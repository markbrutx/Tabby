import { useEffect, useMemo, useRef } from "react";
import type { StoreApi, UseBoundStore } from "zustand";
import type { PaneSnapshotModel } from "@/features/workspace/model/workspaceSnapshot";
import type { GitClient } from "@/app-shell/clients";
import {
  createGitPaneStore,
  type GitActiveView,
  type GitPaneState,
} from "@/features/git/application/useGitPaneStore";
import { PaneErrorState } from "@/components/PaneErrorState";
import { Button } from "@/components/ui/Button";
import { RotateCcw } from "lucide-react";
import { FileTreePanel } from "./FileTreePanel";
import { DiffViewer, type StagingCallbacks } from "./DiffViewer";
import { CommitPanel } from "./CommitPanel";
import { BranchSelector } from "./BranchSelector";
import { HistoryPanel } from "./HistoryPanel";
import { BlameView } from "./BlameView";
import { StashPanel } from "./StashPanel";

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
  const pushAllAction = store((s) => s.pushAll);
  const fetchLastCommitInfo = store((s) => s.fetchLastCommitInfo);
  const branches = store((s) => s.branches);
  const branchesLoading = store((s) => s.branchesLoading);
  const listBranches = store((s) => s.listBranches);
  const checkoutBranch = store((s) => s.checkoutBranch);
  const createBranch = store((s) => s.createBranch);
  const deleteBranch = store((s) => s.deleteBranch);
  const commitLog = store((s) => s.commitLog);
  const commitLogLoading = store((s) => s.commitLogLoading);
  const hasMoreCommits = store((s) => s.hasMoreCommits);
  const selectedCommitHash = store((s) => s.selectedCommitHash);
  const commitDiffContent = store((s) => s.commitDiffContent);
  const fetchCommitLog = store((s) => s.fetchCommitLog);
  const fetchMoreCommits = store((s) => s.fetchMoreCommits);
  const selectCommit = store((s) => s.selectCommit);
  const blameEntries = store((s) => s.blameEntries);
  const blameFilePath = store((s) => s.blameFilePath);
  const blameLoading = store((s) => s.blameLoading);
  const fetchBlame = store((s) => s.fetchBlame);
  const stashes = store((s) => s.stashes);
  const stashesLoading = store((s) => s.stashesLoading);
  const listStashes = store((s) => s.listStashes);
  const stashPush = store((s) => s.stashPush);
  const stashPop = store((s) => s.stashPop);
  const stashApply = store((s) => s.stashApply);
  const stashDrop = store((s) => s.stashDrop);

  const stagingCallbacks: StagingCallbacks = useMemo(() => ({
    onStageLines: (filePath: string, lineRanges: string[]) => void stageLines(filePath, lineRanges),
    onUnstageLines: (filePath: string, lineRanges: string[]) => void unstageLines(filePath, lineRanges),
    onStageHunk: (filePath: string, hunkIndex: number) => void stageHunk(filePath, hunkIndex),
    onUnstageHunk: (filePath: string, hunkIndex: number) => void unstageHunk(filePath, hunkIndex),
  }), [stageLines, unstageLines, stageHunk, unstageHunk]);

  useEffect(() => {
    void refreshStatus();
  }, [refreshStatus]);

  useEffect(() => {
    if (activeView === "branches") {
      void listBranches();
    }
    if (activeView === "history") {
      void fetchCommitLog();
    }
    if (activeView === "stash") {
      void listStashes();
    }
  }, [activeView, listBranches, fetchCommitLog, listStashes]);

  if (loading) {
    return (
      <div className="flex h-full flex-col bg-[var(--color-bg)]">
        <LoadingSkeleton />
      </div>
    );
  }

  if (error) {
    return (
      <PaneErrorState
        title="Git Error"
        message={error}
        action={
          <Button variant="secondary" onClick={() => { void refreshStatus(); }}>
            <RotateCcw size={14} className="mr-2" />
            Retry
          </Button>
        }
      />
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
              className={`rounded px-2 py-0.5 text-xs transition-colors ${activeView === tab.key
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

      {/* Main layout */}
      <div className="flex min-h-0 flex-1">
        {activeView === "branches" ? (
          <div className="flex min-w-0 flex-1 flex-col" data-testid="git-branches-view">
            <BranchSelector
              branches={branches}
              loading={branchesLoading}
              onCheckout={checkoutBranch}
              onCreateBranch={createBranch}
              onDeleteBranch={deleteBranch}
              onRefresh={listBranches}
            />
          </div>
        ) : activeView === "blame" ? (
          <div className="flex min-w-0 flex-1 flex-col" data-testid="git-blame-view">
            {blameLoading ? (
              <div className="flex h-full items-center justify-center">
                <div className="h-5 w-5 animate-spin rounded-full border-2 border-[var(--color-border)] border-t-[var(--color-accent)]" />
              </div>
            ) : (
              <BlameView
                filePath={blameFilePath ?? ""}
                entries={blameEntries}
                onCommitClick={(hash) => {
                  setActiveView("history");
                  void fetchCommitLog().then(() => void selectCommit(hash));
                }}
              />
            )}
          </div>
        ) : activeView === "history" ? (
          <>
            {/* Left panel — commit list */}
            <div
              className="flex w-72 flex-col border-r border-[var(--color-border)]"
              data-testid="git-history-view"
            >
              <HistoryPanel
                commits={commitLog}
                loading={commitLogLoading}
                hasMore={hasMoreCommits}
                selectedCommitHash={selectedCommitHash}
                headCommitHash={commitLog.length > 0 ? commitLog[0].hash : null}
                commitDiffContent={commitDiffContent}
                onSelectCommit={(hash) => void selectCommit(hash)}
                onLoadMore={fetchMoreCommits}
              />
            </div>

            {/* Right panel — diff viewer for selected commit */}
            <div className="flex min-w-0 flex-1 flex-col">
              <div className="min-h-0 flex-1" data-testid="git-history-diff-area">
                <DiffViewer diffContent={commitDiffContent} />
              </div>
            </div>
          </>
        ) : activeView === "stash" ? (
          <div className="flex min-w-0 flex-1 flex-col" data-testid="git-stash-view">
            <StashPanel
              stashes={stashes}
              loading={stashesLoading}
              onPush={stashPush}
              onPop={stashPop}
              onApply={stashApply}
              onDrop={stashDrop}
              onRefresh={listStashes}
            />
          </div>
        ) : (
          <>
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
                onBlameFile={(path) => void fetchBlame(path)}
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
                  onPushAll={pushAllAction}
                  onFetchLastCommitInfo={fetchLastCommitInfo}
                  onCommitSuccess={async () => { await refreshStatus(); }}
                />
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
