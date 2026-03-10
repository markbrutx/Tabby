import { X } from "lucide-react";
import { useEscapeKey } from "@/hooks/useEscapeKey";
import { ShortcutBadge } from "@/components/ShortcutBadge";

interface ShortcutEntry {
  keys: string[];
  description: string;
}

interface ShortcutGroup {
  title: string;
  shortcuts: ShortcutEntry[];
}

const SHORTCUT_GROUPS: ShortcutGroup[] = [
  {
    title: "Tab Management",
    shortcuts: [
      { keys: ["\u2318", "T"], description: "New workspace" },
      { keys: ["\u2318", "1\u20139"], description: "Switch to tab by number" },
      { keys: ["\u2318", "\u21e7", "W"], description: "Close entire tab" },
      { keys: ["\u2318", "W"], description: "Close active pane" },
    ],
  },
  {
    title: "Pane Management",
    shortcuts: [
      { keys: ["\u2318", "D"], description: "Split right" },
      { keys: ["\u2318", "E"], description: "Split down" },
      { keys: ["\u2318", "\u2325", "\u2190\u2191\u2192\u2193"], description: "Navigate between panes" },
      { keys: ["\u2318", "]"], description: "Next pane" },
      { keys: ["\u2318", "["], description: "Previous pane" },
      { keys: ["\u2318", "\u21e7", "R"], description: "Restart active pane" },
    ],
  },
  {
    title: "View",
    shortcuts: [
      { keys: ["\u2318", "+"], description: "Zoom in" },
      { keys: ["\u2318", "\u2212"], description: "Zoom out" },
      { keys: ["\u2318", "0"], description: "Reset zoom" },
    ],
  },
  {
    title: "Application",
    shortcuts: [
      { keys: ["\u2318", ","], description: "Open settings" },
      { keys: ["\u2318", "/"], description: "Show keyboard shortcuts" },
    ],
  },
];

interface ShortcutsModalProps {
  onClose: () => void;
}

export function ShortcutsModal({ onClose }: ShortcutsModalProps) {
  useEscapeKey(onClose);

  return (
    <div
      data-testid="shortcuts-modal"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
      onClick={(event) => {
        if (event.target === event.currentTarget) onClose();
      }}
      role="dialog"
    >
      <div className="w-full max-w-md rounded-lg border border-[var(--color-border)] bg-[var(--color-surface)] p-6 shadow-2xl">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold">Keyboard Shortcuts</h2>
          <button
            className="rounded p-1 text-[var(--color-text-muted)] hover:bg-[var(--color-surface-hover)]"
            onClick={onClose}
          >
            <X size={16} />
          </button>
        </div>

        <div className="mt-5 space-y-5">
          {SHORTCUT_GROUPS.map((group) => (
            <div key={group.title}>
              <h3 className="mb-2 text-xs font-semibold uppercase tracking-wider text-[var(--color-text-muted)]">
                {group.title}
              </h3>
              <div className="space-y-1.5">
                {group.shortcuts.map((shortcut, i) => (
                  <div
                    key={i}
                    className="flex items-center justify-between rounded px-2 py-1 text-sm"
                  >
                    <span className="text-[var(--color-text-soft)]">
                      {shortcut.description}
                    </span>
                    <ShortcutBadge keys={shortcut.keys} />
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
