import { useState, useCallback, useEffect, useRef } from "react";
import type { CommitInfo, FileStatus } from "@/features/git/domain/models";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface CommitPanelProps {
  readonly files: readonly FileStatus[];
  readonly onCommit: (message: string, amend: boolean) => Promise<void>;
  readonly onPushAll: (message: string) => Promise<void>;
  readonly onFetchLastCommitInfo: () => Promise<CommitInfo | null>;
  readonly onCommitSuccess: () => Promise<void>;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function countStagedFiles(files: readonly FileStatus[]): number {
  return files.filter(
    (f) =>
      f.indexStatus === "modified" ||
      f.indexStatus === "added" ||
      f.indexStatus === "deleted" ||
      f.indexStatus === "renamed" ||
      f.indexStatus === "copied",
  ).length;
}

// ---------------------------------------------------------------------------
// CommitPanel
// ---------------------------------------------------------------------------

export function CommitPanel({ files, onCommit, onPushAll, onFetchLastCommitInfo, onCommitSuccess }: CommitPanelProps) {
  const [message, setMessage] = useState("");
  const [amend, setAmend] = useState(false);
  const [authorName, setAuthorName] = useState<string | null>(null);
  const [authorEmail, setAuthorEmail] = useState<string | null>(null);
  const [commitError, setCommitError] = useState<string | null>(null);
  const [committing, setCommitting] = useState(false);
  const [pushing, setPushing] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const prevAmendRef = useRef(false);

  const stagedCount = countStagedFiles(files);
  const canCommit = stagedCount > 0 && message.trim().length > 0 && !committing;

  // Fetch author info on mount
  useEffect(() => {
    async function fetchAuthorInfo() {
      try {
        const info = await onFetchLastCommitInfo();
        if (info !== null) {
          setAuthorName(info.authorName);
          setAuthorEmail(info.authorEmail);
        }
      } catch {
        // Author info is optional; silently ignore
      }
    }
    void fetchAuthorInfo();
  }, [onFetchLastCommitInfo]);

  // Populate message when amend is toggled on
  useEffect(() => {
    if (amend && !prevAmendRef.current) {
      async function fetchLastMessage() {
        try {
          const info = await onFetchLastCommitInfo();
          if (info !== null) {
            setMessage(info.message);
          }
        } catch {
          // If we can't fetch, leave message as-is
        }
      }
      void fetchLastMessage();
    }
    prevAmendRef.current = amend;
  }, [amend, onFetchLastCommitInfo]);

  const handleCommit = useCallback(async () => {
    if (!canCommit) return;

    setCommitting(true);
    setCommitError(null);

    try {
      await onCommit(message.trim(), amend);
      setMessage("");
      setAmend(false);
      await onCommitSuccess();
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : "Commit failed";
      setCommitError(errorMessage);
    } finally {
      setCommitting(false);
    }
  }, [canCommit, onCommit, message, amend, onCommitSuccess]);

  const handlePushAll = useCallback(async () => {
    if (pushing) return;

    setPushing(true);
    setCommitError(null);

    try {
      await onPushAll(message);
      setMessage("");
      setAmend(false);
      await onCommitSuccess();
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : "Push failed";
      setCommitError(errorMessage);
    } finally {
      setPushing(false);
    }
  }, [pushing, onPushAll, message, onCommitSuccess]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
        e.preventDefault();
        void handleCommit();
      }
    },
    [handleCommit],
  );

  return (
    <div className="flex flex-col gap-2" data-testid="commit-panel">
      {/* Author info */}
      {authorName !== null && (
        <div className="flex items-center gap-1 text-[10px] text-[var(--color-text-soft)]" data-testid="commit-author">
          <span>{authorName}</span>
          {authorEmail !== null && (
            <span>&lt;{authorEmail}&gt;</span>
          )}
        </div>
      )}

      {/* Commit message textarea */}
      <textarea
        ref={textareaRef}
        className="w-full resize-none rounded border border-[var(--color-border)] bg-[var(--color-surface)] p-2 text-xs text-[var(--color-text)] placeholder:text-[var(--color-text-soft)] focus:border-[var(--color-accent)] focus:outline-none"
        placeholder="Commit message..."
        rows={3}
        value={message}
        onChange={(e) => {
          setMessage(e.target.value);
          setCommitError(null);
        }}
        onKeyDown={handleKeyDown}
        data-testid="commit-message-input"
      />

      {/* Controls row */}
      <div className="flex items-center gap-2">
        {/* Amend checkbox */}
        <label className="flex items-center gap-1 text-xs text-[var(--color-text-soft)]" data-testid="amend-label">
          <input
            type="checkbox"
            checked={amend}
            onChange={(e) => setAmend(e.target.checked)}
            className="accent-[var(--color-accent)]"
            data-testid="amend-checkbox"
          />
          Amend
        </label>

        {/* Staged count */}
        <span className="text-[10px] text-[var(--color-text-soft)]" data-testid="staged-count">
          {stagedCount} {stagedCount === 1 ? "file" : "files"} staged
        </span>

        {/* Spacer */}
        <div className="flex-1" />

        {/* Commit button */}
        <button
          type="button"
          disabled={!canCommit}
          onClick={() => void handleCommit()}
          className={`rounded px-3 py-1 text-xs font-medium transition-colors ${
            canCommit
              ? "bg-[var(--color-accent)] text-white hover:opacity-90"
              : "cursor-not-allowed bg-[var(--color-surface)] text-[var(--color-text-soft)] opacity-50"
          }`}
          data-testid="commit-button"
        >
          {committing ? "Committing..." : "Commit"}
        </button>

        {/* Push All button */}
        <button
          type="button"
          disabled={pushing || committing}
          onClick={() => void handlePushAll()}
          className={`rounded px-3 py-1 text-xs font-medium transition-colors ${
            !pushing && !committing
              ? "bg-[var(--color-success,#22c55e)] text-white hover:opacity-90"
              : "cursor-not-allowed bg-[var(--color-surface)] text-[var(--color-text-soft)] opacity-50"
          }`}
          title="Stage all, commit, and push"
          data-testid="push-all-button"
        >
          {pushing ? "Pushing..." : "Push All"}
        </button>
      </div>

      {/* Error display */}
      {commitError !== null && (
        <div
          className="rounded bg-red-900/20 px-2 py-1 text-xs text-red-400"
          data-testid="commit-error"
        >
          {commitError}
        </div>
      )}
    </div>
  );
}
