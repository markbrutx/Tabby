import { GitBranch, GripVertical, RefreshCw, X } from "lucide-react";
import { shortenPath } from "@/features/workspace/utils/shortenPath";

interface PaneHeaderProps {
  profileLabel: string;
  cwd: string;
  isActive: boolean;
  paneCount: number;
  onClose: () => void;
  onRestart: () => void;
  onOpenGitView?: () => void;
  draggable?: boolean;
  onDragStart?: React.DragEventHandler;
  onDragEnd?: React.DragEventHandler;
}

export function PaneHeader({
  profileLabel,
  cwd,
  isActive,
  paneCount,
  onClose,
  onRestart,
  onOpenGitView,
  draggable = false,
  onDragStart,
  onDragEnd,
}: PaneHeaderProps) {
  return (
    <div
      className={`flex h-6 shrink-0 select-none items-center gap-1 px-1 ${
        isActive
          ? "border-b border-[var(--color-accent)] bg-[var(--color-surface)] text-[var(--color-text)]"
          : "border-b border-[var(--color-border)] bg-[var(--color-surface)] text-[var(--color-text-muted)]"
      }`}
      draggable={draggable}
      onDragStart={onDragStart}
      onDragEnd={onDragEnd}
      data-testid="pane-header"
    >
      <GripVertical
        size={12}
        className="shrink-0 cursor-grab text-[var(--color-text-muted)]"
      />

      <span className="shrink-0 text-xs font-medium" data-testid="pane-header-profile">
        {profileLabel}
      </span>

      <span
        className="min-w-0 flex-1 truncate text-right text-xs text-[var(--color-text-muted)]"
        data-testid="pane-header-cwd"
        title={cwd}
      >
        {shortenPath(cwd)}
      </span>

      {onOpenGitView && cwd ? (
        <button
          className="ml-1 flex shrink-0 items-center justify-center rounded p-0.5 text-[var(--color-text-muted)] hover:bg-[var(--color-border)] hover:text-[var(--color-text)]"
          onClick={(e) => {
            e.stopPropagation();
            onOpenGitView();
          }}
          title="Open Git View"
          data-testid="pane-header-open-git"
        >
          <GitBranch size={12} />
        </button>
      ) : null}

      <button
        className="ml-1 flex shrink-0 items-center justify-center rounded p-0.5 text-[var(--color-text-muted)] hover:bg-[var(--color-border)] hover:text-[var(--color-text)]"
        onClick={(e) => {
          e.stopPropagation();
          onRestart();
        }}
        title="Restart (⌘⇧R)"
        data-testid="pane-header-restart"
      >
        <RefreshCw size={12} />
      </button>

      {paneCount > 1 ? (
        <button
          className="ml-1 flex shrink-0 items-center justify-center rounded p-0.5 text-[var(--color-text-muted)] hover:bg-[var(--color-border)] hover:text-[var(--color-text)]"
          onClick={(e) => {
            e.stopPropagation();
            onClose();
          }}
          data-testid="pane-header-close"
        >
          <X size={12} />
        </button>
      ) : null}
    </div>
  );
}
