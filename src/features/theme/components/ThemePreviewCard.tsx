import { Pencil } from "lucide-react";
import type { ThemeDefinition } from "../domain/models";

interface ThemePreviewCardProps {
  theme: ThemeDefinition;
  isActive: boolean;
  onSelect: () => void;
  onEdit?: () => void;
}

const SWATCH_KEYS = [
  "bg",
  "surface",
  "accent",
  "text",
  "danger",
  "tokenKeyword",
] as const;

export function ThemePreviewCard({
  theme,
  isActive,
  onSelect,
  onEdit,
}: ThemePreviewCardProps) {
  return (
    <button
      type="button"
      data-testid={`theme-card-${theme.id}`}
      onClick={onSelect}
      className={`group relative w-[180px] rounded-lg border p-3 text-left transition ${
        isActive
          ? "border-[var(--color-accent)] ring-1 ring-[var(--color-accent)]"
          : "border-[var(--color-border)] hover:border-[var(--color-border-strong)]"
      } bg-[var(--color-surface-overlay)]`}
    >
      <div className="flex items-center justify-between">
        <span className="truncate text-sm font-medium text-[var(--color-text)]">
          {theme.name}
        </span>
        <span
          className={`shrink-0 rounded-full px-1.5 py-0.5 text-[10px] font-medium leading-none ${
            theme.kind === "dark"
              ? "bg-[var(--color-surface-hover)] text-[var(--color-text-muted)]"
              : "bg-[var(--color-accent-soft)] text-[var(--color-accent-strong)]"
          }`}
        >
          {theme.kind === "dark" ? "Dark" : "Light"}
        </span>
      </div>

      <div className="mt-2.5 flex gap-1.5">
        {SWATCH_KEYS.map((key) => (
          <div
            key={key}
            className="h-5 w-5 rounded-sm border border-black/10"
            style={{ backgroundColor: theme.colors[key] }}
            title={key}
          />
        ))}
      </div>

      {onEdit && (
        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation();
            onEdit();
          }}
          className="absolute right-1.5 top-1.5 hidden rounded p-1 text-[var(--color-text-muted)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)] group-hover:block"
          aria-label={`Edit ${theme.name}`}
        >
          <Pencil size={12} />
        </button>
      )}
    </button>
  );
}
