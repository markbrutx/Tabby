/**
 * Internal Git domain models.
 *
 * These types are independent of transport/generated bindings.
 * Mappers in the application layer convert between these models
 * and the wire-format DTOs from tauri-bindings.
 */

// ---------------------------------------------------------------------------
// Enums / Unions
// ---------------------------------------------------------------------------

export type FileStatusKind =
  | "modified"
  | "added"
  | "deleted"
  | "renamed"
  | "copied"
  | "untracked"
  | "ignored"
  | "conflicted";

export type DiffLineKind = "context" | "addition" | "deletion" | "hunkHeader";

// ---------------------------------------------------------------------------
// File status
// ---------------------------------------------------------------------------

export interface FileStatus {
  readonly path: string;
  readonly oldPath: string | null;
  readonly indexStatus: FileStatusKind;
  readonly worktreeStatus: FileStatusKind;
}

// ---------------------------------------------------------------------------
// Diff
// ---------------------------------------------------------------------------

export interface DiffLine {
  readonly kind: DiffLineKind;
  readonly oldLineNo: number | null;
  readonly newLineNo: number | null;
  readonly content: string;
}

export interface DiffHunk {
  readonly oldStart: number;
  readonly oldCount: number;
  readonly newStart: number;
  readonly newCount: number;
  readonly header: string;
  readonly lines: readonly DiffLine[];
}

export interface DiffContent {
  readonly filePath: string;
  readonly oldPath: string | null;
  readonly hunks: readonly DiffHunk[];
  readonly isBinary: boolean;
  readonly fileModeChange: string | null;
}

// ---------------------------------------------------------------------------
// Commits & Branches
// ---------------------------------------------------------------------------

export interface CommitInfo {
  readonly hash: string;
  readonly shortHash: string;
  readonly authorName: string;
  readonly authorEmail: string;
  readonly date: string;
  readonly message: string;
  readonly parentHashes: readonly string[];
}

export interface BranchInfo {
  readonly name: string;
  readonly isCurrent: boolean;
  readonly upstream: string | null;
  readonly ahead: number;
  readonly behind: number;
}

// ---------------------------------------------------------------------------
// Blame & Stash
// ---------------------------------------------------------------------------

export interface BlameEntry {
  readonly hash: string;
  readonly author: string;
  readonly date: string;
  readonly lineStart: number;
  readonly lineCount: number;
  readonly content: string;
}

export interface StashEntry {
  readonly index: number;
  readonly message: string;
  readonly date: string;
}

// ---------------------------------------------------------------------------
// Repository state
// ---------------------------------------------------------------------------

export interface GitRepoState {
  readonly repoPath: string;
  readonly headBranch: string | null;
  readonly isDetached: boolean;
  readonly statusClean: boolean;
}
