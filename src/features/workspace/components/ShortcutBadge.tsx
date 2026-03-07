interface ShortcutBadgeProps {
  keys: string[];
}

export function ShortcutBadge({ keys }: ShortcutBadgeProps) {
  return (
    <span className="inline-flex items-center gap-0.5">
      {keys.map((key, i) => (
        <kbd
          key={i}
          className="inline-flex min-w-[1.5rem] items-center justify-center rounded border border-[var(--color-border)] bg-[var(--color-surface-hover)] px-1.5 py-0.5 text-[11px] font-medium text-[var(--color-text-soft)]"
        >
          {key}
        </kbd>
      ))}
    </span>
  );
}
