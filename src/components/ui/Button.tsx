import type { ButtonHTMLAttributes, PropsWithChildren } from "react";

type Variant = "primary" | "secondary" | "ghost" | "danger";
type Size = "sm" | "md" | "lg";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  size?: Size;
}

const BASE =
  "inline-flex items-center justify-center rounded-lg border font-medium transition disabled:cursor-not-allowed disabled:opacity-50";

const VARIANTS: Record<Variant, string> = {
  primary:
    "border-transparent bg-[var(--color-accent-strong)] text-white hover:bg-[var(--color-accent)]",
  secondary:
    "border-[var(--color-border-strong)] bg-[var(--color-surface-overlay)] text-[var(--color-text)] hover:bg-[var(--color-surface-hover)]",
  ghost:
    "border-transparent bg-transparent text-[var(--color-text-soft)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]",
  danger:
    "border-transparent bg-[var(--color-danger)] text-white hover:bg-[var(--color-danger-strong)]",
};

const SIZES: Record<Size, string> = {
  sm: "h-8 px-3 text-xs",
  md: "h-10 px-4 text-sm",
  lg: "h-12 px-5 text-sm",
};

export function Button({
  children,
  className = "",
  variant = "primary",
  size = "md",
  ...props
}: PropsWithChildren<ButtonProps>) {
  return (
    <button
      className={`${BASE} ${VARIANTS[variant]} ${SIZES[size]} ${className}`}
      {...props}
    >
      {children}
    </button>
  );
}
