import { Minus, Plus, X, Terminal, Globe, GitBranch } from "lucide-react";
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

const MODE_ICONS: Record<PaneGroupConfig["mode"], React.ElementType> = {
  terminal: Terminal,
  browser: Globe,
  git: GitBranch,
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
  const Icon = MODE_ICONS[group.mode];

  return (
    <div
      data-testid={`pane-group-${index}`}
      className="flex items-start gap-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-overlay)] p-2 pl-3 shadow-sm transition hover:border-[var(--color-border-strong)]"
    >
      <div className="flex w-[100px] shrink-0 items-center gap-2.5 pt-1">
        <div className={`flex h-7 w-7 shrink-0 items-center justify-center rounded-lg ${dotColor} text-white shadow-sm`}>
          <Icon size={14} />
        </div>
        <span className="font-medium text-[var(--color-text)] text-sm">
          {MODE_LABELS[group.mode]}
        </span>
      </div>

      <PaneConfigurator
        layout="inline"
        values={groupToFieldValues(group)}
        profiles={profiles}
        onChange={(values) => onChange(fieldValuesToGroup(values, group.count))}
        testIdPrefix={`group-${index}`}
      />

      <div className="flex shrink-0 items-center gap-2 border-l border-[var(--color-border)] pl-4 pt-1">
        <div className="flex items-center gap-0.5 rounded-lg border border-[var(--color-border-strong)] bg-[var(--color-surface)] p-0.5 shadow-sm">
          <button
            data-testid={`group-decrement-${index}`}
            className="flex h-6 w-6 items-center justify-center rounded-md text-[var(--color-text-muted)] transition hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)] disabled:opacity-40"
            disabled={group.count <= 1}
            onClick={() => onChange(withCount(group, group.count - 1))}
          >
            <Minus size={12} />
          </button>
          <span
            data-testid={`group-count-${index}`}
            className="w-5 text-center text-xs font-medium text-[var(--color-text)]"
          >
            {group.count}
          </span>
          <button
            data-testid={`group-increment-${index}`}
            className="flex h-6 w-6 items-center justify-center rounded-md text-[var(--color-text-muted)] transition hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)] disabled:opacity-40"
            disabled={group.count >= maxCount}
            onClick={() => onChange(withCount(group, group.count + 1))}
          >
            <Plus size={12} />
          </button>
        </div>

        {canRemove ? (
          <button
            data-testid={`group-remove-${index}`}
            className="flex h-7 w-7 items-center justify-center rounded-lg text-[var(--color-text-muted)] transition hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]"
            onClick={onRemove}
          >
            <X size={14} />
          </button>
        ) : <div className="h-7 w-7" />}
      </div>
    </div>
  );
}
