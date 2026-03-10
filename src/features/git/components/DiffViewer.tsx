import { useRef, useState, useCallback, useEffect, useMemo } from "react";
import type { DiffContent, DiffHunk, DiffLine } from "@/features/git/domain/models";

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const LINE_HEIGHT_PX = 20;
const OVERSCAN_COUNT = 10;

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

export interface DiffViewerProps {
  readonly diffContent: DiffContent | null;
}

// ---------------------------------------------------------------------------
// Helpers — flatten hunks into renderable rows
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
// Main component
// ---------------------------------------------------------------------------

export function DiffViewer({ diffContent }: DiffViewerProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);

  const rows = useMemo(
    () => (diffContent ? flattenHunks(diffContent.hunks) : []),
    [diffContent],
  );

  const { handleScroll, totalHeight, startIdx, endIdx } = useVirtualScroll(
    rows.length,
    containerRef,
  );

  // Null / empty states
  if (diffContent === null) {
    return <EmptyState />;
  }

  if (diffContent.isBinary) {
    return <BinaryIndicator filePath={diffContent.filePath} />;
  }

  const hasNoLines = rows.length === 0;

  return (
    <div className="flex h-full flex-col" data-testid="diff-viewer">
      {/* File mode change banner */}
      {diffContent.fileModeChange !== null && (
        <FileModeChange mode={diffContent.fileModeChange} />
      )}

      {hasNoLines ? (
        <EmptyState />
      ) : (
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
      )}
    </div>
  );
}
