import { useRef, useCallback } from "react";
import type { DiffHunk } from "@/features/git/domain/models";
import { lineKey, lineRange, isHunkFullyStaged } from "@/features/git/domain/stagingHelpers";
import type { DiffRow, StagingCallbacks } from "./diffTypes";
import { LINE_HEIGHT_PX } from "./diffTypes";
import { useVirtualScroll } from "./useVirtualScroll";
import { HunkHeaderCell } from "./HunkHeader";
import { DiffLineCell } from "./DiffLineCell";

interface UnifiedRendererProps {
  readonly rows: readonly DiffRow[];
  readonly filePath: string;
  readonly hunks: readonly DiffHunk[];
  readonly language: string | null;
  readonly staging?: StagingCallbacks;
  readonly stagedLines?: ReadonlySet<string>;
}

export function UnifiedRenderer({ rows, filePath, hunks, language, staging, stagedLines }: UnifiedRendererProps) {
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
                <HunkHeaderCell
                  header={row.header}
                  isStaged={stagedLines !== undefined && hunks[row.hunkIndex] !== undefined && isHunkFullyStaged(hunks[row.hunkIndex], stagedLines)}
                  onStageHunk={staging !== undefined ? () => {
                    const hunk = hunks[row.hunkIndex];
                    if (hunk === undefined) return;
                    const fullyStaged = stagedLines !== undefined && isHunkFullyStaged(hunk, stagedLines);
                    if (fullyStaged) {
                      staging.onUnstageHunk(filePath, row.hunkIndex);
                    } else {
                      staging.onStageHunk(filePath, row.hunkIndex);
                    }
                  } : undefined}
                />
              ) : (
                <DiffLineCell
                  line={row.line}
                  language={language}
                  isStaged={stagedLines !== undefined && stagedLines.has(lineKey(row.line))}
                  onToggleStage={staging !== undefined ? () => {
                    const key = lineKey(row.line);
                    const staged = stagedLines !== undefined && stagedLines.has(key);
                    const range = lineRange(row.line);
                    if (range === "") return;
                    if (staged) {
                      staging.onUnstageLines(filePath, [range]);
                    } else {
                      staging.onStageLines(filePath, [range]);
                    }
                  } : undefined}
                />
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
