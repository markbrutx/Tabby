import { useCallback, useEffect, useMemo, useState } from "react";
import { X, Trash2 } from "lucide-react";
import { useEscapeKey } from "@/hooks/useEscapeKey";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { Select } from "@/components/ui/Select";
import { useThemeStore } from "../application/themeStore";
import { ColorTokenInput } from "./ColorTokenInput";
import type { ThemeColorTokens, ThemeKind } from "../domain/models";

interface ThemeEditorModalProps {
  readonly themeId: string | null;
  readonly onClose: () => void;
}

interface TokenSection {
  readonly label: string;
  readonly tokens: readonly {
    readonly key: keyof ThemeColorTokens;
    readonly label: string;
  }[];
}

const TOKEN_SECTIONS: readonly TokenSection[] = [
  {
    label: "Base",
    tokens: [
      { key: "bg", label: "Background" },
      { key: "surface", label: "Surface" },
      { key: "surfaceOverlay", label: "Surface Overlay" },
      { key: "surfaceHover", label: "Surface Hover" },
    ],
  },
  {
    label: "Text",
    tokens: [
      { key: "text", label: "Text" },
      { key: "textSoft", label: "Text Soft" },
      { key: "textMuted", label: "Text Muted" },
    ],
  },
  {
    label: "Accent",
    tokens: [
      { key: "accent", label: "Accent" },
      { key: "accentStrong", label: "Accent Strong" },
      { key: "accentSoft", label: "Accent Soft" },
    ],
  },
  {
    label: "Status",
    tokens: [
      { key: "danger", label: "Danger" },
      { key: "dangerStrong", label: "Danger Strong" },
      { key: "dangerSoft", label: "Danger Soft" },
      { key: "warning", label: "Warning" },
    ],
  },
  {
    label: "Borders",
    tokens: [
      { key: "border", label: "Border" },
      { key: "borderStrong", label: "Border Strong" },
      { key: "scrollbar", label: "Scrollbar" },
    ],
  },
  {
    label: "Syntax",
    tokens: [
      { key: "tokenKeyword", label: "Keyword" },
      { key: "tokenString", label: "String" },
      { key: "tokenComment", label: "Comment" },
      { key: "tokenNumber", label: "Number" },
      { key: "tokenType", label: "Type" },
      { key: "tokenPunctuation", label: "Punctuation" },
    ],
  },
];

export function ThemeEditorModal({ themeId, onClose }: ThemeEditorModalProps) {
  const store = useThemeStore();
  const { draft, themes } = store;

  const [baseColors, setBaseColors] = useState<ThemeColorTokens | null>(null);
  const openEditor = store.openEditor;

  useEffect(() => {
    openEditor(themeId);
  }, [themeId, openEditor]);

  useEffect(() => {
    if (draft && !baseColors) {
      setBaseColors(draft.colors);
    }
  }, [draft, baseColors]);

  const handleCancel = useCallback(() => {
    store.discardDraft();
    onClose();
  }, [store, onClose]);

  useEscapeKey(handleCancel);

  const handleSave = useCallback(() => {
    store.saveDraft();
    onClose();
  }, [store, onClose]);

  const handleDelete = useCallback(() => {
    if (themeId) {
      store.deleteTheme(themeId);
      onClose();
    }
  }, [store, themeId, onClose]);

  const handleNameChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      store.updateDraftMeta({ name: event.target.value });
    },
    [store],
  );

  const handleKindChange = useCallback(
    (kind: ThemeKind) => {
      store.updateDraftMeta({ kind });
    },
    [store],
  );

  const handleCloneFrom = useCallback(
    (event: React.ChangeEvent<HTMLSelectElement>) => {
      const sourceId = event.target.value;
      if (!sourceId) return;
      const source = themes.find((t) => t.id === sourceId);
      if (source) {
        store.updateDraft({ ...source.colors });
        setBaseColors(source.colors);
      }
    },
    [store, themes],
  );

  const handleTokenChange = useCallback(
    (key: keyof ThemeColorTokens, value: string) => {
      store.updateDraft({ [key]: value });
    },
    [store],
  );

  const isExistingCustom = useMemo(() => {
    if (!themeId) return false;
    const target = themes.find((t) => t.id === themeId);
    return target ? !target.builtIn : false;
  }, [themeId, themes]);

  const isNewTheme = themeId === null;
  const title = isNewTheme ? "New Theme" : "Edit Theme";

  if (!draft || !baseColors) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
      onClick={(event) => {
        if (event.target === event.currentTarget) handleCancel();
      }}
      role="dialog"
    >
      <div className="flex max-h-[85vh] w-full max-w-2xl flex-col rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] shadow-2xl">
        {/* Header */}
        <div className="flex shrink-0 items-center justify-between border-b border-[var(--color-border)] p-4">
          <h2 className="text-lg font-semibold">{title}</h2>
          <button
            className="rounded p-1 text-[var(--color-text-muted)] hover:bg-[var(--color-surface-hover)]"
            onClick={handleCancel}
          >
            <X size={16} />
          </button>
        </div>

        {/* Meta section */}
        <div className="shrink-0 space-y-3 border-b border-[var(--color-border)] p-4">
          <label className="block">
            <span className="mb-1.5 block text-sm text-[var(--color-text-soft)]">
              Theme name
            </span>
            <Input
              value={draft.name}
              onChange={handleNameChange}
              placeholder="My Theme"
            />
          </label>

          <div>
            <span className="mb-1.5 block text-sm text-[var(--color-text-soft)]">
              Kind
            </span>
            <div className="flex gap-2">
              <Button
                size="sm"
                variant={draft.kind === "dark" ? "primary" : "secondary"}
                onClick={() => handleKindChange("dark")}
              >
                Dark
              </Button>
              <Button
                size="sm"
                variant={draft.kind === "light" ? "primary" : "secondary"}
                onClick={() => handleKindChange("light")}
              >
                Light
              </Button>
            </div>
          </div>

          <label className="block">
            <span className="mb-1.5 block text-sm text-[var(--color-text-soft)]">
              Clone from
            </span>
            <Select onChange={handleCloneFrom} value="">
              <option value="">Select a theme...</option>
              {themes.map((t) => (
                <option key={t.id} value={t.id}>
                  {t.name}
                </option>
              ))}
            </Select>
          </label>
        </div>

        {/* Scrollable token editor */}
        <div className="flex-1 overflow-y-auto p-4">
          {TOKEN_SECTIONS.map((section) => (
            <div key={section.label} className="mb-4">
              <h3 className="mb-1 text-xs font-semibold uppercase tracking-wider text-[var(--color-text-muted)]">
                {section.label}
              </h3>
              <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-surface-overlay)] px-3 py-1">
                {section.tokens.map((token) => (
                  <ColorTokenInput
                    key={token.key}
                    label={token.label}
                    value={draft.colors[token.key]}
                    baseValue={baseColors[token.key]}
                    onChange={(newValue) =>
                      handleTokenChange(token.key, newValue)
                    }
                  />
                ))}
              </div>
            </div>
          ))}
        </div>

        {/* Footer */}
        <div className="flex shrink-0 items-center gap-2 border-t border-[var(--color-border)] p-4">
          {isExistingCustom && (
            <Button variant="danger" size="sm" onClick={handleDelete}>
              <Trash2 size={14} className="mr-1" />
              Delete
            </Button>
          )}
          <div className="flex-1" />
          <Button variant="ghost" onClick={handleCancel}>
            Cancel
          </Button>
          <Button onClick={handleSave}>Save</Button>
        </div>
      </div>
    </div>
  );
}
