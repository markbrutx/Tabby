import { useMemo } from "react";
import { treeFromCount, computePaneRects } from "@/features/workspace/splitTree";
import type { PaneGroupConfig } from "@/features/workspace/store/types";

const GROUP_COLORS = [
  "bg-[var(--color-accent)]",
  "bg-blue-500",
  "bg-emerald-500",
  "bg-purple-500",
  "bg-amber-500",
];

const GROUP_LABELS: Record<string, string> = {
  terminal: "T",
  browser: "B",
  claude: "C",
  codex: "X",
  custom: "\u2699",
};

interface LayoutPreviewProps {
  groups: PaneGroupConfig[];
}

export function LayoutPreview({ groups }: LayoutPreviewProps) {
  const totalCount = groups.reduce((sum, g) => sum + g.count, 0);

  const cells = useMemo(() => {
    if (totalCount === 0) return [];

    const paneIds = Array.from({ length: totalCount }, (_, i) => `p${i}`);
    const tree = treeFromCount(paneIds);
    const rects = computePaneRects(tree);

    const assignments: Array<{ groupIndex: number; profileId: string }> = [];
    for (let gi = 0; gi < groups.length; gi++) {
      for (let c = 0; c < groups[gi].count; c++) {
        assignments.push({
          groupIndex: gi,
          profileId: groups[gi].mode === "browser" ? "browser" : groups[gi].profileId,
        });
      }
    }

    return paneIds.map((id, i) => {
      const rect = rects.get(id);
      if (!rect) return null;
      const assignment = assignments[i];
      return {
        id,
        rect,
        groupIndex: assignment?.groupIndex ?? 0,
        label: GROUP_LABELS[assignment?.profileId ?? ""] ?? "T",
      };
    }).filter(Boolean) as Array<{
      id: string;
      rect: { x: number; y: number; w: number; h: number };
      groupIndex: number;
      label: string;
    }>;
  }, [totalCount, groups]);

  return (
    <div className="space-y-3">
      <h3 className="text-xs font-medium text-[var(--color-text-muted)]">
        Preview
      </h3>
      <div
        data-testid="layout-preview"
        className="relative w-full overflow-hidden rounded-lg border border-[var(--color-border)] bg-[var(--color-bg)]"
        style={{ aspectRatio: "16 / 10" }}
      >
        {totalCount === 0 ? (
          <div className="flex h-full items-center justify-center">
            <span className="text-xs text-[var(--color-text-muted)]">
              Add a group to begin
            </span>
          </div>
        ) : (
          cells.map((cell) => (
            <div
              key={cell.id}
              className={`absolute flex items-center justify-center rounded-sm text-[10px] font-bold text-white/90 ${GROUP_COLORS[cell.groupIndex % GROUP_COLORS.length]}`}
              style={{
                left: `calc(${cell.rect.x * 100}% + 1px)`,
                top: `calc(${cell.rect.y * 100}% + 1px)`,
                width: `calc(${cell.rect.w * 100}% - 2px)`,
                height: `calc(${cell.rect.h * 100}% - 2px)`,
              }}
            >
              {cell.label}
            </div>
          ))
        )}
      </div>
      <p className="text-center text-xs text-[var(--color-text-muted)]">
        {totalCount === 0
          ? "No panes configured"
          : `${totalCount} pane${totalCount !== 1 ? "s" : ""} \u00b7 auto layout`}
      </p>
    </div>
  );
}
