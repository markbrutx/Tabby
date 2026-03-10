interface ShortcutBadgeProps {
  keys: string[];
}

export function ShortcutBadge({ keys }: ShortcutBadgeProps) {
  return (
    <span className="inline-flex items-center gap-0.5 opacity-80 transition-opacity group-hover:opacity-100">
      {keys.map((key, i) => (
        <kbd
          key={i}
          className="inline-flex min-w-[1.25rem] items-center justify-center rounded border border-[var(--color-border)] bg-transparent px-1 py-0.5 text-[10px] font-medium text-[var(--color-text-muted)] transition-colors group-hover:border-[var(--color-border-strong)] group-hover:text-[var(--color-text-soft)]"
        >
          {key}
        </kbd>
      ))}
    </span>
  );
}
