import { useMemo } from "react";
import { treeFromCount, computePaneRects } from "@/features/workspace/layoutReadModel";
import { getLayoutVariants, type LayoutVariant } from "@/features/workspace/layoutVariants";
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
  git: "G",
  claude: "C",
  codex: "X",
  gemini: "M",
  opencode: "O",
  custom: "\u2699",
};

interface LayoutPreviewProps {
  groups: PaneGroupConfig[];
  selectedVariantId: string | null;
  onSelectVariant: (variantId: string) => void;
}

function buildAssignments(groups: PaneGroupConfig[]) {
  const assignments: Array<{ groupIndex: number; profileId: string }> = [];
  for (let gi = 0; gi < groups.length; gi++) {
    const group = groups[gi];
    const label = group.mode === "terminal" ? group.profileId : group.mode;
    for (let c = 0; c < group.count; c++) {
      assignments.push({ groupIndex: gi, profileId: label });
    }
  }
  return assignments;
}

function buildCells(
  totalCount: number,
  groups: PaneGroupConfig[],
  variant: LayoutVariant | null,
) {
  if (totalCount === 0) return [];

  const paneIds = Array.from({ length: totalCount }, (_, i) => `p${i}`);
  const tree = variant ? variant.build(paneIds) : treeFromCount(paneIds);
  const rects = computePaneRects(tree);
  const assignments = buildAssignments(groups);

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
}

function VariantThumbnail({
  variant,
  totalCount,
  isSelected,
  onClick,
}: {
  variant: LayoutVariant;
  totalCount: number;
  isSelected: boolean;
  onClick: () => void;
}) {
  const cells = useMemo(() => {
    const paneIds = Array.from({ length: totalCount }, (_, i) => `p${i}`);
    const tree = variant.build(paneIds);
    const rects = computePaneRects(tree);
    return paneIds.map((id) => rects.get(id)).filter(Boolean) as Array<{
      x: number;
      y: number;
      w: number;
      h: number;
    }>;
  }, [variant, totalCount]);

  return (
    <button
      data-testid={`variant-${variant.id}`}
      className={`relative h-[28px] w-[40px] shrink-0 overflow-hidden rounded border transition ${isSelected
          ? "border-[var(--color-accent)] shadow-sm shadow-[var(--color-accent)]/20"
          : "border-[var(--color-border)] hover:border-[var(--color-border-strong)]"
        }`}
      style={{ background: "var(--color-bg)" }}
      onClick={onClick}
      title={variant.label}
    >
      {cells.map((rect, i) => (
        <div
          key={i}
          className={`absolute rounded-[1px] ${isSelected ? "bg-[var(--color-accent)]" : "bg-[var(--color-text-muted)]/30"
            }`}
          style={{
            left: `calc(${rect.x * 100}% + 1px)`,
            top: `calc(${rect.y * 100}% + 1px)`,
            width: `calc(${rect.w * 100}% - 2px)`,
            height: `calc(${rect.h * 100}% - 2px)`,
          }}
        />
      ))}
    </button>
  );
}

export function LayoutPreview({
  groups,
  selectedVariantId,
  onSelectVariant,
}: LayoutPreviewProps) {
  const totalCount = groups.reduce((sum, g) => sum + g.count, 0);
  const variants = useMemo(() => getLayoutVariants(totalCount), [totalCount]);
  const selectedVariant = variants.find((v) => v.id === selectedVariantId) ?? variants[0] ?? null;

  const cells = useMemo(
    () => buildCells(totalCount, groups, selectedVariant),
    [totalCount, groups, selectedVariant],
  );

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
      {variants.length > 1 ? (
        <div className="flex flex-wrap items-center justify-center gap-1.5">
          {variants.map((variant) => (
            <VariantThumbnail
              key={variant.id}
              variant={variant}
              totalCount={totalCount}
              isSelected={variant.id === (selectedVariant?.id ?? null)}
              onClick={() => onSelectVariant(variant.id)}
            />
          ))}
        </div>
      ) : (
        <p className="text-center text-xs text-[var(--color-text-muted)]">
          {totalCount === 0
            ? "No panes configured"
            : `${totalCount} pane${totalCount !== 1 ? "s" : ""}`}
        </p>
      )}
    </div>
  );
}
