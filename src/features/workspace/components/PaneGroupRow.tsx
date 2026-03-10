import { Minus, Plus, X } from "lucide-react";
import type { ProfileReadModel } from "@/features/settings/domain/models";
import type { PaneGroupConfig } from "@/features/workspace/store/types";
import { PaneConfigurator, type PaneFieldValues } from "./PaneConfigurator";

const GROUP_DOT_COLORS = [
  "bg-[var(--color-accent)]",
  "bg-blue-500",
  "bg-emerald-500",
  "bg-amber-500",
  "bg-rose-500",
];

const MODE_LABELS: Record<PaneGroupConfig["mode"], string> = {
  terminal: "Terminal",
  browser: "Browser",
  git: "Git",
};

interface PaneGroupRowProps {
  index: number;
  group: PaneGroupConfig;
  profiles: ProfileReadModel[];
  maxCount: number;
  canRemove: boolean;
  onChange: (updated: PaneGroupConfig) => void;
  onRemove: () => void;
}

export function groupToFieldValues(group: PaneGroupConfig): PaneFieldValues {
  switch (group.mode) {
    case "browser":
      return { mode: "browser", url: group.url };
    case "git":
      return { mode: "git", workingDirectory: group.workingDirectory };
    case "terminal":
      return {
        mode: "terminal",
        profileId: group.profileId,
        workingDirectory: group.workingDirectory,
        customCommand: group.customCommand,
      };
  }
}

function fieldValuesToGroup(values: PaneFieldValues, count: number): PaneGroupConfig {
  switch (values.mode) {
    case "browser":
      return { mode: "browser", url: values.url, count };
    case "git":
      return { mode: "git", workingDirectory: values.workingDirectory, count };
    case "terminal":
      return {
        mode: "terminal",
        profileId: values.profileId,
        workingDirectory: values.workingDirectory,
        customCommand: values.customCommand,
        count,
      };
  }
}

function withCount(group: PaneGroupConfig, count: number): PaneGroupConfig {
  return { ...group, count };
}

export function PaneGroupRow({
  index,
  group,
  profiles,
  maxCount,
  canRemove,
  onChange,
  onRemove,
}: PaneGroupRowProps) {
  const dotColor = GROUP_DOT_COLORS[index % GROUP_DOT_COLORS.length];

  return (
    <div
      data-testid={`pane-group-${index}`}
      className="space-y-3 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-overlay)] p-4"
    >
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className={`h-2 w-2 rounded-full ${dotColor}`} />
          <span className="text-xs font-medium text-[var(--color-text-muted)]">
            Group {index + 1} — {MODE_LABELS[group.mode]}
          </span>
        </div>
        <div className="flex items-center gap-1">
          <div className="flex items-center gap-0.5 rounded-full border border-[var(--color-border-strong)] bg-[var(--color-surface-overlay)] px-1">
            <button
              data-testid={`group-decrement-${index}`}
              className="flex h-7 w-7 items-center justify-center rounded-full text-[var(--color-text)] transition hover:bg-[var(--color-surface-hover)] disabled:opacity-40"
              disabled={group.count <= 1}
              onClick={() => onChange(withCount(group, group.count - 1))}
            >
              <Minus size={14} />
            </button>
            <span
              data-testid={`group-count-${index}`}
              className="w-6 text-center text-sm font-medium text-[var(--color-text)]"
            >
              {group.count}
            </span>
            <button
              data-testid={`group-increment-${index}`}
              className="flex h-7 w-7 items-center justify-center rounded-full text-[var(--color-text)] transition hover:bg-[var(--color-surface-hover)] disabled:opacity-40"
              disabled={group.count >= maxCount}
              onClick={() => onChange(withCount(group, group.count + 1))}
            >
              <Plus size={14} />
            </button>
          </div>
          {canRemove ? (
            <button
              data-testid={`group-remove-${index}`}
              className="rounded p-1 text-[var(--color-text-muted)] transition hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]"
              onClick={onRemove}
            >
              <X size={14} />
            </button>
          ) : null}
        </div>
      </div>

      <PaneConfigurator
        values={groupToFieldValues(group)}
        profiles={profiles}
        onChange={(values) => onChange(fieldValuesToGroup(values, group.count))}
        testIdPrefix={`group-${index}`}
      />
    </div>
  );
}
