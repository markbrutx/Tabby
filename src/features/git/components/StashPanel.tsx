import { useState } from "react";
import type { StashEntry } from "@/features/git/domain/models";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

export interface StashPanelProps {
  readonly stashes: readonly StashEntry[];
  readonly loading: boolean;
  readonly onPush: (message: string | null) => void;
  readonly onPop: (index: number) => void;
  readonly onApply: (index: number) => void;
  readonly onDrop: (index: number) => void;
  readonly onRefresh: () => void;
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

  if (diffSec < 60) return "just now";
  if (diffMin < 60) return `${diffMin}m ago`;
  if (diffHr < 24) return `${diffHr}h ago`;
  if (diffDay < 7) return `${diffDay}d ago`;
  return date.toLocaleDateString();
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

function EmptyState() {
  return (
    <div
      className="flex h-full items-center justify-center"
      data-testid="stash-empty"
    >
      <div className="flex flex-col items-center gap-2 text-center">
        <span className="text-sm text-[var(--color-text-soft)]">
          No stashes
        </span>
        <span className="text-xs text-[var(--color-text-soft)]">
          Use the form above to stash your current changes.
        </span>
      </div>
    </div>
  );
}

function LoadingIndicator() {
  return (
    <div
      className="flex items-center justify-center py-3"
      data-testid="stash-loading"
    >
      <div className="h-4 w-4 animate-spin rounded-full border-2 border-[var(--color-border)] border-t-[var(--color-accent)]" />
    </div>
  );
}

interface StashRowProps {
  readonly entry: StashEntry;
  readonly isSelected: boolean;
  readonly onClick: () => void;
}

function StashRow({ entry, isSelected, onClick }: StashRowProps) {
  return (
    <button
      type="button"
      className={`flex w-full flex-col gap-0.5 border-b border-[var(--color-border)] px-3 py-2 text-left transition-colors ${
        isSelected
          ? "bg-[var(--color-accent)]/10"
          : "hover:bg-[var(--color-bg-elevated)]"
      }`}
      onClick={onClick}
      data-testid={`stash-row-${entry.index}`}
    >
      <div className="flex items-center gap-2">
        <span className="font-mono text-xs text-[var(--color-accent)]">
          stash@{"{"}
          {entry.index}
          {"}"}
        </span>
        <span className="ml-auto text-[10px] text-[var(--color-text-soft)]">
          {formatRelativeDate(entry.date)}
        </span>
      </div>
      <span className="truncate text-xs text-[var(--color-text)]">
        {entry.message}
      </span>
    </button>
  );
}

interface ConfirmDropDialogProps {
  readonly stashIndex: number;
  readonly onConfirm: () => void;
  readonly onCancel: () => void;
}

function ConfirmDropDialog({ stashIndex, onConfirm, onCancel }: ConfirmDropDialogProps) {
  return (
    <div
      className="flex items-center gap-2 rounded border border-red-500/30 bg-red-500/10 px-3 py-2"
      data-testid="stash-drop-confirm"
    >
      <span className="flex-1 text-xs text-[var(--color-text)]">
        Drop stash@{"{"}
        {stashIndex}
        {"}"}?
      </span>
      <button
        type="button"
        className="rounded bg-red-500 px-2 py-0.5 text-xs text-white transition-colors hover:bg-red-600"
        onClick={onConfirm}
        data-testid="stash-drop-confirm-button"
      >
        Drop
      </button>
      <button
        type="button"
        className="rounded px-2 py-0.5 text-xs text-[var(--color-text-soft)] transition-colors hover:text-[var(--color-text)]"
        onClick={onCancel}
        data-testid="stash-drop-cancel-button"
      >
        Cancel
      </button>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

export function StashPanel({
  stashes,
  loading,
  onPush,
  onPop,
  onApply,
  onDrop,
  onRefresh,
}: StashPanelProps) {
  const [pushMessage, setPushMessage] = useState("");
  const [selectedIndex, setSelectedIndex] = useState<number | null>(null);
  const [confirmDropIndex, setConfirmDropIndex] = useState<number | null>(null);

  const handlePush = () => {
    const message = pushMessage.trim();
    onPush(message.length > 0 ? message : null);
    setPushMessage("");
  };

  const handlePop = () => {
    if (selectedIndex === null) return;
    onPop(selectedIndex);
    setSelectedIndex(null);
  };

  const handleApply = () => {
    if (selectedIndex === null) return;
    onApply(selectedIndex);
  };

  const handleDropRequest = () => {
    if (selectedIndex === null) return;
    setConfirmDropIndex(selectedIndex);
  };

  const handleDropConfirm = () => {
    if (confirmDropIndex === null) return;
    onDrop(confirmDropIndex);
    setConfirmDropIndex(null);
    setSelectedIndex(null);
  };

  const handleDropCancel = () => {
    setConfirmDropIndex(null);
  };

  return (
    <div className="flex h-full min-h-0 flex-col" data-testid="stash-panel">
      {/* Push form */}
      <div className="flex items-center gap-2 border-b border-[var(--color-border)] px-3 py-2">
        <input
          type="text"
          className="min-w-0 flex-1 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 py-1 text-xs text-[var(--color-text)] placeholder:text-[var(--color-text-soft)] focus:border-[var(--color-accent)] focus:outline-none"
          placeholder="Stash message (optional)"
          value={pushMessage}
          onChange={(e) => setPushMessage(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") handlePush();
          }}
          data-testid="stash-message-input"
        />
        <button
          type="button"
          className="rounded bg-[var(--color-accent)] px-2 py-1 text-xs text-white transition-colors hover:opacity-90"
          onClick={handlePush}
          data-testid="stash-push-button"
        >
          Push
        </button>
        <button
          type="button"
          className="rounded px-2 py-1 text-xs text-[var(--color-text-soft)] transition-colors hover:text-[var(--color-text)]"
          onClick={onRefresh}
          data-testid="stash-refresh-button"
        >
          Refresh
        </button>
      </div>

      {/* Action bar for selected stash */}
      {selectedIndex !== null && confirmDropIndex === null && (
        <div
          className="flex items-center gap-1 border-b border-[var(--color-border)] px-3 py-1.5"
          data-testid="stash-actions"
        >
          <span className="flex-1 text-xs text-[var(--color-text-soft)]">
            stash@{"{"}
            {selectedIndex}
            {"}"}
          </span>
          <button
            type="button"
            className="rounded bg-[var(--color-accent)] px-2 py-0.5 text-xs text-white transition-colors hover:opacity-90"
            onClick={handlePop}
            data-testid="stash-pop-button"
          >
            Pop
          </button>
          <button
            type="button"
            className="rounded border border-[var(--color-border)] px-2 py-0.5 text-xs text-[var(--color-text)] transition-colors hover:bg-[var(--color-bg-elevated)]"
            onClick={handleApply}
            data-testid="stash-apply-button"
          >
            Apply
          </button>
          <button
            type="button"
            className="rounded border border-red-500/30 px-2 py-0.5 text-xs text-red-400 transition-colors hover:bg-red-500/10"
            onClick={handleDropRequest}
            data-testid="stash-drop-button"
          >
            Drop
          </button>
        </div>
      )}

      {/* Drop confirmation */}
      {confirmDropIndex !== null && (
        <div className="border-b border-[var(--color-border)] px-3 py-1.5">
          <ConfirmDropDialog
            stashIndex={confirmDropIndex}
            onConfirm={handleDropConfirm}
            onCancel={handleDropCancel}
          />
        </div>
      )}

      {/* Stash list */}
      <div className="min-h-0 flex-1 overflow-y-auto" data-testid="stash-list">
        {loading ? (
          <LoadingIndicator />
        ) : stashes.length === 0 ? (
          <EmptyState />
        ) : (
          stashes.map((entry) => (
            <StashRow
              key={entry.index}
              entry={entry}
              isSelected={selectedIndex === entry.index}
              onClick={() =>
                setSelectedIndex(
                  selectedIndex === entry.index ? null : entry.index,
                )
              }
            />
          ))
        )}
      </div>
    </div>
  );
}
