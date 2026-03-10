import type { PaneSnapshotModel } from "@/features/workspace/model/workspaceSnapshot";
import type { ThemeDefinition } from "@/features/theme/domain/models";
import { useTerminalSession } from "@/features/terminal/hooks/useTerminalSession";

interface TerminalPaneProps {
  pane: PaneSnapshotModel;
  theme: ThemeDefinition;
  active: boolean;
  visible: boolean;
}

export function TerminalPane({
  pane,
  theme,
  active,
  visible,
}: TerminalPaneProps) {
  const { containerRef } = useTerminalSession({
    pane,
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
