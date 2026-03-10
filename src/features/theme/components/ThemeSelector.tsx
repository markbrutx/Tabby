import { Plus, Upload } from "lucide-react";
import { Button } from "@/components/ui/Button";
import type { ThemeDefinition } from "../domain/models";
import { ThemePreviewCard } from "./ThemePreviewCard";

interface ThemeSelectorProps {
  activeThemeId: string;
  themes: readonly ThemeDefinition[];
  onSelectTheme: (themeId: string) => void;
  onCreateTheme: () => void;
  onEditTheme: (themeId: string) => void;
  onImportTheme: () => void;
}

export function ThemeSelector({
  activeThemeId,
  themes,
  onSelectTheme,
  onCreateTheme,
  onEditTheme,
  onImportTheme,
}: ThemeSelectorProps) {
  const builtInThemes = themes.filter((t) => t.builtIn);
  const customThemes = themes.filter((t) => !t.builtIn);
  const isSystemMode = activeThemeId === "system";

  return (
    <div className="space-y-3">
      <label className="flex items-center gap-2 rounded-lg border border-[var(--color-border)] px-3 py-2">
        <input
          type="checkbox"
          checked={isSystemMode}
          onChange={(e) => {
            if (e.target.checked) {
              onSelectTheme("system");
            } else {
              const firstBuiltIn = builtInThemes[0];
              onSelectTheme(firstBuiltIn?.id ?? "midnight");
            }
          }}
          className="h-4 w-4 accent-[var(--color-accent)]"
        />
        <span className="text-sm text-[var(--color-text)]">
          System (auto-detect)
        </span>
      </label>

      <div className="grid grid-cols-3 gap-3">
        {builtInThemes.map((theme) => (
          <ThemePreviewCard
            key={theme.id}
            theme={theme}
            isActive={activeThemeId === theme.id}
            onSelect={() => onSelectTheme(theme.id)}
          />
        ))}
      </div>

      {customThemes.length > 0 && (
        <>
          <div className="border-t border-[var(--color-border)]" />
          <div className="grid grid-cols-3 gap-3">
            {customThemes.map((theme) => (
              <ThemePreviewCard
                key={theme.id}
                theme={theme}
                isActive={activeThemeId === theme.id}
                onSelect={() => onSelectTheme(theme.id)}
                onEdit={() => onEditTheme(theme.id)}
              />
            ))}
          </div>
        </>
      )}

      <div className="flex gap-2 pt-1">
        <Button variant="ghost" size="sm" onClick={onCreateTheme}>
          <Plus size={14} className="mr-1" />
          New Theme
        </Button>
        <Button variant="ghost" size="sm" onClick={onImportTheme}>
          <Upload size={14} className="mr-1" />
          Import
        </Button>
      </div>
    </div>
  );
}
