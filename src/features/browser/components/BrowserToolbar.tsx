import { useCallback, useRef, useState } from "react";
import { ArrowLeft, ArrowRight, GripVertical, RotateCw, X } from "lucide-react";
import { DEFAULT_BROWSER_URL } from "@/features/workspace/domain";
import { normalizeUrl } from "@/features/browser/hooks/useBrowserWebview";

interface BrowserToolbarProps {
  url: string;
  isActive: boolean;
  paneCount: number;
  onNavigate: (url: string) => void;
  onReload: () => void;
  onClose: () => void;
  draggable?: boolean;
  onDragStart?: React.DragEventHandler;
  onDragOver?: React.DragEventHandler;
  onDragEnter?: React.DragEventHandler;
  onDragLeave?: React.DragEventHandler;
  onDrop?: React.DragEventHandler;
  onDragEnd?: React.DragEventHandler;
  isDragOver?: boolean;
}

export function BrowserToolbar({
  url,
  isActive,
  paneCount,
  onNavigate,
  onReload,
  onClose,
  draggable = false,
  onDragStart,
  onDragOver,
  onDragEnter,
  onDragLeave,
  onDrop,
  onDragEnd,
  isDragOver = false,
}: BrowserToolbarProps) {
  const [urlInput, setUrlInput] = useState(() => normalizeUrl(url));
  const prevUrlRef = useRef(url);

  if (url !== prevUrlRef.current) {
    prevUrlRef.current = url;
    setUrlInput(normalizeUrl(url));
  }

  const handleNavigate = useCallback(() => {
    const trimmed = urlInput.trim();
    if (!trimmed) return;
    onNavigate(trimmed);
  }, [urlInput, onNavigate]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter") {
        handleNavigate();
      }
    },
    [handleNavigate],
  );

  return (
    <div
      className={`flex h-9 shrink-0 select-none items-center gap-1.5 px-2 ${
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
      data-testid="browser-toolbar"
    >
      <GripVertical
        size={12}
        className="shrink-0 cursor-grab text-[var(--color-text-muted)]"
      />

      <button
        type="button"
        className="rounded p-0.5 text-[var(--color-text-muted)] hover:bg-[var(--color-border)] hover:text-[var(--color-text)]"
        title="Back"
        data-testid="browser-back-btn"
        disabled
      >
        <ArrowLeft size={13} />
      </button>
      <button
        type="button"
        className="rounded p-0.5 text-[var(--color-text-muted)] hover:bg-[var(--color-border)] hover:text-[var(--color-text)]"
        title="Forward"
        data-testid="browser-forward-btn"
        disabled
      >
        <ArrowRight size={13} />
      </button>
      <button
        type="button"
        className="rounded p-0.5 text-[var(--color-text-muted)] hover:bg-[var(--color-border)] hover:text-[var(--color-text)]"
        title="Reload"
        onClick={(e) => {
          e.stopPropagation();
          onReload();
        }}
        data-testid="browser-reload-btn"
      >
        <RotateCw size={13} />
      </button>

      <input
        type="text"
        value={urlInput}
        onChange={(e) => setUrlInput(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder={DEFAULT_BROWSER_URL}
        className="mx-1 h-6 min-w-0 flex-1 rounded-md border border-[var(--color-border)] bg-[var(--color-bg)] px-2 text-xs text-[var(--color-text)] placeholder-[var(--color-text-muted)] outline-none focus:border-[var(--color-accent)]"
        data-testid="browser-url-input"
      />

      <button
        type="button"
        className="rounded-md bg-[var(--color-accent)] px-2 py-0.5 text-xs font-medium text-white hover:opacity-90"
        onClick={(e) => {
          e.stopPropagation();
          handleNavigate();
        }}
        data-testid="browser-go-btn"
      >
        Go
      </button>

      {paneCount > 1 ? (
        <button
          className="ml-0.5 flex shrink-0 items-center justify-center rounded p-0.5 text-[var(--color-text-muted)] hover:bg-[var(--color-border)] hover:text-[var(--color-text)]"
          onClick={(e) => {
            e.stopPropagation();
            onClose();
          }}
          data-testid="browser-toolbar-close"
        >
          <X size={13} />
        </button>
      ) : null}
    </div>
  );
}
