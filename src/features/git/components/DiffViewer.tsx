import { useRef, useState, useCallback, useEffect, useMemo } from "react";
import type { DiffContent, DiffHunk, DiffLine } from "@/features/git/domain/models";

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const LINE_HEIGHT_PX = 20;
const OVERSCAN_COUNT = 10;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type DiffViewMode = "unified" | "split";

export interface DiffViewerProps {
  readonly diffContent: DiffContent | null;
  readonly mode?: DiffViewMode;
}

// ---------------------------------------------------------------------------
// Helpers — flatten hunks into renderable rows (unified mode)
// ---------------------------------------------------------------------------

interface HunkHeaderRow {
  readonly type: "hunkHeader";
  readonly header: string;
}

interface DiffLineRow {
  readonly type: "line";
  readonly line: DiffLine;
}

type DiffRow = HunkHeaderRow | DiffLineRow;

function flattenHunks(hunks: readonly DiffHunk[]): readonly DiffRow[] {
  const rows: DiffRow[] = [];
  for (const hunk of hunks) {
    rows.push({ type: "hunkHeader", header: hunk.header });
    for (const line of hunk.lines) {
      rows.push({ type: "line", line });
    }
  }
  return rows;
}

// ---------------------------------------------------------------------------
// Helpers — split mode: build aligned left/right rows
// ---------------------------------------------------------------------------

interface SplitHunkHeaderRow {
  readonly type: "hunkHeader";
  readonly header: string;
}

interface SplitLineRow {
  readonly type: "line";
  readonly lineNo: number | null;
  readonly content: string;
  readonly kind: "context" | "addition" | "deletion" | "blank";
}

type SplitRow = SplitHunkHeaderRow | SplitLineRow;

interface SplitPair {
  readonly left: SplitRow;
  readonly right: SplitRow;
}

function buildSplitRows(hunks: readonly DiffHunk[]): readonly SplitPair[] {
  const pairs: SplitPair[] = [];

  for (const hunk of hunks) {
    const headerRow: SplitHunkHeaderRow = { type: "hunkHeader", header: hunk.header };
    pairs.push({ left: headerRow, right: headerRow });

    const lines = hunk.lines;
    let i = 0;
    while (i < lines.length) {
      const line = lines[i];

      if (line.kind === "context") {
        pairs.push({
          left: { type: "line", lineNo: line.oldLineNo, content: line.content, kind: "context" },
          right: { type: "line", lineNo: line.newLineNo, content: line.content, kind: "context" },
        });
        i++;
      } else if (line.kind === "deletion") {
        // Collect consecutive deletions then pair with consecutive additions
        const deletions: DiffLine[] = [];
        while (i < lines.length && lines[i].kind === "deletion") {
          deletions.push(lines[i]);
          i++;
        }
        const additions: DiffLine[] = [];
        while (i < lines.length && lines[i].kind === "addition") {
          additions.push(lines[i]);
          i++;
        }

        const maxLen = Math.max(deletions.length, additions.length);
        for (let j = 0; j < maxLen; j++) {
          const del = deletions[j];
          const add = additions[j];
          pairs.push({
            left: del
              ? { type: "line", lineNo: del.oldLineNo, content: del.content, kind: "deletion" }
              : { type: "line", lineNo: null, content: "", kind: "blank" },
            right: add
              ? { type: "line", lineNo: add.newLineNo, content: add.content, kind: "addition" }
              : { type: "line", lineNo: null, content: "", kind: "blank" },
          });
        }
      } else if (line.kind === "addition") {
        // Standalone addition (no preceding deletion)
        pairs.push({
          left: { type: "line", lineNo: null, content: "", kind: "blank" },
          right: { type: "line", lineNo: line.newLineNo, content: line.content, kind: "addition" },
        });
        i++;
      } else {
        // hunkHeader lines shouldn't appear here, skip
        i++;
      }
    }
  }

  return pairs;
}

// ---------------------------------------------------------------------------
// Line styling
// ---------------------------------------------------------------------------

function getLineClassName(kind: DiffLine["kind"]): string {
  switch (kind) {
    case "addition":
      return "bg-green-900/25 text-green-300";
    case "deletion":
      return "bg-red-900/25 text-red-300";
    case "hunkHeader":
      return "bg-blue-900/20 text-blue-300";
    default:
      return "text-[var(--color-text)]";
  }
}

function getSplitLineClassName(kind: SplitLineRow["kind"]): string {
  switch (kind) {
    case "addition":
      return "bg-green-900/25 text-green-300";
    case "deletion":
      return "bg-red-900/25 text-red-300";
    case "blank":
      return "bg-[var(--color-surface)]";
    default:
      return "text-[var(--color-text)]";
  }
}

function formatLineNo(n: number | null): string {
  if (n === null) return "";
  return String(n);
}

// ---------------------------------------------------------------------------
// Sub-components
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

