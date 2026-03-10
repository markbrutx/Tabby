import { useCallback, useEffect, useRef } from "react";
import type { CommitInfo, DiffContent } from "@/features/git/domain/models";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

export interface HistoryPanelProps {
  readonly commits: readonly CommitInfo[];
  readonly loading: boolean;
  readonly hasMore: boolean;
  readonly selectedCommitHash: string | null;
  readonly headCommitHash: string | null;
  readonly commitDiffContent: DiffContent | null;
  readonly onSelectCommit: (hash: string | null) => void;
  readonly onLoadMore: () => void;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function formatRelativeDate(dateString: string): string {
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSec = Math.floor(diffMs / 1000);
  const diffMin = Math.floor(diffSec / 60);
  const diffHr = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHr / 24);
  const diffWeek = Math.floor(diffDay / 7);
  const diffMonth = Math.floor(diffDay / 30);

  if (diffSec < 60) return "just now";
  if (diffMin < 60) return `${diffMin}m ago`;
  if (diffHr < 24) return `${diffHr}h ago`;
  if (diffDay < 7) return `${diffDay}d ago`;
  if (diffWeek < 5) return `${diffWeek}w ago`;
  if (diffMonth < 12) return `${diffMonth}mo ago`;
  return date.toLocaleDateString();
}

function firstLine(message: string): string {
  const idx = message.indexOf("\n");
  return idx === -1 ? message : message.slice(0, idx);
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

function EmptyState() {
  return (
    <div
      className="flex h-full items-center justify-center"
      data-testid="history-empty"
    >
      <div className="flex flex-col items-center gap-2 text-center">
        <span className="text-sm text-[var(--color-text-soft)]">
          No commits yet
        </span>
        <span className="text-xs text-[var(--color-text-soft)]">
          This repository has no commit history.
        </span>
      </div>
    </div>
  );
}

function LoadingIndicator() {
  return (
    <div
      className="flex items-center justify-center py-3"
      data-testid="history-loading"
    >
      <div className="h-4 w-4 animate-spin rounded-full border-2 border-[var(--color-border)] border-t-[var(--color-accent)]" />
    </div>
  );
}

interface CommitRowProps {
  readonly commit: CommitInfo;
  readonly isSelected: boolean;
  readonly isHead: boolean;
  readonly onClick: () => void;
}

function CommitRow({ commit, isSelected, isHead, onClick }: CommitRowProps) {
  return (
    <button
      type="button"
      className={`flex w-full flex-col gap-0.5 border-b border-[var(--color-border)] px-3 py-2 text-left transition-colors ${
        isSelected
          ? "bg-[var(--color-accent)]/10"
          : "hover:bg-[var(--color-bg-elevated)]"
      }`}
      onClick={onClick}
      data-testid={`commit-row-${commit.shortHash}`}
    >
      <div className="flex items-center gap-2">
        <span className="font-mono text-xs text-[var(--color-accent)]">
          {commit.shortHash}
        </span>
        {isHead && (
          <span
            className="rounded bg-[var(--color-accent)] px-1 py-0.5 text-[10px] font-bold leading-none text-white"
            data-testid="head-indicator"
          >
            HEAD
          </span>
        )}
        <span className="ml-auto text-[10px] text-[var(--color-text-soft)]">
          {formatRelativeDate(commit.date)}
        </span>
      </div>
      <span className="truncate text-xs text-[var(--color-text)]">
        {firstLine(commit.message)}
      </span>
      <span className="text-[10px] text-[var(--color-text-soft)]">
        {commit.authorName}
      </span>
    </button>
  );
}

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

const SCROLL_THRESHOLD = 100;

export function HistoryPanel({
  commits,
  loading,
  hasMore,
  selectedCommitHash,
  headCommitHash,
  commitDiffContent,
  onSelectCommit,
  onLoadMore,
}: HistoryPanelProps) {
  const scrollContainerRef = useRef<HTMLDivElement>(null);

  const handleScroll = useCallback(() => {
    const container = scrollContainerRef.current;
    if (container === null || loading || !hasMore) return;

    const { scrollTop, scrollHeight, clientHeight } = container;
    if (scrollHeight - scrollTop - clientHeight < SCROLL_THRESHOLD) {
      onLoadMore();
    }
  }, [loading, hasMore, onLoadMore]);

  useEffect(() => {
    const container = scrollContainerRef.current;
    if (container === null) return;

    container.addEventListener("scroll", handleScroll);
    return () => container.removeEventListener("scroll", handleScroll);
  }, [handleScroll]);

  if (!loading && commits.length === 0) {
    return <EmptyState />;
  }

  return (
    <div className="flex h-full min-h-0 flex-col" data-testid="history-panel">
      {/* Commit list */}
      <div
        ref={scrollContainerRef}
        className="min-h-0 flex-1 overflow-y-auto"
        data-testid="history-commit-list"
      >
        {commits.map((commit) => (
          <CommitRow
            key={commit.hash}
            commit={commit}
            isSelected={selectedCommitHash === commit.hash}
            isHead={headCommitHash === commit.hash}
            onClick={() => onSelectCommit(commit.hash)}
          />
        ))}
        {loading && <LoadingIndicator />}
        {!loading && !hasMore && commits.length > 0 && (
          <div className="py-2 text-center text-[10px] text-[var(--color-text-soft)]">
            End of history
          </div>
        )}
      </div>

      {/* Selected commit diff summary */}
      {selectedCommitHash !== null && commitDiffContent !== null && (
        <div
          className="border-t border-[var(--color-border)] px-3 py-2"
          data-testid="history-diff-summary"
        >
          <span className="text-xs text-[var(--color-text-soft)]">
            {commitDiffContent.filePath}
            {commitDiffContent.hunks.length > 0 && (
              <> &middot; {commitDiffContent.hunks.length} hunk{commitDiffContent.hunks.length !== 1 ? "s" : ""}</>
            )}
          </span>
        </div>
      )}
    </div>
  );
}
