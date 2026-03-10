/**
 * Pure helper functions for diff line staging.
 *
 * These live in the domain layer because they operate on domain models
 * (DiffLine, DiffHunk) and are consumed by both the application store
 * and presentation components.
 */

import type { DiffLine, DiffHunk } from "./models";

// ---------------------------------------------------------------------------
// Line key — unique identifier for a diff line (used for staged-line tracking)
// ---------------------------------------------------------------------------

export function lineKey(line: DiffLine): string {
  if (line.kind === "addition") return `add:${line.newLineNo}`;
  if (line.kind === "deletion") return `del:${line.oldLineNo}`;
  return `ctx:${line.oldLineNo}:${line.newLineNo}`;
}

// ---------------------------------------------------------------------------
// Line range — git-compatible range string for staging individual lines
// ---------------------------------------------------------------------------

export function lineRange(line: DiffLine): string {
  if (line.kind === "addition" && line.newLineNo !== null) {
    return `${line.newLineNo}-${line.newLineNo}`;
  }
  if (line.kind === "deletion" && line.oldLineNo !== null) {
    return `${line.oldLineNo}-${line.oldLineNo}`;
  }
  return "";
}

// ---------------------------------------------------------------------------
// Hunk-level helpers
// ---------------------------------------------------------------------------

export function hunkLineRanges(hunk: DiffHunk): string[] {
  const ranges: string[] = [];
  for (const line of hunk.lines) {
    const r = lineRange(line);
    if (r !== "" && (line.kind === "addition" || line.kind === "deletion")) {
      ranges.push(r);
    }
  }
  return ranges;
}

export function isHunkFullyStaged(hunk: DiffHunk, stagedLines: ReadonlySet<string>): boolean {
  for (const line of hunk.lines) {
    if (line.kind === "addition" || line.kind === "deletion") {
      if (!stagedLines.has(lineKey(line))) return false;
    }
  }
  return true;
}
