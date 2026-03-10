import { useEffect, useRef } from "react";
import type { StoreApi, UseBoundStore } from "zustand";
import type { PaneSnapshotModel } from "@/features/workspace/model/workspaceSnapshot";
import type { GitClient } from "@/app-shell/clients";
import {
  createGitPaneStore,
  type GitActiveView,
  type GitPaneState,
} from "@/features/git/application/useGitPaneStore";

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
          className="flex w-56 flex-col overflow-y-auto border-r border-[var(--color-border)]"
          data-testid="git-file-list"
        >
          {files.length === 0 ? (
            <div className="p-3 text-xs text-[var(--color-text-soft)]">
              No changes
            </div>
          ) : (
            files.map((file) => (
              <button
                key={file.path}
                type="button"
                className={`w-full px-3 py-1 text-left text-xs transition-colors ${
                  selectedFile === file.path
                    ? "bg-[var(--color-accent)]/15 text-[var(--color-text)]"
                    : "text-[var(--color-text-soft)] hover:bg-[var(--color-surface-hover)]"
                }`}
                onClick={() => void selectFile(file.path)}
              >
                <span className="truncate">{file.path}</span>
              </button>
            ))
          )}
        </div>

        {/* Center — diff content */}
        <div className="flex min-w-0 flex-1 flex-col">
          <div
            className="flex-1 overflow-auto p-3"
            data-testid="git-diff-area"
          >
            {diffContent ? (
              <pre className="font-mono text-xs leading-5 text-[var(--color-text)]">
                {diffContent.hunks.map((hunk) =>
                  hunk.lines.map((line, idx) => (
                    <div
                      key={`${hunk.header}-${idx}`}
                      className={
                        line.kind === "addition"
                          ? "bg-green-900/20 text-green-300"
                          : line.kind === "deletion"
                            ? "bg-red-900/20 text-red-300"
                            : ""
                      }
                    >
                      {line.content}
                    </div>
                  )),
                )}
              </pre>
            ) : (
              <div className="flex h-full items-center justify-center text-xs text-[var(--color-text-soft)]">
                Select a file to view diff
              </div>
            )}
          </div>

          {/* Bottom — commit area */}
          <div
            className="border-t border-[var(--color-border)] p-3"
            data-testid="git-commit-area"
          >
            <textarea
              className="w-full resize-none rounded border border-[var(--color-border)] bg-[var(--color-surface)] p-2 text-xs text-[var(--color-text)] placeholder:text-[var(--color-text-soft)] focus:border-[var(--color-accent)] focus:outline-none"
              placeholder="Commit message..."
              rows={2}
            />
          </div>
        </div>
      </div>
    </div>
  );
}
