// ---------------------------------------------------------------------------
// Hunk header cells for unified and split modes
// ---------------------------------------------------------------------------

interface HunkHeaderCellProps {
  readonly header: string;
  readonly isStaged?: boolean;
  readonly onStageHunk?: () => void;
}

export function HunkHeaderCell({ header, isStaged, onStageHunk }: HunkHeaderCellProps) {
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
      {onStageHunk !== undefined && (
        <button
          type="button"
          className="mr-2 shrink-0 rounded px-2 py-0.5 text-xs text-blue-300 hover:bg-blue-900/30"
          onClick={onStageHunk}
          data-testid="stage-hunk-btn"
        >
          {isStaged ? "Unstage Hunk" : "Stage Hunk"}
        </button>
      )}
    </div>
  );
}

interface SplitHunkHeaderProps {
  readonly header: string;
  readonly isStaged?: boolean;
  readonly onStageHunk?: () => void;
}

export function SplitHunkHeader({ header, isStaged, onStageHunk }: SplitHunkHeaderProps) {
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
      {onStageHunk !== undefined && (
        <button
          type="button"
          className="mr-2 shrink-0 rounded px-2 py-0.5 text-xs text-blue-300 hover:bg-blue-900/30"
          onClick={onStageHunk}
          data-testid="stage-hunk-btn"
        >
          {isStaged ? "Unstage Hunk" : "Stage Hunk"}
        </button>
      )}
    </div>
  );
}
