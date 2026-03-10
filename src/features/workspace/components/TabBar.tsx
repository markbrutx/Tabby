import { useCallback, useEffect, useRef, useState } from "react";
import { HelpCircle, Plus, Settings, X } from "lucide-react";
import { ShortcutBadge } from "@/components/ShortcutBadge";
import { isTauriRuntime } from "@/lib/runtime";

interface TabEntry {
  id: string;
  title: string;
  isWizard?: boolean;
}

interface TabBarProps {
  tabs: TabEntry[];
  activeTabId: string;
  onSelect: (tabId: string) => void;
  onClose: (tabId: string) => void;
  onRename: (tabId: string, title: string) => void;
  onNewTab: () => void;
  showNewTab?: boolean;
  onOpenSettings: () => void;
  onOpenShortcuts: () => void;
}

export function TabBar({
  tabs,
  activeTabId,
  onSelect,
  onClose,
  onRename,
  onNewTab,
  showNewTab = true,
  onOpenSettings,
  onOpenShortcuts,
}: TabBarProps) {
  const [editingTabId, setEditingTabId] = useState<string | null>(null);
  const [editValue, setEditValue] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (editingTabId && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [editingTabId]);

  const commitRename = useCallback(() => {
    if (editingTabId && editValue.trim()) {
      onRename(editingTabId, editValue.trim());
    }
    setEditingTabId(null);
    setEditValue("");
  }, [editingTabId, editValue, onRename]);

  const cancelRename = useCallback(() => {
    setEditingTabId(null);
    setEditValue("");
  }, []);

  const startEditing = useCallback((tab: TabEntry) => {
    setEditingTabId(tab.id);
    setEditValue(tab.title);
  }, []);

  const handleDragStart = useCallback((e: React.MouseEvent) => {
    if (e.button !== 0) return;
    const target = e.target as HTMLElement;
    if (target.closest("button, input, a, [role='button']")) return;
    if (!isTauriRuntime()) return;
    void import("@tauri-apps/api/window").then(({ getCurrentWindow }) => {
      void getCurrentWindow().startDragging();
    });
  }, []);

  return (
    <div
      className="flex h-11 shrink-0 select-none items-center border-b border-[var(--color-border)] bg-[var(--color-bg)] pl-[72px] pr-2 text-xs"
      data-tauri-drag-region
      onMouseDown={handleDragStart}
    >
      {import.meta.env.DEV && (
        <span className="mr-1 rounded bg-amber-500/20 px-1.5 py-0.5 font-mono text-[10px] font-bold text-amber-400">
          DEV
        </span>
      )}
      <div className="flex items-center gap-1.5 overflow-x-auto px-2">
        {tabs.map((tab, index) => {
          const isActive = tab.id === activeTabId;
          const isEditing = editingTabId === tab.id;

          return (
            <button
              key={tab.id}
              data-testid={`tab-${index + 1}`}
              className={`group flex h-7 items-center gap-2 rounded-md px-3 transition-colors ${isActive
                ? "bg-[var(--color-surface-hover)] text-[var(--color-text)] shadow-sm ring-1 ring-[var(--color-border)]"
                : "text-[var(--color-text-muted)] hover:bg-[var(--color-surface-overlay)] hover:text-[var(--color-text-soft)]"
                }`}
              onClick={() => {
                if (!isEditing) {
                  onSelect(tab.id);
                }
              }}
              onDoubleClick={() => {
                if (!tab.isWizard) {
                  startEditing(tab);
                }
              }}
            >
              {isEditing ? (
                <input
                  ref={inputRef}
                  data-testid={`tab-rename-input-${index + 1}`}
                  className="w-24 rounded bg-[var(--color-surface)] px-1.5 py-0.5 text-xs text-[var(--color-text)] outline-none ring-1 ring-[var(--color-accent)] focus:ring-[var(--color-accent-strong)]"
                  value={editValue}
                  maxLength={64}
                  onChange={(event) => setEditValue(event.target.value)}
                  onKeyDown={(event) => {
                    if (event.key === "Enter") {
                      commitRename();
                    } else if (event.key === "Escape") {
                      cancelRename();
                    }
                    event.stopPropagation();
                  }}
                  onBlur={commitRename}
                  onClick={(event) => event.stopPropagation()}
                />
              ) : (
                <span className="truncate">{tab.title}</span>
              )}
              {(!tab.isWizard || tabs.length > 1) ? (
                <span
                  data-testid={`close-tab-${index + 1}`}
                  className={`flex h-4 w-4 items-center justify-center rounded transition-opacity ${isActive
                    ? "opacity-60 hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-danger)] hover:opacity-100"
                    : "opacity-0 hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-danger)] group-hover:opacity-100"
                    }`}
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
                  <X size={12} strokeWidth={2.5} />
                </span>
              ) : null}
            </button>
          );
        })}
        {showNewTab ? (
          <button
            data-testid="new-tab-button"
            className="ml-1 flex h-7 w-7 items-center justify-center rounded-md text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]"
            onClick={onNewTab}
            aria-label="New tab"
          >
            <Plus size={14} strokeWidth={2.5} />
          </button>
        ) : null}
      </div>
      <div className="flex-1" data-tauri-drag-region />
      <div className="flex items-center gap-1">
        <button
          data-testid="shortcuts-button"
          className="group flex h-7 items-center gap-2 rounded-md px-2.5 text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]"
          onClick={onOpenShortcuts}
          aria-label="Keyboard shortcuts"
        >
          <HelpCircle size={14} />
          <ShortcutBadge keys={["\u2318", "/"]} />
        </button>
        <button
          data-testid="settings-button"
          className="group flex h-7 items-center gap-2 rounded-md px-2.5 text-[var(--color-text-muted)] transition-colors hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]"
          onClick={onOpenSettings}
          aria-label="Settings"
        >
          <Settings size={14} />
          <ShortcutBadge keys={["\u2318", ","]} />
        </button>
      </div>
    </div>
  );
}
