import {
  Command,
  FolderTree,
  Settings2,
  X,
} from "lucide-react";
import brandIcon from "@/assets/tabby-brand.png";
import { Button } from "@/components/ui/Button";
import { DrawerOverlay } from "@/components/ui/DrawerOverlay";
import type { LayoutPreset, WorkspaceSettings } from "@/features/workspace/domain";
import { LAYOUT_PRESET_CARDS } from "@/features/workspace/presets";

interface AppSidebarProps {
  isWorking: boolean;
  settings: WorkspaceSettings;
  onCreateTab: (preset: LayoutPreset) => void;
  onOpenSettings: () => void;
  onClose: () => void;
}

export function AppSidebar({
  isWorking,
  settings,
  onCreateTab,
  onOpenSettings,
  onClose,
}: AppSidebarProps) {
  return (
    <DrawerOverlay side="left" maxWidth={360} onClose={onClose}>
      <div className="flex items-start justify-between gap-3">
        <div
          className="flex-1 rounded-[24px] border border-[var(--color-border)] p-5"
          style={{ background: "var(--gradient-brand-card)" }}
        >
          <p className="text-xs uppercase tracking-[0.35em] text-[var(--color-text-muted)]">
            Terminal Workspace
          </p>
          <div className="mt-3 flex items-center gap-3">
            <div className="flex h-12 w-12 items-center justify-center rounded-2xl bg-[var(--color-brand-mark-bg)] p-0.5">
              <img
                src={brandIcon}
                alt=""
                className="h-full w-full rounded-[14px] object-cover"
              />
            </div>
            <div>
              <h1 className="text-3xl font-semibold tracking-tight">Tabby</h1>
              <p className="text-sm text-[var(--color-text-soft)]">
                Handy-inspired command decks for Codex, Claude and plain shells.
              </p>
            </div>
          </div>
        </div>
        <Button variant="ghost" size="sm" onClick={onClose}>
          <X size={16} />
        </Button>
      </div>

      <div className="mt-5 flex-1 overflow-y-auto">
        <div className="mb-3 flex items-center justify-between">
          <p className="text-xs uppercase tracking-[0.25em] text-[var(--color-text-muted)]">
            Launchpads
          </p>
          <span className="rounded-full bg-[var(--color-badge-bg)] px-3 py-1 text-[11px] text-[var(--color-text-soft)]">
            default {settings.defaultLayout}
          </span>
        </div>

        <div className="space-y-2">
          {LAYOUT_PRESET_CARDS.map((card) => {
            const Icon = card.icon;
            return (
              <button
                key={card.preset}
                data-testid={`launchpad-${card.preset}`}
                className="surface-muted w-full rounded-2xl p-4 text-start transition hover:border-[var(--color-accent-strong)] hover:bg-[var(--color-surface-hover)]"
                onClick={() => {
                  onCreateTab(card.preset);
                  onClose();
                }}
                disabled={isWorking}
              >
                <div className="flex items-start gap-3">
                  <div className="mt-1 flex h-9 w-9 items-center justify-center rounded-xl bg-[var(--color-accent-soft)] text-[var(--color-accent)]">
                    <Icon size={18} />
                  </div>
                  <div className="min-w-0">
                    <div className="flex items-center gap-2">
                      <p className="font-medium">{card.title}</p>
                      <span className="rounded-full bg-[var(--color-badge-bg)] px-2 py-0.5 text-[10px] uppercase tracking-[0.2em] text-[var(--color-text-muted)]">
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

        <div className="mt-5 rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface-contrast)] p-4">
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
      </div>

      <div className="space-y-3 pt-5">
        <Button
          data-testid="open-settings"
          variant="secondary"
          size="lg"
          className="w-full justify-between"
          onClick={() => {
            onOpenSettings();
            onClose();
          }}
        >
          Workspace settings
          <Settings2 size={16} />
        </Button>

        <div className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface-overlay)] p-4 text-sm text-[var(--color-text-soft)]">
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
    </DrawerOverlay>
  );
}
