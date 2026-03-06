import {
  Command,
  FolderTree,
  Grid2x2,
  Grid3x3,
  PanelTop,
  Settings2,
  Sparkles,
} from "lucide-react";
import { Button } from "@/components/ui/Button";
import type { LayoutPreset, WorkspaceSettings } from "@/features/workspace/domain";

const PRESET_CARDS: {
  preset: LayoutPreset;
  title: string;
  description: string;
  icon: typeof PanelTop;
}[] = [
  {
    preset: "1x1",
    title: "Solo",
    description: "Single focused shell for one command stream.",
    icon: PanelTop,
  },
  {
    preset: "1x2",
    title: "Pair",
    description: "Two terminals side by side for compare-and-apply flows.",
    icon: Grid2x2,
  },
  {
    preset: "2x2",
    title: "Quad",
    description: "Balanced workspace for implementation, logs, and notes.",
    icon: Grid2x2,
  },
  {
    preset: "2x3",
    title: "Research",
    description: "Six panes for agent work, test loops, and diagnostics.",
    icon: Grid3x3,
  },
  {
    preset: "3x3",
    title: "War Room",
    description: "Nine live panes for broad sweeps and parallel sessions.",
    icon: Sparkles,
  },
];

interface AppSidebarProps {
  isWorking: boolean;
  settings: WorkspaceSettings;
  onCreateTab: (preset: LayoutPreset) => void;
  onOpenSettings: () => void;
}

export function AppSidebar({
  isWorking,
  settings,
  onCreateTab,
  onOpenSettings,
}: AppSidebarProps) {
  return (
    <aside className="surface-panel flex h-full flex-col rounded-[28px] p-5">
      <div className="rounded-[24px] border border-[var(--color-border)] bg-[linear-gradient(135deg,rgba(245,151,184,0.22),rgba(255,255,255,0.04))] p-5">
        <p className="text-xs uppercase tracking-[0.35em] text-[var(--color-text-muted)]">
          Terminal Workspace
        </p>
        <div className="mt-3 flex items-center gap-3">
          <div className="flex h-12 w-12 items-center justify-center rounded-2xl bg-black/25 text-xl font-semibold">
            T
          </div>
          <div>
            <h1 className="text-3xl font-semibold tracking-tight">Tabby</h1>
            <p className="text-sm text-[var(--color-text-soft)]">
              Handy-inspired command decks for Codex, Claude and plain shells.
            </p>
          </div>
        </div>
      </div>

      <div className="mt-5">
        <div className="mb-3 flex items-center justify-between">
          <p className="text-xs uppercase tracking-[0.25em] text-[var(--color-text-muted)]">
            Launchpads
          </p>
          <span className="rounded-full bg-white/6 px-3 py-1 text-[11px] text-[var(--color-text-soft)]">
            default {settings.defaultLayout}
          </span>
        </div>

        <div className="space-y-2">
          {PRESET_CARDS.map((card) => {
            const Icon = card.icon;
            return (
              <button
                key={card.preset}
                data-testid={`launchpad-${card.preset}`}
                className="surface-muted w-full rounded-2xl p-4 text-start transition hover:border-[var(--color-accent-strong)] hover:bg-white/6"
                onClick={() => onCreateTab(card.preset)}
                disabled={isWorking}
              >
                <div className="flex items-start gap-3">
                  <div className="mt-1 flex h-9 w-9 items-center justify-center rounded-xl bg-[var(--color-accent-soft)] text-[var(--color-accent)]">
                    <Icon size={18} />
                  </div>
                  <div className="min-w-0">
                    <div className="flex items-center gap-2">
                      <p className="font-medium">{card.title}</p>
                      <span className="rounded-full bg-white/8 px-2 py-0.5 text-[10px] uppercase tracking-[0.2em] text-[var(--color-text-muted)]">
                        {card.preset}
                      </span>
                    </div>
                    <p className="mt-1 text-sm leading-5 text-[var(--color-text-soft)]">
                      {card.description}
                    </p>
                  </div>
                </div>
              </button>
            );
          })}
        </div>
      </div>

      <div className="mt-5 rounded-2xl border border-[var(--color-border)] bg-black/20 p-4">
        <p className="text-xs uppercase tracking-[0.25em] text-[var(--color-text-muted)]">
          Defaults
        </p>
        <div className="mt-3 space-y-2 text-sm text-[var(--color-text-soft)]">
          <div className="flex items-center gap-2">
            <Command size={16} className="text-[var(--color-accent)]" />
            <span>{settings.defaultProfileId}</span>
          </div>
          <div className="flex items-center gap-2">
            <FolderTree size={16} className="text-[var(--color-accent)]" />
            <span className="truncate">{settings.defaultWorkingDirectory || "Home"}</span>
          </div>
          <div className="flex items-center gap-2">
            <Settings2 size={16} className="text-[var(--color-accent)]" />
            <span>{settings.fontSize}px terminal font</span>
          </div>
        </div>
      </div>

      <div className="mt-auto space-y-3 pt-5">
        <Button
          data-testid="open-settings"
          variant="secondary"
          size="lg"
          className="w-full justify-between"
          onClick={onOpenSettings}
        >
          Workspace settings
          <Settings2 size={16} />
        </Button>

        <div className="rounded-2xl border border-[var(--color-border)] bg-white/3 p-4 text-sm text-[var(--color-text-soft)]">
          <p className="text-xs uppercase tracking-[0.25em] text-[var(--color-text-muted)]">
            Shortcuts
          </p>
          <div className="mt-3 space-y-1">
            <p>
              <span className="text-[var(--color-text)]">Cmd+T</span> new tab
            </p>
            <p>
              <span className="text-[var(--color-text)]">Cmd+W</span> close tab
            </p>
            <p>
              <span className="text-[var(--color-text)]">Cmd+1..9</span> jump tabs
            </p>
          </div>
        </div>
      </div>
    </aside>
  );
}
