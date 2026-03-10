import { useState, useCallback, useEffect, useMemo } from "react";
import { detectLanguage } from "./syntaxHighlight";
import { flattenHunks, buildSplitRows } from "./diff/diffTypes";
import type { DiffViewMode, DiffViewerProps } from "./diff/diffTypes";
import { UnifiedRenderer } from "./diff/UnifiedRenderer";
import { SplitRenderer } from "./diff/SplitRenderer";

// Re-export types for existing consumers
export type { StagingCallbacks } from "./diff/diffTypes";

// ---------------------------------------------------------------------------
// Small presentational sub-components
// ---------------------------------------------------------------------------

function EmptyState() {
  return (
    <div
      className="flex h-full items-center justify-center"
      data-testid="diff-empty"
    >
      <span className="text-xs text-[var(--color-text-soft)]">
        No diff to display
      </span>
    </div>
  );
}

function BinaryIndicator({ filePath }: { readonly filePath: string }) {
  return (
    <div
      className="flex h-full items-center justify-center"
      data-testid="diff-binary"
    >
      <div className="flex flex-col items-center gap-1">
        <span className="text-xs font-medium text-[var(--color-text)]">
          Binary file
        </span>
        <span className="text-xs text-[var(--color-text-soft)]">
          {filePath}
        </span>
      </div>
    </div>
  );
}

function FileModeChange({ mode }: { readonly mode: string }) {
  return (
    <div
      className="border-b border-[var(--color-border)] bg-yellow-900/15 px-3 py-1"
      data-testid="diff-mode-change"
    >
      <span className="font-mono text-xs text-yellow-300">
        File mode changed: {mode}
      </span>
    </div>
  );
}

function ModeToggle({ mode, onToggle }: { readonly mode: DiffViewMode; readonly onToggle: () => void }) {
  return (
    <div
      className="flex items-center border-b border-[var(--color-border)] bg-[var(--color-surface-elevated)] px-3 py-1"
      data-testid="diff-mode-header"
    >
      <button
        type="button"
        className="rounded px-2 py-0.5 text-xs text-[var(--color-text)] hover:bg-[var(--color-surface-hover)]"
        onClick={onToggle}
        data-testid="diff-mode-toggle"
      >
        {mode === "unified" ? "Split" : "Unified"}
      </button>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

export function DiffViewer({ diffContent, mode: initialMode, staging, stagedLines }: DiffViewerProps) {
  const [mode, setMode] = useState<DiffViewMode>(initialMode ?? "unified");

  useEffect(() => {
    if (initialMode !== undefined) {
      setMode(initialMode);
    }
  }, [initialMode]);

  const unifiedRows = useMemo(
    () => (diffContent ? flattenHunks(diffContent.hunks) : []),
    [diffContent],
  );

  const splitPairs = useMemo(
    () => (diffContent ? buildSplitRows(diffContent.hunks) : []),
    [diffContent],
  );

  const language = useMemo(
    () => (diffContent ? detectLanguage(diffContent.filePath) : null),
    [diffContent],
  );

  const handleToggle = useCallback(() => {
    setMode((prev) => (prev === "unified" ? "split" : "unified"));
  }, []);

  if (diffContent === null) {
    return <EmptyState />;
  }

  if (diffContent.isBinary) {
    return <BinaryIndicator filePath={diffContent.filePath} />;
  }

  const hasNoLines = unifiedRows.length === 0;

  return (
    <div className="flex h-full flex-col" data-testid="diff-viewer">
      <ModeToggle mode={mode} onToggle={handleToggle} />

      {diffContent.fileModeChange !== null && (
        <FileModeChange mode={diffContent.fileModeChange} />
      )}

      {hasNoLines ? (
        <EmptyState />
      ) : mode === "unified" ? (
        <UnifiedRenderer
          rows={unifiedRows}
          filePath={diffContent.filePath}
          hunks={diffContent.hunks}
          language={language}
          staging={staging}
          stagedLines={stagedLines}
        />
      ) : (
        <SplitRenderer
          pairs={splitPairs}
          filePath={diffContent.filePath}
          hunks={diffContent.hunks}
          language={language}
          staging={staging}
          stagedLines={stagedLines}
        />
      )}
    </div>
  );
}
