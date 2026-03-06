import { AlertTriangle, RotateCcw } from "lucide-react";
import { Button } from "@/components/ui/Button";
import type { LayoutPreset } from "@/features/workspace/domain";
import { LAYOUT_PRESET_CARDS } from "@/features/workspace/presets";

const QUICK_LAUNCH_COUNT = 4;

interface RecoveryScreenProps {
  title: string;
  message: string;
  onRetry?: () => void;
  onCreateTab?: (preset: LayoutPreset) => void;
}

export function RecoveryScreen({
  title,
  message,
  onRetry,
  onCreateTab,
}: RecoveryScreenProps) {
  return (
    <div className="flex min-h-screen items-center justify-center p-8 text-[var(--color-text)]">
      <div className="surface-panel w-full max-w-xl rounded-[32px] p-8">
        <div className="flex items-center gap-3 text-[var(--color-warning)]">
          <AlertTriangle size={20} />
          <p className="text-sm uppercase tracking-[0.25em]">{title}</p>
        </div>

        <p className="mt-4 text-sm text-[var(--color-text-soft)]">{message}</p>

        {onRetry ? (
          <Button className="mt-6" onClick={onRetry}>
            <RotateCcw size={16} />
            Retry bootstrap
          </Button>
        ) : null}

        {onCreateTab ? (
          <div className="mt-6">
            <p className="mb-3 text-xs uppercase tracking-[0.25em] text-[var(--color-text-muted)]">
              Quick launch
            </p>
            <div className="grid grid-cols-2 gap-2">
              {LAYOUT_PRESET_CARDS.slice(0, QUICK_LAUNCH_COUNT).map((card) => {
                const Icon = card.icon;
                return (
                  <button
                    key={card.preset}
                    className="surface-muted flex items-center gap-3 rounded-2xl p-3 text-start transition hover:bg-white/6"
                    onClick={() => onCreateTab(card.preset)}
                  >
                    <div className="flex h-8 w-8 items-center justify-center rounded-xl bg-[var(--color-accent-soft)] text-[var(--color-accent)]">
                      <Icon size={16} />
                    </div>
                    <div className="min-w-0">
                      <p className="text-sm font-medium">{card.title}</p>
                      <p className="text-xs text-[var(--color-text-muted)]">
                        {card.preset}
                      </p>
                    </div>
                  </button>
                );
              })}
            </div>
          </div>
        ) : null}
      </div>
    </div>
  );
}
