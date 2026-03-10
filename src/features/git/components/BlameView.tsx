import { useMemo } from "react";
import type { BlameEntry } from "@/features/git/domain/models";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

export interface BlameViewProps {
  readonly filePath: string;
  readonly entries: readonly BlameEntry[];
  readonly onCommitClick: (hash: string) => void;
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
  const diffWeek = Math.floor(diffDay / 7);
  const diffMonth = Math.floor(diffDay / 30);

  if (diffSec < 60) return "just now";
  if (diffMin < 60) return `${diffMin}m ago`;
  if (diffHr < 24) return `${diffHr}h ago`;
  if (diffDay < 7) return `${diffDay}d ago`;
  if (diffWeek < 5) return `${diffWeek}w ago`;
  if (diffMonth < 12) return `${diffMonth}mo ago`;
  return date.toLocaleDateString();
}

/**
 * Alternating background colors for visual grouping of blame blocks.
 * Even-indexed blocks get one color, odd-indexed blocks get another.
 */
const BLOCK_COLORS = [
  "bg-[var(--color-bg)]",
  "bg-[var(--color-bg-elevated)]",
] as const;

interface ExpandedBlameLine {
  readonly lineNumber: number;
  readonly content: string;
  readonly entry: BlameEntry;
  readonly isBlockStart: boolean;
  readonly blockIndex: number;
}

function expandBlameEntries(entries: readonly BlameEntry[]): readonly ExpandedBlameLine[] {
  const lines: ExpandedBlameLine[] = [];

  for (let blockIndex = 0; blockIndex < entries.length; blockIndex++) {
    const entry = entries[blockIndex];
    const contentLines = entry.content.split("\n");

    for (let offset = 0; offset < entry.lineCount; offset++) {
      lines.push({
        lineNumber: entry.lineStart + offset,
        content: contentLines[offset] ?? "",
        entry,
        isBlockStart: offset === 0,
        blockIndex,
      });
    }
  }

  return lines;
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

interface BlameAnnotationProps {
  readonly entry: BlameEntry;
  readonly isBlockStart: boolean;
  readonly onCommitClick: (hash: string) => void;
}

function BlameAnnotation({ entry, isBlockStart, onCommitClick }: BlameAnnotationProps) {
  if (!isBlockStart) {
    return <span className="inline-block w-52 shrink-0" />;
  }

  return (
    <span className="inline-flex w-52 shrink-0 items-baseline gap-1.5 truncate pr-2 text-[11px]">
      <button
        type="button"
        className="font-mono text-[var(--color-accent)] hover:underline"
        onClick={() => onCommitClick(entry.hash)}
        data-testid={`blame-hash-${entry.hash}`}
        title={`View commit ${entry.hash}`}
      >
        {entry.hash.slice(0, 7)}
      </button>
      <span className="truncate text-[var(--color-text-soft)]" title={entry.author}>
        {entry.author}
      </span>
      <span className="shrink-0 text-[var(--color-text-soft)]">
        {formatRelativeDate(entry.date)}
      </span>
    </span>
  );
}

// ---------------------------------------------------------------------------
// Main component
// ---------------------------------------------------------------------------

export function BlameView({ filePath, entries, onCommitClick }: BlameViewProps) {
  const expandedLines = useMemo(() => expandBlameEntries(entries), [entries]);

  if (entries.length === 0) {
    return (
      <div
        className="flex h-full items-center justify-center"
        data-testid="blame-empty"
      >
        <span className="text-sm text-[var(--color-text-soft)]">
          No blame data available
        </span>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col" data-testid="blame-view">
      {/* Header */}
      <div className="flex items-center border-b border-[var(--color-border)] px-3 py-1.5">
        <span className="text-xs font-medium text-[var(--color-text)]">
          Blame: {filePath}
        </span>
      </div>

      {/* Content */}
      <div className="min-h-0 flex-1 overflow-auto" data-testid="blame-content">
        <div className="min-w-fit">
          {expandedLines.map((line) => (
            <div
              key={line.lineNumber}
              className={`flex items-baseline border-b border-[var(--color-border)]/20 ${BLOCK_COLORS[line.blockIndex % 2]}`}
              data-testid={`blame-line-${line.lineNumber}`}
            >
              {/* Annotation gutter */}
              <BlameAnnotation
                entry={line.entry}
                isBlockStart={line.isBlockStart}
                onCommitClick={onCommitClick}
              />

              {/* Line number */}
              <span className="inline-block w-10 shrink-0 pr-2 text-right font-mono text-[11px] text-[var(--color-text-soft)] select-none">
                {line.lineNumber}
              </span>

              {/* Line separator */}
              <span className="inline-block w-px shrink-0 self-stretch bg-[var(--color-border)]" />

              {/* Content */}
              <span className="flex-1 whitespace-pre pl-2 font-mono text-xs text-[var(--color-text)]">
                {line.content}
              </span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