function HunkHeaderCell({ header }: { readonly header: string }) {
  return (
    <div
      className="flex bg-blue-900/20"
      data-testid="hunk-header"
    >
      <span className="w-[100px] shrink-0 border-r border-[var(--color-border)] bg-blue-900/10 px-1 text-right text-[var(--color-text-soft)]">
        ···
      </span>
      <span className="flex-1 px-2 text-blue-300">
        {header}
      </span>
    </div>
  );
}

interface DiffLineCellProps {
  readonly line: DiffLine;
}

function DiffLineCell({ line }: DiffLineCellProps) {
  const lineClass = getLineClassName(line.kind);
  const gutterBg =
    line.kind === "addition"
      ? "bg-green-900/15"
      : line.kind === "deletion"
        ? "bg-red-900/15"
        : "";

  return (
    <div className={`flex ${lineClass}`} data-testid="diff-line">
      <span
        className={`w-[50px] shrink-0 select-none border-r border-[var(--color-border)] px-1 text-right text-[var(--color-text-soft)] ${gutterBg}`}
        data-testid="line-no-old"
      >
        {formatLineNo(line.oldLineNo)}
      </span>
      <span
        className={`w-[50px] shrink-0 select-none border-r border-[var(--color-border)] px-1 text-right text-[var(--color-text-soft)] ${gutterBg}`}
        data-testid="line-no-new"
      >
        {formatLineNo(line.newLineNo)}
      </span>
      <span className="flex-1 whitespace-pre px-2" data-testid="line-content">
        {line.content}
      </span>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Split mode sub-components
// ---------------------------------------------------------------------------

function SplitHunkHeader({ header }: { readonly header: string }) {
  return (
    <div
      className="flex bg-blue-900/20"
      data-testid="split-hunk-header"
    >
      <span className="w-[50px] shrink-0 border-r border-[var(--color-border)] bg-blue-900/10 px-1 text-right text-[var(--color-text-soft)]">
        ···
      </span>
      <span className="flex-1 px-2 text-blue-300">
        {header}
      </span>
    </div>
  );
}

function SplitLineCell({ row }: { readonly row: SplitLineRow }) {
  const lineClass = getSplitLineClassName(row.kind);
  const gutterBg =
    row.kind === "addition"
      ? "bg-green-900/15"
      : row.kind === "deletion"
        ? "bg-red-900/15"
        : "";

  return (
    <div className={`flex ${lineClass}`} data-testid="split-line">
      <span
        className={`w-[50px] shrink-0 select-none border-r border-[var(--color-border)] px-1 text-right text-[var(--color-text-soft)] ${gutterBg}`}
        data-testid="split-line-no"
      >
        {formatLineNo(row.lineNo)}
      </span>
      <span className="flex-1 whitespace-pre px-2" data-testid="split-line-content">
        {row.content}
      </span>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Mode toggle button
// ---------------------------------------------------------------------------

interface ModeToggleProps {
  readonly mode: DiffViewMode;
  readonly onToggle: () => void;
}

function ModeToggle({ mode, onToggle }: ModeToggleProps) {
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
// Virtual scrolling hook
// ---------------------------------------------------------------------------

function useVirtualScroll(totalRows: number, containerRef: React.RefObject<HTMLDivElement | null>) {
  const [scrollTop, setScrollTop] = useState(0);
  const [containerHeight, setContainerHeight] = useState(0);

  const handleScroll = useCallback(() => {
    const el = containerRef.current;
    if (el) {
      setScrollTop(el.scrollTop);
    }
  }, [containerRef]);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        setContainerHeight(entry.contentRect.height);
      }
    });
    observer.observe(el);
    setContainerHeight(el.clientHeight);

    return () => observer.disconnect();
  }, [containerRef]);

  const totalHeight = totalRows * LINE_HEIGHT_PX;
  const startIdx = Math.max(0, Math.floor(scrollTop / LINE_HEIGHT_PX) - OVERSCAN_COUNT);
  const visibleCount = Math.ceil(containerHeight / LINE_HEIGHT_PX) + 2 * OVERSCAN_COUNT;
  const endIdx = Math.min(totalRows, startIdx + visibleCount);

  return { handleScroll, totalHeight, startIdx, endIdx };
}

// ---------------------------------------------------------------------------
// Synchronized scroll hook for split mode
// ---------------------------------------------------------------------------

function useSyncScroll(
  leftRef: React.RefObject<HTMLDivElement | null>,
  rightRef: React.RefObject<HTMLDivElement | null>,
) {
  const scrollingRef = useRef<"left" | "right" | null>(null);

  const syncScroll = useCallback(
    (source: "left" | "right") => {
      if (scrollingRef.current !== null && scrollingRef.current !== source) return;

      scrollingRef.current = source;

      const sourceEl = source === "left" ? leftRef.current : rightRef.current;
      const targetEl = source === "left" ? rightRef.current : leftRef.current;

      if (sourceEl && targetEl) {
        targetEl.scrollTop = sourceEl.scrollTop;
      }

      requestAnimationFrame(() => {
        scrollingRef.current = null;
      });
    },
    [leftRef, rightRef],
  );

  const onLeftScroll = useCallback(() => syncScroll("left"), [syncScroll]);
  const onRightScroll = useCallback(() => syncScroll("right"), [syncScroll]);

  return { onLeftScroll, onRightScroll };
}

