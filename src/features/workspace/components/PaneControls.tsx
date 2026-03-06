import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";

interface PaneControlsProps {
  paneId: string;
  active: boolean;
  profileDraft: string;
  cwdDraft: string;
  commandDraft: string;
  isApplying: boolean;
  onCwdChange: (value: string) => void;
  onCommandChange: (value: string) => void;
  onApplyCwd: () => void;
  onApplyProfile: () => void;
}

export function PaneControls({
  paneId,
  active,
  profileDraft,
  cwdDraft,
  commandDraft,
  isApplying,
  onCwdChange,
  onCommandChange,
  onApplyCwd,
  onApplyProfile,
}: PaneControlsProps) {
  if (!active) {
    return null;
  }

  return (
    <div className="border-b border-[var(--color-border)] bg-black/15 px-4 pb-3">
      <div className="grid gap-2 lg:grid-cols-[minmax(0,1fr)_auto]">
        <div className="flex gap-2">
          <Input
            data-testid={`cwd-input-${paneId}`}
            value={cwdDraft}
            onChange={(event) => onCwdChange(event.target.value)}
            placeholder="Working directory"
            className="h-9 text-xs"
          />
          <Button
            variant="secondary"
            size="sm"
            onClick={onApplyCwd}
            disabled={isApplying}
          >
            Apply cwd
          </Button>
        </div>

        {profileDraft === "custom" ? (
          <div className="flex gap-2">
            <Input
              data-testid={`command-input-${paneId}`}
              value={commandDraft}
              onChange={(event) => onCommandChange(event.target.value)}
              placeholder="Custom command"
              className="h-9 text-xs"
            />
            <Button
              size="sm"
              onClick={onApplyProfile}
              disabled={isApplying || !commandDraft.trim()}
            >
              Launch
            </Button>
          </div>
        ) : (
          <div className="rounded-xl border border-[var(--color-border)] bg-white/4 px-3 py-2 text-xs text-[var(--color-text-muted)]">
            Built-in profiles relaunch instantly when selected.
          </div>
        )}
      </div>
    </div>
  );
}
