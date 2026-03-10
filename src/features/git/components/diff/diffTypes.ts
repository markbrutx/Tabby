import type { DiffLine, DiffHunk } from "@/features/git/domain/models";
import { lineKey } from "@/features/git/domain/stagingHelpers";

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

export const LINE_HEIGHT_PX = 20;
export const OVERSCAN_COUNT = 10;

// ---------------------------------------------------------------------------
// Shared types
// ---------------------------------------------------------------------------

export type DiffViewMode = "unified" | "split";

export interface StagingCallbacks {
  readonly onStageLines: (filePath: string, lineRanges: string[]) => void;
  readonly onUnstageLines: (filePath: string, lineRanges: string[]) => void;
  readonly onStageHunk: (filePath: string, hunkIndex: number) => void;
  readonly onUnstageHunk: (filePath: string, hunkIndex: number) => void;
}

export interface DiffViewerProps {
  readonly diffContent: DiffContent | null;
  readonly mode?: DiffViewMode;
  readonly staging?: StagingCallbacks;
  readonly stagedLines?: ReadonlySet<string>;
}

// ---------------------------------------------------------------------------
// Unified mode row types
// ---------------------------------------------------------------------------

export interface HunkHeaderRow {
  readonly type: "hunkHeader";
  readonly header: string;
  readonly hunkIndex: number;
}

export interface DiffLineRow {
  readonly type: "line";
  readonly line: DiffLine;
  readonly hunkIndex: number;
}

export type DiffRow = HunkHeaderRow | DiffLineRow;

// ---------------------------------------------------------------------------
// Split mode row types
// ---------------------------------------------------------------------------

export interface SplitHunkHeaderRow {
  readonly type: "hunkHeader";
  readonly header: string;
  readonly hunkIndex: number;
}

export interface SplitLineRow {
  readonly type: "line";
  readonly lineNo: number | null;
  readonly content: string;
  readonly kind: "context" | "addition" | "deletion" | "blank";
  readonly sourceLineKey: string | null;
}

export type SplitRow = SplitHunkHeaderRow | SplitLineRow;

export interface SplitPair {
  readonly left: SplitRow;
  readonly right: SplitRow;
}

// We need DiffContent for props but it's already in domain models
import type { DiffContent } from "@/features/git/domain/models";

// ---------------------------------------------------------------------------
// Row builders
// ---------------------------------------------------------------------------

export function flattenHunks(hunks: readonly DiffHunk[]): readonly DiffRow[] {
  const rows: DiffRow[] = [];
  for (let hunkIndex = 0; hunkIndex < hunks.length; hunkIndex++) {
    const hunk = hunks[hunkIndex];
    rows.push({ type: "hunkHeader", header: hunk.header, hunkIndex });
    for (const line of hunk.lines) {
      rows.push({ type: "line", line, hunkIndex });
    }
  }
  return rows;
}

export function buildSplitRows(hunks: readonly DiffHunk[]): readonly SplitPair[] {
  const pairs: SplitPair[] = [];

  for (let hunkIndex = 0; hunkIndex < hunks.length; hunkIndex++) {
    const hunk = hunks[hunkIndex];
    const headerRow: SplitHunkHeaderRow = { type: "hunkHeader", header: hunk.header, hunkIndex };
    pairs.push({ left: headerRow, right: headerRow });

    const lines = hunk.lines;
    let i = 0;
    while (i < lines.length) {
      const line = lines[i];

      if (line.kind === "context") {
        const key = lineKey(line);
        pairs.push({
          left: { type: "line", lineNo: line.oldLineNo, content: line.content, kind: "context", sourceLineKey: key },
          right: { type: "line", lineNo: line.newLineNo, content: line.content, kind: "context", sourceLineKey: key },
        });
        i++;
      } else if (line.kind === "deletion") {
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
              ? { type: "line", lineNo: del.oldLineNo, content: del.content, kind: "deletion", sourceLineKey: lineKey(del) }
              : { type: "line", lineNo: null, content: "", kind: "blank", sourceLineKey: null },
            right: add
              ? { type: "line", lineNo: add.newLineNo, content: add.content, kind: "addition", sourceLineKey: lineKey(add) }
              : { type: "line", lineNo: null, content: "", kind: "blank", sourceLineKey: null },
          });
        }
      } else if (line.kind === "addition") {
        pairs.push({
          left: { type: "line", lineNo: null, content: "", kind: "blank", sourceLineKey: null },
          right: { type: "line", lineNo: line.newLineNo, content: line.content, kind: "addition", sourceLineKey: lineKey(line) },
        });
        i++;
      } else {
        i++;
      }
    }
  }

  return pairs;
}

// ---------------------------------------------------------------------------
// Line styling
// ---------------------------------------------------------------------------

export function getLineClassName(kind: DiffLine["kind"]): string {
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

export function getSplitLineClassName(kind: SplitLineRow["kind"]): string {
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

export function formatLineNo(n: number | null): string {
  if (n === null) return "";
  return String(n);
}
