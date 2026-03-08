import { HelpCircle, Plus, Settings, X } from "lucide-react";

interface TabEntry {
  id: string;
  title: string;
}

interface TabBarProps {
  tabs: TabEntry[];
  activeTabId: string;
  onSelect: (tabId: string) => void;
  onClose: (tabId: string) => void;
  onNewTab: () => void;
  onOpenSettings: () => void;
  onOpenShortcuts: () => void;
}

export function TabBar({
  tabs,
  activeTabId,
  onSelect,
  onClose,
  onNewTab,
  onOpenSettings,
  onOpenShortcuts,
}: TabBarProps) {
  return (
    <div
      className="flex h-10 shrink-0 select-none items-center gap-0 overflow-x-auto bg-[var(--color-surface)] pl-[72px] text-xs"
      data-tauri-drag-region
    >
      {tabs.map((tab, index) => {
        const isActive = tab.id === activeTabId;

        return (
          <button
            key={tab.id}
            data-testid={`tab-${index + 1}`}
            className={`group relative flex h-full items-center gap-2 px-4 transition-colors ${
              isActive
                ? "bg-[var(--color-surface-overlay)] text-[var(--color-text)]"
                : "text-[var(--color-text-muted)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text-soft)]"
            }`}
            onClick={() => onSelect(tab.id)}
          >
            <span className="truncate">{tab.title}</span>
            <span
              data-testid={`close-tab-${index + 1}`}
              className="flex h-4 w-4 items-center justify-center rounded opacity-0 transition-opacity hover:bg-[var(--color-surface-hover)] group-hover:opacity-100"
              role="button"
              tabIndex={-1}
              onClick={(event) => {
                event.stopPropagation();
                onClose(tab.id);
              }}
              onKeyDown={(event) => {
                if (event.key === "Enter") {
                  event.stopPropagation();
                  onClose(tab.id);
                }
              }}
            >
              <X size={12} />
            </span>
            {isActive ? (
              <span className="absolute bottom-0 left-2 right-2 h-[2px] rounded-t bg-[var(--color-accent)]" />
            ) : null}
          </button>
        );
      })}
      <button
        data-testid="new-tab-button"
        className="flex h-full items-center px-3 text-[var(--color-text-muted)] transition-colors hover:text-[var(--color-text)]"
        onClick={onNewTab}
        aria-label="New tab"
      >
        <Plus size={14} />
      </button>
      <div className="flex-1" data-tauri-drag-region />
      <button
        data-testid="shortcuts-button"
        className="flex h-full items-center px-3 text-[var(--color-text-muted)] transition-colors hover:text-[var(--color-text)]"
        onClick={onOpenShortcuts}
        aria-label="Keyboard shortcuts"
      >
        <HelpCircle size={14} />
      </button>
      <button
        data-testid="settings-button"
        className="flex h-full items-center px-3 text-[var(--color-text-muted)] transition-colors hover:text-[var(--color-text)]"
        onClick={onOpenSettings}
        aria-label="Settings"
      >
        <Settings size={14} />
      </button>
    </div>
  );
}
