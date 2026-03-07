import type { PaneSnapshot } from "@/features/workspace/domain";
import type { ResolvedTheme } from "@/features/workspace/theme";
import { useTerminalSession } from "@/features/workspace/hooks/useTerminalSession";

interface TerminalPaneProps {
  pane: PaneSnapshot;
  fontSize: number;
  theme: ResolvedTheme;
  active: boolean;
  visible: boolean;
  onFocus: (paneId: string) => Promise<void>;
}

export function TerminalPane({
  pane,
  fontSize,
  theme,
  active,
  visible,
  onFocus,
}: TerminalPaneProps) {
  const { containerRef } = useTerminalSession({
    pane,
    fontSize,
    theme,
    active,
    visible,
  });

  return (
    <div
      data-testid={`pane-${pane.id}`}
      data-active={active ? "true" : "false"}
      className={`h-full overflow-hidden ${
        active
          ? "ring-1 ring-[var(--color-accent)] ring-opacity-60"
          : ""
      }`}
      onMouseDown={() => void onFocus(pane.id)}
    >
      <div ref={containerRef} className="h-full w-full px-1 py-1" />
    </div>
  );
}
