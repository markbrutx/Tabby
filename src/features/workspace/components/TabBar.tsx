import { useCallback, useEffect, useRef, useState } from "react";
import { HelpCircle, Plus, Settings, X } from "lucide-react";
import { ShortcutBadge } from "@/features/workspace/components/ShortcutBadge";
import { isTauriRuntime } from "@/lib/runtime";

interface TabEntry {
  id: string;
  title: string;
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
      className="flex h-10 shrink-0 select-none items-center bg-[var(--color-surface)] pl-[72px] text-xs"
      data-tauri-drag-region
      onMouseDown={handleDragStart}
    >
      <div className="flex items-center gap-0 overflow-x-auto">
      {tabs.map((tab, index) => {
        const isActive = tab.id === activeTabId;
        const isEditing = editingTabId === tab.id;

        return (
          <button
            key={tab.id}
            data-testid={`tab-${index + 1}`}
            className={`group relative flex h-full items-center gap-2 px-4 transition-colors ${
              isActive
                ? "bg-[var(--color-surface-overlay)] text-[var(--color-text)]"
                : "text-[var(--color-text-muted)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text-soft)]"
            }`}
            onClick={() => {
              if (!isEditing) {
                onSelect(tab.id);
              }
            }}
            onDoubleClick={() => startEditing(tab)}
          >
            {isEditing ? (
              <input
                ref={inputRef}
                data-testid={`tab-rename-input-${index + 1}`}
                className="w-24 rounded bg-[var(--color-surface)] px-1 text-xs text-[var(--color-text)] outline-none ring-1 ring-[var(--color-accent)]"
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
      {showNewTab ? (
        <button
          data-testid="new-tab-button"
          className="flex h-full items-center px-3 text-[var(--color-text-muted)] transition-colors hover:text-[var(--color-text)]"
          onClick={onNewTab}
          aria-label="New tab"
        >
          <Plus size={14} />
        </button>
      ) : null}
      </div>
      <div className="flex-1" data-tauri-drag-region />
      <button
        data-testid="shortcuts-button"
        className="flex h-full items-center gap-2 px-3 text-[var(--color-text-muted)] transition-colors hover:text-[var(--color-text)]"
        onClick={onOpenShortcuts}
        aria-label="Keyboard shortcuts"
      >
        <HelpCircle size={14} />
        <ShortcutBadge keys={["\u2318", "/"]} />
      </button>
      <button
        data-testid="settings-button"
        className="flex h-full items-center gap-2 px-3 text-[var(--color-text-muted)] transition-colors hover:text-[var(--color-text)]"
        onClick={onOpenSettings}
        aria-label="Settings"
      >
        <Settings size={14} />
        <ShortcutBadge keys={["\u2318", ","]} />
      </button>
    </div>
  );
}
