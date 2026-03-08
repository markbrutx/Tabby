import { FolderOpen, X } from "lucide-react";
import { useEffect, useState } from "react";
import { useEscapeKey } from "@/hooks/useEscapeKey";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Select } from "@/components/ui/Select";
import { CUSTOM_PROFILE_ID } from "@/features/workspace/domain/models";
import type { LayoutPreset } from "@/features/settings/domain/models";
import type { ProfileReadModel, SettingsReadModel } from "@/features/settings/domain/models";
import { pickDirectory } from "@/lib/pickDirectory";

const LAYOUT_OPTIONS: { value: LayoutPreset; label: string }[] = [
  { value: "1x1", label: "1x1 (single)" },
  { value: "1x2", label: "1x2 (side-by-side)" },
  { value: "2x2", label: "2x2 (quad)" },
  { value: "2x3", label: "2x3 (six)" },
  { value: "3x3", label: "3x3 (nine)" },
];

const THEME_OPTIONS = [
  { value: "system", label: "System" },
  { value: "dawn", label: "Dawn (light)" },
  { value: "midnight", label: "Midnight (dark)" },
];

interface SettingsModalProps {
  settings: SettingsReadModel;
  profiles: readonly ProfileReadModel[];
  onClose: () => void;
  onSave: (settings: SettingsReadModel) => Promise<void>;
  onReset: () => Promise<void>;
}

export function SettingsModal({
  settings,
  profiles,
  onClose,
  onSave,
  onReset,
}: SettingsModalProps) {
  const [draft, setDraft] = useState(settings);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    setDraft(settings);
  }, [settings]);

  useEscapeKey(onClose);

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

  async function handleReset() {
    setIsSaving(true);
    await onReset();
    setIsSaving(false);
    onClose();
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
      onClick={(event) => {
        if (event.target === event.currentTarget) onClose();
      }}
      role="dialog"
    >
      <div
        data-testid="settings-modal"
        className="w-full max-w-md rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-2xl"
      >
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold">Settings</h2>
          <button
            className="rounded p-1 text-[var(--color-text-muted)] hover:bg-[var(--color-surface-hover)]"
            onClick={onClose}
          >
            <X size={16} />
          </button>
        </div>

        <div className="mt-5 space-y-4">
          <label className="block">
            <span className="mb-1.5 block text-sm text-[var(--color-text-soft)]">
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
              {LAYOUT_OPTIONS.map((opt) => (
                <option key={opt.value} value={opt.value}>
                  {opt.label}
                </option>
              ))}
            </Select>
          </label>

          <label className="block">
            <span className="mb-1.5 block text-sm text-[var(--color-text-soft)]">
              Default profile
            </span>
            <Select
              data-testid="settings-profile"
              value={draft.defaultTerminalProfileId}
              onChange={(event) =>
                setDraft((current) => ({
                  ...current,
                  defaultTerminalProfileId: event.target.value,
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

          {draft.defaultTerminalProfileId === CUSTOM_PROFILE_ID ? (
            <label className="block">
              <span className="mb-1.5 block text-sm text-[var(--color-text-soft)]">
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
            <span className="mb-1.5 block text-sm text-[var(--color-text-soft)]">
              Working directory
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
                placeholder="Not set — you'll choose each time"
              />
              <Button variant="secondary" onClick={() => void handlePickDirectory()}>
                <FolderOpen size={14} />
              </Button>
            </div>
          </div>

          <label className="block">
            <span className="mb-1.5 block text-sm text-[var(--color-text-soft)]">
              Font size
            </span>
            <div className="flex items-center gap-3">
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
              <span className="text-xs text-[var(--color-text-muted)]">
                {draft.fontSize}px
              </span>
            </div>
          </label>

          <label className="block">
            <span className="mb-1.5 block text-sm text-[var(--color-text-soft)]">
              Theme
            </span>
            <Select
              data-testid="settings-theme"
              value={draft.theme}
              onChange={(event) =>
                setDraft((current) => ({
                  ...current,
                  theme: event.target.value as SettingsReadModel["theme"],
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

          <label className="flex items-center justify-between rounded-lg border border-[var(--color-border)] px-3 py-2">
            <span className="text-sm">Launch fullscreen</span>
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
              className="h-4 w-4 accent-[var(--color-accent)]"
            />
          </label>
        </div>

        <div className="mt-5 flex items-center gap-2">
          <Button
            variant="danger"
            size="sm"
            disabled={isSaving}
            onClick={() => void handleReset()}
          >
            Reset to defaults
          </Button>
          <div className="flex-1" />
          <Button variant="ghost" onClick={onClose}>
            Cancel
          </Button>
          <Button
            data-testid="save-settings"
            disabled={isSaving}
            onClick={() => void handleSave()}
          >
            Save
          </Button>
        </div>
      </div>
    </div>
  );
}
