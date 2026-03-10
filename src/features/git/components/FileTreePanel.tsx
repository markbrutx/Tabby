import { useState, useCallback, useEffect, useRef } from "react";
import type { FileStatus, FileStatusKind } from "@/features/git/domain/models";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface FileTreePanelProps {
  readonly files: readonly FileStatus[];
  readonly selectedFile: string | null;
  readonly onSelectFile: (path: string) => void;
  readonly onStageFiles: (paths: readonly string[]) => void;
  readonly onUnstageFiles: (paths: readonly string[]) => void;
  readonly onDiscardChanges: (paths: readonly string[]) => void;
  readonly onBlameFile?: (path: string) => void;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const STATUS_BADGE_MAP: Record<FileStatusKind, { label: string; color: string }> = {
  modified: { label: "M", color: "text-yellow-400" },
  added: { label: "A", color: "text-green-400" },
  deleted: { label: "D", color: "text-red-400" },
  renamed: { label: "R", color: "text-blue-400" },
  copied: { label: "C", color: "text-blue-400" },
  untracked: { label: "?", color: "text-gray-400" },
  ignored: { label: "!", color: "text-gray-500" },
  conflicted: { label: "U", color: "text-orange-400" },
};

function getStatusBadge(status: FileStatusKind) {
  return STATUS_BADGE_MAP[status] ?? { label: "?", color: "text-gray-400" };
}

function hasUnstagedChanges(file: FileStatus): boolean {
  return (
    file.worktreeStatus === "modified" ||
    file.worktreeStatus === "deleted" ||
    file.worktreeStatus === "untracked"
  );
}

function categorizeStagedFiles(files: readonly FileStatus[]): readonly FileStatus[] {
  return files.filter(
    (f) =>
      f.indexStatus === "modified" ||
      f.indexStatus === "added" ||
      f.indexStatus === "deleted" ||
      f.indexStatus === "renamed" ||
      f.indexStatus === "copied",
  );
}

function categorizeUnstagedFiles(files: readonly FileStatus[]): readonly FileStatus[] {
  return files.filter((f) => hasUnstagedChanges(f));
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

function StatusBadge({ status }: { readonly status: FileStatusKind }) {
  const badge = getStatusBadge(status);
  return (
    <span
      className={`inline-block w-4 text-center font-mono text-xs font-bold ${badge.color}`}
      data-testid="status-badge"
      title={status}
    >
      {badge.label}
    </span>
  );
}

interface FileEntryProps {
  readonly file: FileStatus;
  readonly statusKind: FileStatusKind;
  readonly isSelected: boolean;
  readonly onSelect: (path: string) => void;
  readonly onContextMenu?: (event: React.MouseEvent, path: string) => void;
  readonly actionButtons: React.ReactNode;
}

function FileEntry({ file, statusKind, isSelected, onSelect, onContextMenu, actionButtons }: FileEntryProps) {
  const handleContextMenu = (e: React.MouseEvent) => {
    if (onContextMenu) {
      e.preventDefault();
      onContextMenu(e, file.path);
    }
  };

  return (
    <div
      className={`group flex items-center gap-1 px-2 py-0.5 text-xs transition-colors ${
        isSelected
          ? "bg-[var(--color-accent)]/15 text-[var(--color-text)]"
          : "text-[var(--color-text-soft)] hover:bg-[var(--color-surface-hover)]"
      }`}
      data-testid="file-entry"
      onContextMenu={handleContextMenu}
    >
      <StatusBadge status={statusKind} />
      <button
        type="button"
        className="min-w-0 flex-1 truncate text-left"
        title={file.path}
        onClick={() => onSelect(file.path)}
        data-testid="file-select-button"
      >
        {file.path}
      </button>
      <div className="flex shrink-0 gap-0.5 opacity-0 transition-opacity group-hover:opacity-100">
        {actionButtons}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Context menu
// ---------------------------------------------------------------------------

interface ContextMenuState {
  readonly x: number;
  readonly y: number;
  readonly filePath: string;
}

interface FileContextMenuProps {
  readonly menu: ContextMenuState;
  readonly onBlame: (path: string) => void;
  readonly onClose: () => void;
}

function FileContextMenu({ menu, onBlame, onClose }: FileContextMenuProps) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        onClose();
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [onClose]);

  return (
    <div
      ref={ref}
      className="fixed z-50 min-w-[120px] rounded border border-[var(--color-border)] bg-[var(--color-bg-elevated)] py-1 shadow-lg"
      style={{ left: menu.x, top: menu.y }}
      data-testid="file-context-menu"
    >
      <button
        type="button"
        className="w-full px-3 py-1 text-left text-xs text-[var(--color-text)] hover:bg-[var(--color-surface-hover)]"
        onClick={() => {
          onBlame(menu.filePath);
          onClose();
        }}
        data-testid="context-menu-blame"
      >
        Blame
      </button>
    </div>
  );
}

interface SectionHeaderProps {
  readonly title: string;
  readonly count: number;
  readonly isExpanded: boolean;
  readonly onToggle: () => void;
  readonly batchAction?: React.ReactNode;
}

function SectionHeader({ title, count, isExpanded, onToggle, batchAction }: SectionHeaderProps) {
  return (
    <div className="flex items-center gap-1 border-b border-[var(--color-border)] px-2 py-1">
      <button
        type="button"
        className="flex flex-1 items-center gap-1 text-left text-xs font-medium text-[var(--color-text)]"
        onClick={onToggle}
        data-testid="section-toggle"
      >
        <span className="inline-block w-3 text-center text-[10px]">
          {isExpanded ? "▼" : "▶"}
        </span>
        {title}
        <span className="text-[var(--color-text-soft)]">({count})</span>
      </button>
      {batchAction && (
        <div className="shrink-0">{batchAction}</div>
      )}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Discard confirmation
// ---------------------------------------------------------------------------

interface DiscardConfirmProps {
  readonly filePath: string;
  readonly onConfirm: () => void;
  readonly onCancel: () => void;
}

function DiscardConfirm({ filePath, onConfirm, onCancel }: DiscardConfirmProps) {
  return (
    <div
      className="border-b border-[var(--color-border)] bg-red-900/20 px-2 py-1.5"
      data-testid="discard-confirm"
    >
      <p className="mb-1 text-xs text-red-300">
        Discard changes to <strong>{filePath}</strong>?
      </p>
      <div className="flex gap-1">
        <button
          type="button"
          className="rounded bg-red-600 px-2 py-0.5 text-xs text-white hover:bg-red-500"
          onClick={onConfirm}
          data-testid="discard-confirm-yes"
        >
          Discard
        </button>
        <button
          type="button"
          className="rounded px-2 py-0.5 text-xs text-[var(--color-text-soft)] hover:text-[var(--color-text)]"
          onClick={onCancel}
          data-testid="discard-confirm-no"
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

export function FileTreePanel({
  files,
  selectedFile,
  onSelectFile,
  onStageFiles,
  onUnstageFiles,
  onDiscardChanges,
  onBlameFile,
}: FileTreePanelProps) {
  const [stagedExpanded, setStagedExpanded] = useState(true);
  const [unstagedExpanded, setUnstagedExpanded] = useState(true);
  const [discardTarget, setDiscardTarget] = useState<string | null>(null);
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null);

  const handleContextMenu = useCallback(
    (e: React.MouseEvent, filePath: string) => {
      if (onBlameFile) {
        setContextMenu({ x: e.clientX, y: e.clientY, filePath });
      }
    },
    [onBlameFile],
  );

  const handleCloseContextMenu = useCallback(() => {
    setContextMenu(null);
  }, []);

  const stagedFiles = categorizeStagedFiles(files);
  const unstagedFiles = categorizeUnstagedFiles(files);

  const handleConfirmDiscard = useCallback(() => {
    if (discardTarget !== null) {
      onDiscardChanges([discardTarget]);
      setDiscardTarget(null);
    }
  }, [discardTarget, onDiscardChanges]);

  const handleCancelDiscard = useCallback(() => {
    setDiscardTarget(null);
  }, []);

  if (files.length === 0) {
    return (
      <div
        className="flex h-full items-center justify-center p-3"
        data-testid="file-tree-empty"
      >
        <span className="text-xs text-[var(--color-text-soft)]">
          No changes in working directory
        </span>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col overflow-y-auto" data-testid="file-tree-panel">
      {/* Staged Changes section */}
      <div data-testid="staged-section">
        <SectionHeader
          title="Staged Changes"
          count={stagedFiles.length}
          isExpanded={stagedExpanded}
          onToggle={() => setStagedExpanded((prev) => !prev)}
          batchAction={
            stagedFiles.length > 0 ? (
              <button
                type="button"
                className="text-xs text-[var(--color-text-soft)] hover:text-[var(--color-text)]"
                onClick={() => onUnstageFiles(stagedFiles.map((f) => f.path))}
                title="Unstage All"
                data-testid="unstage-all-button"
              >
                −
              </button>
            ) : null
          }
        />
        {stagedExpanded && stagedFiles.map((file) => (
          <FileEntry
            key={`staged-${file.path}`}
            file={file}
            statusKind={file.indexStatus}
            isSelected={selectedFile === file.path}
            onSelect={onSelectFile}
            onContextMenu={handleContextMenu}
            actionButtons={
              <button
                type="button"
                className="rounded px-1 text-xs text-[var(--color-text)] hover:bg-[var(--color-surface-hover)]"
                onClick={() => onUnstageFiles([file.path])}
                title="Unstage"
                data-testid="unstage-button"
              >
                −
              </button>
            }
          />
        ))}
      </div>

      {/* Unstaged Changes section */}
      <div data-testid="unstaged-section">
        <SectionHeader
          title="Changes"
          count={unstagedFiles.length}
          isExpanded={unstagedExpanded}
          onToggle={() => setUnstagedExpanded((prev) => !prev)}
          batchAction={
            unstagedFiles.length > 0 ? (
              <button
                type="button"
                className="text-xs text-[var(--color-text-soft)] hover:text-[var(--color-text)]"
                onClick={() => onStageFiles(unstagedFiles.map((f) => f.path))}
                title="Stage All"
                data-testid="stage-all-button"
              >
                +
              </button>
            ) : null
          }
        />
        {unstagedExpanded && unstagedFiles.map((file) => (
          <FileEntry
            key={`unstaged-${file.path}`}
            file={file}
            statusKind={file.worktreeStatus}
            isSelected={selectedFile === file.path}
            onSelect={onSelectFile}
            onContextMenu={handleContextMenu}
            actionButtons={
              <>
                <button
                  type="button"
                  className="rounded px-1 text-xs text-[var(--color-text)] hover:bg-[var(--color-surface-hover)]"
                  onClick={() => onStageFiles([file.path])}
                  title="Stage"
                  data-testid="stage-button"
                >
                  +
                </button>
                <button
                  type="button"
                  className="rounded px-1 text-xs text-red-400 hover:bg-red-900/30"
                  onClick={() => setDiscardTarget(file.path)}
                  title="Discard Changes"
                  data-testid="discard-button"
                >
                  🗑
                </button>
              </>
            }
          />
        ))}
      </div>

      {/* Discard confirmation dialog */}
      {discardTarget !== null && (
        <DiscardConfirm
          filePath={discardTarget}
          onConfirm={handleConfirmDiscard}
          onCancel={handleCancelDiscard}
        />
      )}

      {/* Context menu */}
      {contextMenu !== null && onBlameFile && (
        <FileContextMenu
          menu={contextMenu}
          onBlame={onBlameFile}
          onClose={handleCloseContextMenu}
        />
      )}
    </div>
  );
}
