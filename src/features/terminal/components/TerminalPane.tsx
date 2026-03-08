import type { PaneSnapshotModel } from "@/features/workspace/model/workspaceSnapshot";
import type { ResolvedTheme } from "@/features/workspace/theme";
import { useTerminalSession } from "@/features/terminal/hooks/useTerminalSession";

interface TerminalPaneProps {
  pane: PaneSnapshotModel;
  fontSize: number;
  theme: ResolvedTheme;
  active: boolean;
  visible: boolean;
}

export function TerminalPane({
  pane,
  fontSize,
  theme,
  active,
  visible,
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
      className="h-full overflow-hidden"
    >
      <div ref={containerRef} className="h-full w-full px-1 py-1" />
    </div>
  );
}
