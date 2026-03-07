import { useCallback, useRef, useState } from "react";
import { ArrowLeft, ArrowRight, GripVertical, RotateCw, X } from "lucide-react";
import { DEFAULT_BROWSER_URL } from "@/features/workspace/domain";
import { shortenPath } from "@/features/workspace/utils/shortenPath";

interface PaneHeaderProps {
  profileLabel: string;
  cwd: string;
  isActive: boolean;
  paneCount: number;
  onClose: () => void;
  draggable?: boolean;
  onDragStart?: React.DragEventHandler;
  onDragOver?: React.DragEventHandler;
  onDragEnter?: React.DragEventHandler;
  onDragLeave?: React.DragEventHandler;
  onDrop?: React.DragEventHandler;
  onDragEnd?: React.DragEventHandler;
  isDragOver?: boolean;
  isBrowser?: boolean;
  browserUrl?: string;
  onBrowserNavigate?: (url: string) => void;
  onBrowserReload?: () => void;
}

export function PaneHeader({
  profileLabel,
  cwd,
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
  isBrowser = false,
  browserUrl = "",
  onBrowserNavigate,
  onBrowserReload,
}: PaneHeaderProps) {
  const [urlInput, setUrlInput] = useState(browserUrl);
  const prevBrowserUrlRef = useRef(browserUrl);

  // Sync external URL changes (e.g. from native navigation) into the input
  if (browserUrl !== prevBrowserUrlRef.current) {
    prevBrowserUrlRef.current = browserUrl;
    setUrlInput(browserUrl);
  }

  const handleNavigate = useCallback(() => {
    const trimmed = urlInput.trim();
    if (!trimmed) return;
    onBrowserNavigate?.(trimmed);
  }, [urlInput, onBrowserNavigate]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter") {
        handleNavigate();
      }
    },
    [handleNavigate],
  );

  const heightClass = isBrowser ? "h-8" : "h-6";

  return (
    <div
      className={`flex ${heightClass} shrink-0 select-none items-center gap-1 px-1 ${
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
      data-testid="pane-header"
    >
      <GripVertical
        size={12}
        className="shrink-0 cursor-grab text-[var(--color-text-muted)]"
      />

      {isBrowser ? (
        <>
          <button
            type="button"
            className="rounded p-0.5 text-[var(--color-text-muted)] hover:bg-[var(--color-border)] hover:text-[var(--color-text)]"
            title="Back"
            data-testid="browser-back-btn"
            disabled
          >
            <ArrowLeft size={12} />
          </button>
          <button
            type="button"
            className="rounded p-0.5 text-[var(--color-text-muted)] hover:bg-[var(--color-border)] hover:text-[var(--color-text)]"
            title="Forward"
            data-testid="browser-forward-btn"
            disabled
          >
            <ArrowRight size={12} />
          </button>
          <button
            type="button"
            className="rounded p-0.5 text-[var(--color-text-muted)] hover:bg-[var(--color-border)] hover:text-[var(--color-text)]"
            title="Reload"
            onClick={(e) => {
              e.stopPropagation();
              onBrowserReload?.();
            }}
            data-testid="browser-reload-btn"
          >
            <RotateCw size={12} />
          </button>
          <input
            type="text"
            value={urlInput}
            onChange={(e) => setUrlInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={DEFAULT_BROWSER_URL}
            className="mx-0.5 h-5 min-w-0 flex-1 rounded border border-[var(--color-border)] bg-[var(--color-bg)] px-1.5 text-xs text-[var(--color-text)] placeholder-[var(--color-text-muted)] outline-none focus:border-[var(--color-accent)]"
            data-testid="browser-url-input"
          />
          <button
            type="button"
            className="rounded bg-[var(--color-accent)] px-1.5 py-0.5 text-xs text-white hover:opacity-90"
            onClick={(e) => {
              e.stopPropagation();
              handleNavigate();
            }}
            data-testid="browser-go-btn"
          >
            Go
          </button>
        </>
      ) : (
        <>
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
        </>
      )}

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
