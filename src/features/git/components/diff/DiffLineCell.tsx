import { useMemo } from "react";
import type { DiffLine } from "@/features/git/domain/models";
import { getLineClassName, formatLineNo, type SplitLineRow, getSplitLineClassName } from "./diffTypes";
import { detectLanguage, tokenizeLine, type Token } from "../syntaxHighlight";

// ---------------------------------------------------------------------------
// Token styling
// ---------------------------------------------------------------------------

const TOKEN_CLASS_MAP: Record<Token["type"], string> = {
  keyword: "text-[var(--color-token-keyword)]",
  string: "text-[var(--color-token-string)]",
  comment: "text-[var(--color-token-comment)] italic",
  number: "text-[var(--color-token-number)]",
  type: "text-[var(--color-token-type)]",
  punctuation: "text-[var(--color-token-punctuation)]",
  plain: "",
};

// ---------------------------------------------------------------------------
// Highlighted content
// ---------------------------------------------------------------------------

interface HighlightedContentProps {
  readonly content: string;
  readonly language: string | null;
}

export function HighlightedContent({ content, language }: HighlightedContentProps) {
  const tokens = useMemo(() => tokenizeLine(content, language), [content, language]);

  if (tokens.length === 1 && tokens[0].type === "plain") {
    return <>{content}</>;
  }

  return (
    <>
      {tokens.map((token, i) => {
        const cls = TOKEN_CLASS_MAP[token.type];
        if (cls === "") return <span key={i}>{token.text}</span>;
        return (
          <span key={i} className={cls} data-token-type={token.type}>
            {token.text}
          </span>
        );
      })}
    </>
  );
}

// ---------------------------------------------------------------------------
// Unified mode line cell
// ---------------------------------------------------------------------------

interface DiffLineCellProps {
  readonly line: DiffLine;
  readonly language: string | null;
  readonly isStaged?: boolean;
  readonly onToggleStage?: () => void;
}

export function DiffLineCell({ line, language, isStaged, onToggleStage }: DiffLineCellProps) {
  const lineClass = getLineClassName(line.kind);
  const isStageable = line.kind === "addition" || line.kind === "deletion";
  const gutterBg =
    line.kind === "addition"
      ? "bg-green-900/15"
      : line.kind === "deletion"
        ? "bg-red-900/15"
        : "";
  const stagedHighlight = isStaged ? " bg-yellow-900/20" : "";

  return (
    <div className={`flex ${lineClass}${stagedHighlight}`} data-testid="diff-line">
      {onToggleStage !== undefined && (
        <button
          type="button"
          className={`w-[24px] shrink-0 select-none border-r border-[var(--color-border)] text-center text-xs ${
            isStageable
              ? "cursor-pointer hover:bg-[var(--color-surface-hover)] text-[var(--color-text)]"
              : "cursor-default text-transparent"
          } ${gutterBg}`}
          onClick={isStageable ? onToggleStage : undefined}
          disabled={!isStageable}
          data-testid="stage-line-btn"
          aria-label={isStaged ? "Unstage line" : "Stage line"}
        >
          {isStageable ? (isStaged ? "\u2713" : "+") : ""}
        </button>
      )}
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
        <HighlightedContent content={line.content} language={language} />
      </span>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Split mode line cell
// ---------------------------------------------------------------------------

interface SplitLineCellProps {
  readonly row: SplitLineRow;
  readonly language: string | null;
  readonly isStaged?: boolean;
  readonly onToggleStage?: () => void;
}

export function SplitLineCell({ row, language, isStaged, onToggleStage }: SplitLineCellProps) {
  const lineClass = getSplitLineClassName(row.kind);
  const isStageable = row.kind === "addition" || row.kind === "deletion";
  const gutterBg =
    row.kind === "addition"
      ? "bg-green-900/15"
      : row.kind === "deletion"
        ? "bg-red-900/15"
        : "";
  const stagedHighlight = isStaged ? " bg-yellow-900/20" : "";

  return (
    <div className={`flex ${lineClass}${stagedHighlight}`} data-testid="split-line">
      {onToggleStage !== undefined && (
        <button
          type="button"
          className={`w-[24px] shrink-0 select-none border-r border-[var(--color-border)] text-center text-xs ${
            isStageable
              ? "cursor-pointer hover:bg-[var(--color-surface-hover)] text-[var(--color-text)]"
              : "cursor-default text-transparent"
          } ${gutterBg}`}
          onClick={isStageable ? onToggleStage : undefined}
          disabled={!isStageable}
          data-testid="stage-line-btn"
          aria-label={isStaged ? "Unstage line" : "Stage line"}
        >
          {isStageable ? (isStaged ? "\u2713" : "+") : ""}
        </button>
      )}
      <span
        className={`w-[50px] shrink-0 select-none border-r border-[var(--color-border)] px-1 text-right text-[var(--color-text-soft)] ${gutterBg}`}
        data-testid="split-line-no"
      >
        {formatLineNo(row.lineNo)}
      </span>
      <span className="flex-1 whitespace-pre px-2" data-testid="split-line-content">
        <HighlightedContent content={row.content} language={language} />
      </span>
    </div>
  );
}
