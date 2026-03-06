import type { ButtonHTMLAttributes, PropsWithChildren } from "react";

type Variant = "primary" | "secondary" | "ghost" | "danger";
type Size = "sm" | "md" | "lg";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  size?: Size;
}

const BASE =
  "inline-flex items-center justify-center rounded-xl border font-medium transition disabled:cursor-not-allowed disabled:opacity-50";

const VARIANTS: Record<Variant, string> = {
  primary:
    "border-transparent bg-[var(--color-accent-strong)] text-black hover:bg-[var(--color-accent)]",
  secondary:
    "border-[var(--color-border-strong)] bg-white/4 text-[var(--color-text)] hover:bg-white/8",
  ghost:
    "border-transparent bg-transparent text-[var(--color-text-soft)] hover:bg-white/6 hover:text-[var(--color-text)]",
  danger:
    "border-transparent bg-[#c9555d] text-white hover:bg-[#d9646d]",
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
