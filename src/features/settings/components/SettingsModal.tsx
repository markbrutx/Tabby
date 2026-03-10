import { X } from "lucide-react";
import { useEffect, useState } from "react";
import { useEscapeKey } from "@/hooks/useEscapeKey";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import type { ProfileReadModel, SettingsReadModel } from "@/features/settings/domain/models";
import { useThemeStore } from "@/features/theme/application/themeStore";
import { ThemeSelector } from "@/features/theme/components/ThemeSelector";
import { ThemeEditorModal } from "@/features/theme/components/ThemeEditorModal";

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
  const [editorThemeId, setEditorThemeId] = useState<string | null | undefined>(
    undefined,
  );
  const themes = useThemeStore((s) => s.themes);
  const importTheme = useThemeStore((s) => s.importTheme);

  useEffect(() => {
    setDraft(settings);
  }, [settings]);

  useEscapeKey(onClose);

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

  function handleImportTheme() {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json";
    input.onchange = () => {
      const file = input.files?.[0];
      if (!file) return;
      const reader = new FileReader();
      reader.onload = () => {
        try {
          const imported = importTheme(reader.result as string);
          setDraft((current) => ({ ...current, theme: imported.id }));
        } catch {
          // Import failed — invalid file, no action needed
        }
      };
      reader.readAsText(file);
    };
    input.click();
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
        className="w-full max-w-xl rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-2xl"
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
              Interface font size
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

          <div>
            <span className="mb-1.5 block text-sm text-[var(--color-text-soft)]">
              Theme
            </span>
            <ThemeSelector
              activeThemeId={draft.theme}
              themes={themes}
              onSelectTheme={(id) =>
                setDraft((current) => ({ ...current, theme: id }))
              }
              onCreateTheme={() => setEditorThemeId(null)}
              onEditTheme={(id) => setEditorThemeId(id)}
              onImportTheme={handleImportTheme}
            />
          </div>

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

      {editorThemeId !== undefined && (
        <ThemeEditorModal
          themeId={editorThemeId}
          onClose={() => setEditorThemeId(undefined)}
        />
      )}
    </div>
  );
}
