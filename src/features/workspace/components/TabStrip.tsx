import { X } from "lucide-react";
import type { TabSnapshot } from "@/features/workspace/domain";
import { Button } from "@/components/ui/Button";

interface TabStripProps {
  tabs: TabSnapshot[];
  activeTabId: string;
  isWorking: boolean;
  onSelect: (tabId: string) => void;
  onClose: (tabId: string) => void;
}

export function TabStrip({
  tabs,
  activeTabId,
  isWorking,
  onSelect,
  onClose,
}: TabStripProps) {
  return (
    <div className="surface-muted flex items-center gap-2 overflow-x-auto rounded-2xl p-2">
      {tabs.map((tab, index) => {
        const isActive = tab.id === activeTabId;

        return (
          <button
            key={tab.id}
            data-testid={`tab-${index + 1}`}
            className={`group flex min-w-[180px] items-center gap-3 rounded-2xl border px-4 py-3 text-start transition ${
              isActive
                ? "border-[var(--color-accent-strong)] bg-[var(--color-accent-soft)]"
                : "border-transparent bg-[var(--color-surface-overlay)] hover:border-[var(--color-border-strong)] hover:bg-[var(--color-surface-hover)]"
            }`}
            onClick={() => onSelect(tab.id)}
          >
            <div className="min-w-0 flex-1">
              <p className="truncate text-sm font-medium">{tab.title}</p>
              <p className="mt-1 text-xs uppercase tracking-[0.2em] text-[var(--color-text-muted)]">
                {index + 1}. {tab.preset} · {tab.panes.length} panes
              </p>
            </div>
            <Button
              data-testid={`close-tab-${index + 1}`}
              variant={isActive ? "ghost" : "secondary"}
              size="sm"
              disabled={isWorking}
              onClick={(event) => {
                event.stopPropagation();
                onClose(tab.id);
              }}
            >
              <X size={14} />
            </Button>
          </button>
        );
      })}
    </div>
  );
}
