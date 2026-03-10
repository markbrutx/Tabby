import { GitBranch, GripVertical, X } from "lucide-react";
import { shortenPath } from "@/features/workspace/utils/shortenPath";

interface GitPaneHeaderProps {
  readonly repoPath: string;
  readonly branch: string | null;
  readonly isActive: boolean;
  readonly paneCount: number;
  readonly onClose: () => void;
  readonly draggable?: boolean;
  readonly onDragStart?: React.DragEventHandler;
  readonly onDragOver?: React.DragEventHandler;
  readonly onDragEnter?: React.DragEventHandler;
  readonly onDragLeave?: React.DragEventHandler;
  readonly onDrop?: React.DragEventHandler;
  readonly onDragEnd?: React.DragEventHandler;
  readonly isDragOver?: boolean;
}

function repoBasename(repoPath: string): string {
  const parts = repoPath.replace(/\/+$/, "").split("/");
  return parts[parts.length - 1] ?? repoPath;
}

export function GitPaneHeader({
  repoPath,
  branch,
  isActive,
  paneCount,
  onClose,
  draggable = false,
  onDragStart,
  onDragOver,
  onDragEnter,
  onDragLeave,
  onDrop,
  onDragEnd,
  isDragOver = false,
}: GitPaneHeaderProps) {
  return (
    <div
      className={`flex h-6 shrink-0 select-none items-center gap-1 px-1 ${
        isActive
          ? "border-b border-[var(--color-accent)] bg-[var(--color-surface)] text-[var(--color-text)]"
          : "border-b border-[var(--color-border)] bg-[var(--color-surface)] text-[var(--color-text-muted)]"
      } ${isDragOver ? "ring-2 ring-[var(--color-accent)] ring-inset" : ""}`}
      draggable={draggable}
      onDragStart={onDragStart}
      onDragOver={onDragOver}
      onDragEnter={onDragEnter}
      onDragLeave={onDragLeave}
      onDrop={onDrop}
      onDragEnd={onDragEnd}
      data-testid="git-pane-header"
    >
      <GripVertical
        size={12}
        className="shrink-0 cursor-grab text-[var(--color-text-muted)]"
      />

      <span className="shrink-0 text-xs font-medium" data-testid="git-pane-header-repo">
        {repoBasename(repoPath)}
      </span>

      {branch ? (
        <span
          className="flex shrink-0 items-center gap-0.5 text-xs text-[var(--color-text-muted)]"
          data-testid="git-pane-header-branch"
        >
          <GitBranch size={10} />
          {branch}
        </span>
      ) : null}

      <span
        className="min-w-0 flex-1 truncate text-right text-xs text-[var(--color-text-muted)]"
        title={repoPath}
      >
        {shortenPath(repoPath)}
      </span>

      {paneCount > 1 ? (
        <button
          className="ml-1 flex shrink-0 items-center justify-center rounded p-0.5 text-[var(--color-text-muted)] hover:bg-[var(--color-border)] hover:text-[var(--color-text)]"
          onClick={(e) => {
            e.stopPropagation();
            onClose();
          }}
          data-testid="git-pane-header-close"
        >
          <X size={12} />
        </button>
      ) : null}
    </div>
  );
}
