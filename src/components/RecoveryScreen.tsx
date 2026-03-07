import { AlertTriangle, RotateCcw } from "lucide-react";
import { Button } from "@/components/ui/Button";

interface RecoveryScreenProps {
  title: string;
  message: string;
  onRetry?: () => void;
}

export function RecoveryScreen({
  title,
  message,
  onRetry,
}: RecoveryScreenProps) {
  return (
    <div className="flex min-h-screen items-center justify-center p-8 text-[var(--color-text)]">
      <div className="w-full max-w-md rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-6">
        <div className="flex items-center gap-3 text-[var(--color-warning)]">
          <AlertTriangle size={18} />
          <p className="text-sm font-medium">{title}</p>
        </div>

        <p className="mt-3 text-sm text-[var(--color-text-soft)]">{message}</p>

        {onRetry ? (
          <Button className="mt-4" onClick={onRetry}>
            <RotateCcw size={14} />
            Retry
          </Button>
        ) : null}
      </div>
    </div>
  );
}
