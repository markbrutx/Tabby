import { FolderOpen, Minus, Plus, X } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Select } from "@/components/ui/Select";
import { BROWSER_PROFILE_ID, CUSTOM_PROFILE_ID, type PaneProfile } from "@/features/workspace/domain";
import type { PaneGroupConfig } from "@/features/workspace/store/types";
import { pickDirectory } from "@/lib/pickDirectory";

const GROUP_DOT_COLORS = [
  "bg-[var(--color-accent)]",
  "bg-blue-500",
  "bg-emerald-500",
  "bg-purple-500",
  "bg-amber-500",
];

interface PaneGroupRowProps {
  index: number;
  group: PaneGroupConfig;
  profiles: PaneProfile[];
  maxCount: number;
  canRemove: boolean;
  onChange: (update: Partial<PaneGroupConfig>) => void;
  onRemove: () => void;
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
  async function handlePickDirectory() {
    const selected = await pickDirectory(group.workingDirectory || undefined);
    if (selected) {
      onChange({ workingDirectory: selected });
    }
  }

  const dotColor = GROUP_DOT_COLORS[index % GROUP_DOT_COLORS.length];

  return (
    <div
      data-testid={`pane-group-${index}`}
      className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-overlay)] p-4 space-y-3"
    >
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className={`h-2 w-2 rounded-full ${dotColor}`} />
          <span className="text-xs font-medium text-[var(--color-text-muted)]">
            Group {index + 1}
          </span>
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

      <div className="flex items-center gap-3">
        <div className="flex-1">
          <Select
            data-testid={`group-profile-${index}`}
            value={group.profileId}
            onChange={(e) => onChange({ profileId: e.target.value })}
            className="text-sm"
          >
            {profiles.map((p) => (
              <option key={p.id} value={p.id}>
                {p.label}
              </option>
            ))}
          </Select>
        </div>
        <div className="flex items-center gap-0.5 rounded-full border border-[var(--color-border-strong)] bg-[var(--color-surface-overlay)] px-1">
          <button
            data-testid={`group-decrement-${index}`}
            className="flex h-7 w-7 items-center justify-center rounded-full text-[var(--color-text)] transition hover:bg-[var(--color-surface-hover)] disabled:opacity-40"
            disabled={group.count <= 1}
            onClick={() => onChange({ count: group.count - 1 })}
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
            onClick={() => onChange({ count: group.count + 1 })}
          >
            <Plus size={14} />
          </button>
        </div>
      </div>

      {group.profileId === BROWSER_PROFILE_ID ? (
        <Input
          data-testid={`group-url-${index}`}
          value={group.url ?? ""}
          onChange={(e) => onChange({ url: e.target.value })}
          placeholder="https://google.com"
          className="text-sm"
        />
      ) : (
        <>
          <div className="flex gap-2">
            <Input
              data-testid={`group-dir-${index}`}
              value={group.workingDirectory}
              onChange={(e) => onChange({ workingDirectory: e.target.value })}
              placeholder="Working directory"
              className="text-sm"
            />
            <Button
              variant="secondary"
              size="sm"
              className="shrink-0"
              onClick={() => void handlePickDirectory()}
            >
              <FolderOpen size={14} />
            </Button>
          </div>

          {group.profileId === CUSTOM_PROFILE_ID ? (
            <Input
              data-testid={`group-command-${index}`}
              value={group.customCommand ?? ""}
              onChange={(e) => onChange({ customCommand: e.target.value })}
              placeholder="Custom command (e.g. npm run dev)"
              className="text-sm"
            />
          ) : null}
        </>
      )}
    </div>
  );
}
