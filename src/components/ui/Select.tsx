import type { SelectHTMLAttributes } from "react";

export function Select({
  className = "",
  children,
  ...props
}: SelectHTMLAttributes<HTMLSelectElement>) {
  return (
    <select
      className={`h-10 w-full rounded-lg border border-[var(--color-border)] bg-[var(--color-surface-overlay)] px-3 text-sm text-[var(--color-text)] outline-none focus:border-[var(--color-accent-strong)] ${className}`}
      {...props}
    >
      {children}
    </select>
  );
}
