import { useState, useCallback, useMemo } from "react";
import type { BranchInfo } from "@/features/git/domain/models";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface BranchSelectorProps {
  readonly branches: readonly BranchInfo[];
  readonly loading: boolean;
  readonly onCheckout: (name: string) => Promise<void>;
  readonly onCreateBranch: (name: string, startPoint: string | null) => Promise<void>;
  readonly onDeleteBranch: (name: string, force: boolean) => Promise<void>;
  readonly onRefresh: () => Promise<void>;
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

function AheadBehindBadge({ ahead, behind }: { readonly ahead: number; readonly behind: number }) {
  if (ahead === 0 && behind === 0) return null;
  return (
    <span className="ml-1 text-[10px] text-[var(--color-text-soft)]" data-testid="ahead-behind">
      {ahead > 0 && <span className="text-green-400">+{ahead}</span>}
      {ahead > 0 && behind > 0 && " "}
      {behind > 0 && <span className="text-red-400">-{behind}</span>}
    </span>
  );
}

interface DeleteConfirmProps {
  readonly branchName: string;
  readonly onConfirm: (force: boolean) => void;
  readonly onCancel: () => void;
}

function DeleteConfirm({ branchName, onConfirm, onCancel }: DeleteConfirmProps) {
  return (
    <div
      className="border-b border-[var(--color-border)] bg-red-900/20 px-3 py-2"
      data-testid="delete-confirm"
    >
      <p className="mb-1.5 text-xs text-red-300">
        Delete branch <strong>{branchName}</strong>?
      </p>
      <div className="flex gap-1">
        <button
          type="button"
          className="rounded bg-red-600 px-2 py-0.5 text-xs text-white hover:bg-red-500"
          onClick={() => onConfirm(false)}
          data-testid="delete-confirm-yes"
        >
          Delete
        </button>
        <button
          type="button"
          className="rounded bg-red-800 px-2 py-0.5 text-xs text-red-200 hover:bg-red-700"
          onClick={() => onConfirm(true)}
          data-testid="delete-confirm-force"
        >
          Force Delete
        </button>
        <button
          type="button"
          className="rounded px-2 py-0.5 text-xs text-[var(--color-text-soft)] hover:text-[var(--color-text)]"
          onClick={onCancel}
          data-testid="delete-confirm-cancel"
        >
          Cancel
        </button>
      </div>
    </div>
  );
}

interface CreateBranchFormProps {
  readonly onSubmit: (name: string, startPoint: string | null) => void;
  readonly onCancel: () => void;
}

function CreateBranchForm({ onSubmit, onCancel }: CreateBranchFormProps) {
  const [name, setName] = useState("");
  const [startPoint, setStartPoint] = useState("");

  const handleSubmit = useCallback(() => {
    const trimmed = name.trim();
    if (trimmed.length === 0) return;
    onSubmit(trimmed, startPoint.trim() || null);
  }, [name, startPoint, onSubmit]);

  return (
    <div
      className="border-b border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2"
      data-testid="create-branch-form"
    >
      <div className="mb-1.5 text-xs font-medium text-[var(--color-text)]">New Branch</div>
      <input
        type="text"
        className="mb-1 w-full rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 py-1 text-xs text-[var(--color-text)] placeholder:text-[var(--color-text-soft)] focus:border-[var(--color-accent)] focus:outline-none"
        placeholder="Branch name"
        value={name}
        onChange={(e) => setName(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") handleSubmit();
          if (e.key === "Escape") onCancel();
        }}
        data-testid="create-branch-name"
        autoFocus
      />
      <input
        type="text"
        className="mb-1.5 w-full rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 py-1 text-xs text-[var(--color-text)] placeholder:text-[var(--color-text-soft)] focus:border-[var(--color-accent)] focus:outline-none"
        placeholder="Start point (optional, default: HEAD)"
        value={startPoint}
        onChange={(e) => setStartPoint(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") handleSubmit();
          if (e.key === "Escape") onCancel();
        }}
        data-testid="create-branch-start-point"
      />
      <div className="flex gap-1">
        <button
          type="button"
          className="rounded bg-[var(--color-accent)] px-2 py-0.5 text-xs text-white hover:opacity-90 disabled:opacity-40"
          onClick={handleSubmit}
          disabled={name.trim().length === 0}
          data-testid="create-branch-submit"
        >
          Create & Switch
        </button>
        <button
          type="button"
          className="rounded px-2 py-0.5 text-xs text-[var(--color-text-soft)] hover:text-[var(--color-text)]"
          onClick={onCancel}
          data-testid="create-branch-cancel"
        >
          Cancel
        </button>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

export function BranchSelector({
  branches,
  loading,
  onCheckout,
  onCreateBranch,
  onDeleteBranch,
  onRefresh,
}: BranchSelectorProps) {
  const [searchQuery, setSearchQuery] = useState("");
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<string | null>(null);
  const [actionLoading, setActionLoading] = useState(false);

  const currentBranch = useMemo(
    () => branches.find((b) => b.isCurrent) ?? null,
    [branches],
  );

  const filteredBranches = useMemo(() => {
    if (searchQuery.trim().length === 0) return branches;
    const query = searchQuery.toLowerCase();
    return branches.filter((b) => b.name.toLowerCase().includes(query));
  }, [branches, searchQuery]);

  const handleCheckout = useCallback(
    async (name: string) => {
      setActionLoading(true);
      try {
        await onCheckout(name);
      } finally {
        setActionLoading(false);
      }
    },
    [onCheckout],
  );

  const handleCreate = useCallback(
    async (name: string, startPoint: string | null) => {
      setActionLoading(true);
      try {
        await onCreateBranch(name, startPoint);
        setShowCreateForm(false);
      } finally {
        setActionLoading(false);
      }
    },
    [onCreateBranch],
  );

  const handleDelete = useCallback(
    async (force: boolean) => {
      if (deleteTarget === null) return;
      setActionLoading(true);
      try {
        await onDeleteBranch(deleteTarget, force);
        setDeleteTarget(null);
      } finally {
        setActionLoading(false);
      }
    },
    [deleteTarget, onDeleteBranch],
  );

  if (loading) {
    return (
      <div
        className="flex h-full items-center justify-center"
        data-testid="branch-loading"
      >
        <div className="flex flex-col items-center gap-2">
          <div className="h-5 w-5 animate-spin rounded-full border-2 border-[var(--color-border)] border-t-[var(--color-accent)]" />
          <span className="text-xs text-[var(--color-text-soft)]">Loading branches...</span>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col" data-testid="branch-selector">
      {/* Header with current branch and actions */}
      <div className="flex items-center gap-2 border-b border-[var(--color-border)] px-3 py-2">
        <div className="min-w-0 flex-1">
          <div className="text-[10px] text-[var(--color-text-soft)]">Current Branch</div>
          <div className="truncate text-xs font-medium text-[var(--color-text)]" data-testid="current-branch-name">
            {currentBranch?.name ?? "HEAD (detached)"}
          </div>
        </div>
        <button
          type="button"
          className="rounded px-1.5 py-0.5 text-xs text-[var(--color-text)] hover:bg-[var(--color-surface-hover)]"
          onClick={() => setShowCreateForm((prev) => !prev)}
          title="Create Branch"
          data-testid="create-branch-button"
        >
          +
        </button>
        <button
          type="button"
          className="rounded px-1.5 py-0.5 text-xs text-[var(--color-text)] hover:bg-[var(--color-surface-hover)]"
          onClick={() => void onRefresh()}
          title="Refresh"
          data-testid="refresh-branches-button"
        >
          ↻
        </button>
      </div>

      {/* Create branch form */}
      {showCreateForm && (
        <CreateBranchForm
          onSubmit={(name, startPoint) => void handleCreate(name, startPoint)}
          onCancel={() => setShowCreateForm(false)}
        />
      )}

      {/* Delete confirmation */}
      {deleteTarget !== null && (
        <DeleteConfirm
          branchName={deleteTarget}
          onConfirm={(force) => void handleDelete(force)}
          onCancel={() => setDeleteTarget(null)}
        />
      )}

      {/* Search input */}
      <div className="border-b border-[var(--color-border)] px-3 py-1.5">
        <input
          type="text"
          className="w-full rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-2 py-1 text-xs text-[var(--color-text)] placeholder:text-[var(--color-text-soft)] focus:border-[var(--color-accent)] focus:outline-none"
          placeholder="Filter branches..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          data-testid="branch-search"
        />
      </div>

      {/* Branch list */}
      <div className="flex-1 overflow-y-auto" data-testid="branch-list">
        {filteredBranches.length === 0 && (
          <div className="px-3 py-4 text-center text-xs text-[var(--color-text-soft)]" data-testid="branch-list-empty">
            {searchQuery.trim().length > 0 ? "No branches match your filter" : "No branches found"}
          </div>
        )}
        {filteredBranches.map((branch) => (
          <div
            key={branch.name}
            className={`group flex items-center gap-1 px-3 py-1 text-xs transition-colors ${
              branch.isCurrent
                ? "bg-[var(--color-accent)]/15 text-[var(--color-text)]"
                : "text-[var(--color-text-soft)] hover:bg-[var(--color-surface-hover)]"
            }`}
            data-testid="branch-item"
          >
            {/* Current branch indicator */}
            <span className="inline-block w-3 text-center text-[10px]">
              {branch.isCurrent ? "●" : ""}
            </span>

            {/* Branch name — clickable to checkout */}
            <button
              type="button"
              className="min-w-0 flex-1 truncate text-left"
              title={branch.isCurrent ? "Current branch" : `Switch to ${branch.name}`}
              onClick={() => {
                if (!branch.isCurrent && !actionLoading) {
                  void handleCheckout(branch.name);
                }
              }}
              disabled={branch.isCurrent || actionLoading}
              data-testid="branch-checkout-button"
            >
              {branch.name}
            </button>

            {/* Upstream info */}
            {branch.upstream !== null && (
              <span className="shrink-0 text-[10px] text-[var(--color-text-soft)]" data-testid="branch-upstream" title={branch.upstream}>
                ↕
              </span>
            )}

            {/* Ahead/behind */}
            <AheadBehindBadge ahead={branch.ahead} behind={branch.behind} />

            {/* Delete button — hidden for current branch */}
            {!branch.isCurrent && (
              <button
                type="button"
                className="shrink-0 rounded px-1 text-xs text-red-400 opacity-0 transition-opacity hover:bg-red-900/30 group-hover:opacity-100"
                onClick={() => setDeleteTarget(branch.name)}
                title={`Delete ${branch.name}`}
                disabled={actionLoading}
                data-testid="branch-delete-button"
              >
                ×
              </button>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
