import { FolderOpen, RotateCcw } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { Select } from "@/components/ui/Select";
import type { PaneProfile, PaneSnapshot } from "@/features/workspace/domain";

interface PaneHeaderProps {
  pane: PaneSnapshot;
  profiles: PaneProfile[];
  active: boolean;
  profileDraft: string;
  isApplying: boolean;
  onSelectProfile: (profileId: string) => void;
  onChooseDirectory: () => void;
  onRestart: () => void;
}

export function PaneHeader({
  pane,
  profiles,
  active,
  profileDraft,
  isApplying,
  onSelectProfile,
  onChooseDirectory,
  onRestart,
}: PaneHeaderProps) {
  return (
    <div className="border-b border-[var(--color-border)] bg-black/15 px-4 py-3">
      <div className="flex items-start gap-3">
        <div
          className={`mt-1.5 h-2.5 w-2.5 rounded-full ${
            active ? "bg-[var(--color-success)]" : "bg-white/20"
          }`}
        />
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <p className="truncate text-sm font-medium">{pane.title}</p>
            <span
              data-testid={`profile-badge-${pane.id}`}
              className="rounded-full bg-white/6 px-2 py-0.5 text-[10px] uppercase tracking-[0.18em] text-[var(--color-text-muted)]"
            >
              {pane.profileLabel}
            </span>
          </div>
          <p className="mt-1 truncate text-xs text-[var(--color-text-soft)]">
            {pane.cwd}
          </p>
        </div>
        <div className="flex min-w-[160px] shrink-0 items-center gap-2">
          <Select
            data-testid={`profile-select-${pane.id}`}
            className="h-8 text-xs"
            value={profileDraft}
            onChange={(event) => onSelectProfile(event.target.value)}
          >
            {profiles.map((profile) => (
              <option key={profile.id} value={profile.id}>
                {profile.label}
              </option>
            ))}
          </Select>
          <Button variant="secondary" size="sm" onClick={onChooseDirectory}>
            <FolderOpen size={14} />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={onRestart}
            disabled={isApplying}
          >
            <RotateCcw size={14} />
          </Button>
        </div>
      </div>
    </div>
  );
}
