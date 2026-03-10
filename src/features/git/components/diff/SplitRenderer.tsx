import { useRef, useCallback } from "react";
import type { DiffHunk } from "@/features/git/domain/models";
import { lineKey, isHunkFullyStaged } from "@/features/git/domain/stagingHelpers";
import type { SplitPair, SplitHunkHeaderRow, SplitLineRow, StagingCallbacks } from "./diffTypes";
import { LINE_HEIGHT_PX } from "./diffTypes";
import { useVirtualScroll, useSyncScroll } from "./useVirtualScroll";
import { SplitHunkHeader } from "./HunkHeader";
import { SplitLineCell } from "./DiffLineCell";

interface SplitRendererProps {
  readonly pairs: readonly SplitPair[];
  readonly filePath: string;
  readonly hunks: readonly DiffHunk[];
  readonly language: string | null;
  readonly staging?: StagingCallbacks;
  readonly stagedLines?: ReadonlySet<string>;
}

export function SplitRenderer({ pairs, filePath, hunks, language, staging, stagedLines }: SplitRendererProps) {
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

  function renderStagingToggle(row: SplitLineRow) {
    if (staging === undefined || row.sourceLineKey === null) return undefined;
    return () => {
      const key = row.sourceLineKey;
      if (key === null) return;
      const staged = stagedLines !== undefined && stagedLines.has(key);
      const range = row.lineNo !== null ? `${row.lineNo}-${row.lineNo}` : "";
      if (range === "") return;
      if (staged) {
        staging.onUnstageLines(filePath, [range]);
      } else {
        staging.onStageLines(filePath, [range]);
      }
    };
  }

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
                  <SplitHunkHeader
                    header={pair.left.header}
                    isStaged={stagedLines !== undefined && hunks[pair.left.hunkIndex] !== undefined && isHunkFullyStaged(hunks[pair.left.hunkIndex], stagedLines)}
                    onStageHunk={staging !== undefined ? () => {
                      const hIdx = (pair.left as SplitHunkHeaderRow).hunkIndex;
                      const hunk = hunks[hIdx];
                      if (hunk === undefined) return;
                      const fullyStaged = stagedLines !== undefined && isHunkFullyStaged(hunk, stagedLines);
                      if (fullyStaged) {
                        staging.onUnstageHunk(filePath, hIdx);
                      } else {
                        staging.onStageHunk(filePath, hIdx);
                      }
                    } : undefined}
                  />
                ) : (
                  <SplitLineCell
                    row={pair.left}
                    language={language}
                    isStaged={stagedLines !== undefined && pair.left.sourceLineKey !== null && stagedLines.has(pair.left.sourceLineKey)}
                    onToggleStage={renderStagingToggle(pair.left)}
                  />
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
                  <SplitLineCell
                    row={pair.right}
                    language={language}
                    isStaged={stagedLines !== undefined && pair.right.sourceLineKey !== null && stagedLines.has(pair.right.sourceLineKey)}
                    onToggleStage={renderStagingToggle(pair.right)}
                  />
                )}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