// ---------------------------------------------------------------------------
// Unified mode renderer
// ---------------------------------------------------------------------------

interface UnifiedRendererProps {
  readonly rows: readonly DiffRow[];
}

function UnifiedRenderer({ rows }: UnifiedRendererProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const { handleScroll, totalHeight, startIdx, endIdx } = useVirtualScroll(
    rows.length,
    containerRef,
  );

  return (
    <div
      ref={containerRef}
      className="flex-1 overflow-auto font-mono text-xs leading-5"
      onScroll={handleScroll}
      data-testid="diff-scroll-container"
    >
      <div style={{ height: totalHeight, position: "relative" }}>
        {rows.slice(startIdx, endIdx).map((row, i) => {
          const actualIdx = startIdx + i;
          return (
            <div
              key={actualIdx}
              style={{
                position: "absolute",
                top: actualIdx * LINE_HEIGHT_PX,
                left: 0,
                right: 0,
                height: LINE_HEIGHT_PX,
              }}
            >
              {row.type === "hunkHeader" ? (
                <HunkHeaderCell header={row.header} />
              ) : (
                <DiffLineCell line={row.line} />
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Split mode renderer
// ---------------------------------------------------------------------------

interface SplitRendererProps {
  readonly pairs: readonly SplitPair[];
}

function SplitRenderer({ pairs }: SplitRendererProps) {
  const leftRef = useRef<HTMLDivElement | null>(null);
  const rightRef = useRef<HTMLDivElement | null>(null);
  const { onLeftScroll, onRightScroll } = useSyncScroll(leftRef, rightRef);

  const leftVirtual = useVirtualScroll(pairs.length, leftRef);
  const totalHeight = pairs.length * LINE_HEIGHT_PX;

  const startIdx = leftVirtual.startIdx;
  const endIdx = leftVirtual.endIdx;

  const handleLeftScroll = useCallback(() => {
    leftVirtual.handleScroll();
    onLeftScroll();
  }, [leftVirtual, onLeftScroll]);

  const handleRightScroll = useCallback(() => {
    onRightScroll();
  }, [onRightScroll]);

  return (
    <div className="flex flex-1 min-h-0" data-testid="split-container">
      {/* Left panel — old file */}
      <div
        ref={leftRef}
        className="flex-1 overflow-auto border-r border-[var(--color-border)] font-mono text-xs leading-5"
        onScroll={handleLeftScroll}
        data-testid="split-left"
      >
        <div style={{ height: totalHeight, position: "relative" }}>
          {pairs.slice(startIdx, endIdx).map((pair, i) => {
            const actualIdx = startIdx + i;
            return (
              <div
                key={actualIdx}
                style={{
                  position: "absolute",
                  top: actualIdx * LINE_HEIGHT_PX,
                  left: 0,
                  right: 0,
                  height: LINE_HEIGHT_PX,
                }}
              >
                {pair.left.type === "hunkHeader" ? (
                  <SplitHunkHeader header={pair.left.header} />
                ) : (
                  <SplitLineCell row={pair.left} />
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Right panel — new file */}
      <div
        ref={rightRef}
        className="flex-1 overflow-auto font-mono text-xs leading-5"
        onScroll={handleRightScroll}
        data-testid="split-right"
      >
        <div style={{ height: totalHeight, position: "relative" }}>
          {pairs.slice(startIdx, endIdx).map((pair, i) => {
            const actualIdx = startIdx + i;
            return (
              <div
                key={actualIdx}
                style={{
                  position: "absolute",
                  top: actualIdx * LINE_HEIGHT_PX,
                  left: 0,
                  right: 0,
                  height: LINE_HEIGHT_PX,
                }}
              >
                {pair.right.type === "hunkHeader" ? (
                  <SplitHunkHeader header={pair.right.header} />
                ) : (
                  <SplitLineCell row={pair.right} />
                )}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

export function DiffViewer({ diffContent, mode: initialMode }: DiffViewerProps) {
  const [mode, setMode] = useState<DiffViewMode>(initialMode ?? "unified");

  // Sync with prop changes
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

  const handleToggle = useCallback(() => {
    setMode((prev) => (prev === "unified" ? "split" : "unified"));
  }, []);

  // Null / empty states
  if (diffContent === null) {
    return <EmptyState />;
  }

  if (diffContent.isBinary) {
    return <BinaryIndicator filePath={diffContent.filePath} />;
  }

  const hasNoLines = unifiedRows.length === 0;

  return (
    <div className="flex h-full flex-col" data-testid="diff-viewer">
      {/* Mode toggle header */}
      <ModeToggle mode={mode} onToggle={handleToggle} />

      {/* File mode change banner */}
      {diffContent.fileModeChange !== null && (
        <FileModeChange mode={diffContent.fileModeChange} />
      )}

      {hasNoLines ? (
        <EmptyState />
      ) : mode === "unified" ? (
        <UnifiedRenderer rows={unifiedRows} />
      ) : (
        <SplitRenderer pairs={splitPairs} />
      )}
    </div>
  );
}
