import { FolderOpen, X } from "lucide-react";
import { useEffect, useState } from "react";
import { Button } from "@/components/ui/Button";
import { DrawerOverlay } from "@/components/ui/DrawerOverlay";
import { Input } from "@/components/ui/Input";
import { Select } from "@/components/ui/Select";
import type {
  LayoutPreset,
  PaneProfile,
  WorkspaceSettings,
} from "@/features/workspace/domain";
import { LAYOUT_PRESET_CARDS, THEME_OPTIONS } from "@/features/workspace/presets";
import { pickDirectory } from "@/lib/pickDirectory";

interface SettingsDrawerProps {
  settings: WorkspaceSettings;
  profiles: PaneProfile[];
  onClose: () => void;
  onSave: (settings: WorkspaceSettings) => Promise<void>;
}

export function SettingsDrawer({
  settings,
  profiles,
  onClose,
  onSave,
}: SettingsDrawerProps) {
  const [draft, setDraft] = useState(settings);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    setDraft(settings);
  }, [settings]);

  async function handlePickDirectory() {
    const selected = await pickDirectory(draft.defaultWorkingDirectory);
    if (selected) {
      setDraft((current) => ({
        ...current,
        defaultWorkingDirectory: selected,
      }));
    }
  }

  async function handleSave() {
    setIsSaving(true);
    await onSave(draft);
    setIsSaving(false);
    onClose();
  }

  return (
    <DrawerOverlay side="right" maxWidth={480} onClose={onClose}>
      <div data-testid="settings-drawer" className="flex flex-1 flex-col">
        <div className="flex items-start justify-between gap-4">
          <div>
            <p className="text-xs uppercase tracking-[0.28em] text-[var(--color-text-muted)]">
              Workspace Settings
            </p>
            <h2 className="mt-2 text-2xl font-semibold">Default launch profile</h2>
            <p className="mt-2 text-sm text-[var(--color-text-soft)]">
              These defaults are used for the first tab and every new workspace you
              launch from the sidebar.
            </p>
          </div>
          <Button variant="ghost" size="sm" onClick={onClose}>
            <X size={16} />
          </Button>
        </div>

        <div className="mt-6 space-y-5 overflow-y-auto pr-1">
          <label className="block">
            <span className="mb-2 block text-sm text-[var(--color-text-soft)]">
              Default layout
            </span>
            <Select
              data-testid="settings-layout"
              value={draft.defaultLayout}
              onChange={(event) =>
                setDraft((current) => ({
                  ...current,
                  defaultLayout: event.target.value as LayoutPreset,
                }))
              }
            >
              {LAYOUT_PRESET_CARDS.map((card) => (
                <option key={card.preset} value={card.preset}>
                  {card.preset}
                </option>
              ))}
            </Select>
          </label>

          <label className="block">
            <span className="mb-2 block text-sm text-[var(--color-text-soft)]">
              Default profile
            </span>
            <Select
              data-testid="settings-profile"
              value={draft.defaultProfileId}
              onChange={(event) =>
                setDraft((current) => ({
                  ...current,
                  defaultProfileId: event.target.value,
                }))
              }
            >
              {profiles.map((profile) => (
                <option key={profile.id} value={profile.id}>
                  {profile.label}
                </option>
              ))}
            </Select>
          </label>

          {draft.defaultProfileId === "custom" ? (
            <label className="block">
              <span className="mb-2 block text-sm text-[var(--color-text-soft)]">
                Default custom command
              </span>
              <Input
                data-testid="settings-custom-command"
                value={draft.defaultCustomCommand}
                onChange={(event) =>
                  setDraft((current) => ({
                    ...current,
                    defaultCustomCommand: event.target.value,
                  }))
                }
                placeholder="npm run dev"
              />
            </label>
          ) : null}

          <div className="block">
            <span className="mb-2 block text-sm text-[var(--color-text-soft)]">
              Default working directory
            </span>
            <div className="flex gap-2">
              <Input
                data-testid="settings-working-directory"
                value={draft.defaultWorkingDirectory}
                onChange={(event) =>
                  setDraft((current) => ({
                    ...current,
                    defaultWorkingDirectory: event.target.value,
                  }))
                }
                placeholder="~/projects/tabby"
              />
              <Button variant="secondary" onClick={() => void handlePickDirectory()}>
                <FolderOpen size={16} />
              </Button>
            </div>
          </div>

          <label className="block">
            <span className="mb-2 block text-sm text-[var(--color-text-soft)]">
              Terminal font size
            </span>
            <Input
              data-testid="settings-font-size"
              type="range"
              min={11}
              max={20}
              step={1}
              value={draft.fontSize}
              onChange={(event) =>
                setDraft((current) => ({
                  ...current,
                  fontSize: Number(event.target.value),
                }))
              }
            />
            <span className="mt-2 block text-xs text-[var(--color-text-muted)]">
              {draft.fontSize}px
            </span>
          </label>

          <label className="block">
            <span className="mb-2 block text-sm text-[var(--color-text-soft)]">
              Theme
            </span>
            <Select
              data-testid="settings-theme"
              value={draft.theme}
              onChange={(event) =>
                setDraft((current) => ({
                  ...current,
                  theme: event.target.value as WorkspaceSettings["theme"],
                }))
              }
            >
              {THEME_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </Select>
          </label>

          <label className="flex items-center justify-between rounded-2xl border border-[var(--color-border)] bg-white/3 px-4 py-3">
            <div>
              <span className="block text-sm font-medium">Launch fullscreen</span>
              <span className="block text-xs text-[var(--color-text-muted)]">
                Matches the macOS-first behavior from the spec.
              </span>
            </div>
            <input
              data-testid="settings-fullscreen"
              type="checkbox"
              checked={draft.launchFullscreen}
              onChange={(event) =>
                setDraft((current) => ({
                  ...current,
                  launchFullscreen: event.target.checked,
                }))
              }
              className="h-5 w-5 accent-[var(--color-accent-strong)]"
            />
          </label>
        </div>

        <div className="mt-6 flex justify-end gap-3">
          <Button variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button
            data-testid="save-settings"
            disabled={isSaving}
            onClick={() => void handleSave()}
          >
            Save defaults
          </Button>
        </div>
      </div>
    </DrawerOverlay>
  );
}
