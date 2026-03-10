import type { ReactNode } from "react";
import { AlertOctagon } from "lucide-react";

export interface PaneErrorStateProps {
    title?: string;
    message: string;
    action?: ReactNode;
}

export function PaneErrorState({
    title = "Error",
    message,
    action,
}: PaneErrorStateProps) {
    return (
        <div
            className="flex h-full w-full items-center justify-center p-6 bg-[var(--color-bg)]"
            data-testid="pane-error-state"
        >
            <div className="flex w-full max-w-sm flex-col items-center text-center">
                {/* Icon with glowing background */}
                <div className="relative mb-6 flex items-center justify-center">
                    <div className="absolute inset-0 animate-pulse rounded-full bg-[var(--color-danger-soft)] blur-xl" />
                    <div className="relative flex h-14 w-14 items-center justify-center rounded-2xl border border-[var(--color-danger-soft)] bg-[var(--color-surface)] shadow-lg">
                        <AlertOctagon size={28} className="text-[var(--color-danger)]" />
                    </div>
                </div>

                {/* Text content */}
                <h3 className="mb-2 text-lg font-semibold text-[var(--color-text)]">
                    {title}
                </h3>

                {/* Error message block */}
                <div className="mb-6 w-full rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-sm">
                    <p className="whitespace-pre-wrap break-words text-left text-xs font-medium text-[var(--color-text-muted)] font-mono leading-relaxed">
                        {message}
                    </p>
                </div>

                {/* Action button */}
                {action ? <div>{action}</div> : null}
            </div>
        </div>
    );
}
