import { useEffect, useRef } from "react";
import { Button } from "@/components/ui/Button";

interface ConfirmDialogProps {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  variant?: "danger" | "primary";
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmDialog({
  title,
  message,
  confirmLabel = "Close",
  cancelLabel = "Cancel",
  variant = "danger",
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  const callbacksRef = useRef({ onConfirm, onCancel });
  useEffect(() => {
    callbacksRef.current = { onConfirm, onCancel };
  });

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        event.preventDefault();
        event.stopPropagation();
        callbacksRef.current.onCancel();
      } else if (event.key === "Enter") {
        event.preventDefault();
        event.stopPropagation();
        callbacksRef.current.onConfirm();
      }
    }
    window.addEventListener("keydown", handleKeyDown, true);
    return () => window.removeEventListener("keydown", handleKeyDown, true);
  }, []);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm"
      onClick={(event) => {
        if (event.target === event.currentTarget) onCancel();
      }}
      role="dialog"
      data-testid="confirm-dialog"
    >
      <div className="w-full max-w-xs rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-4 shadow-xl">
        <p className="mb-1 text-sm font-medium">{title}</p>
        <p className="mb-4 text-xs text-[var(--color-text-muted)]">{message}</p>

        <div className="flex justify-end gap-2">
          <Button variant="ghost" size="sm" onClick={onCancel} data-testid="confirm-cancel">
            {cancelLabel}
          </Button>
          <Button
            size="sm"
            onClick={onConfirm}
            data-testid="confirm-ok"
            className={
              variant === "danger"
                ? "bg-red-600 text-white hover:bg-red-700"
                : ""
            }
          >
            {confirmLabel}
          </Button>
        </div>
      </div>
    </div>
  );
}
