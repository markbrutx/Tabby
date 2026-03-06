import type { TabSnapshot } from "@/features/workspace/domain";
import { createGridDefinition } from "@/features/workspace/layouts";

type PaneDirection = "up" | "down" | "left" | "right";

export function selectNextPaneId(
  tab: TabSnapshot,
  activePaneId: string,
): string | null {
  const index = tab.panes.findIndex((pane) => pane.id === activePaneId);
  if (index === -1) {
    return tab.panes[0]?.id ?? null;
  }

  return tab.panes[(index + 1) % tab.panes.length]?.id ?? null;
}

export function selectPreviousPaneId(
  tab: TabSnapshot,
  activePaneId: string,
): string | null {
  const index = tab.panes.findIndex((pane) => pane.id === activePaneId);
  if (index === -1) {
    return tab.panes[0]?.id ?? null;
  }

  const nextIndex = (index - 1 + tab.panes.length) % tab.panes.length;
  return tab.panes[nextIndex]?.id ?? null;
}

export function selectAdjacentPaneId(
  tab: TabSnapshot,
  activePaneId: string,
  direction: PaneDirection,
): string | null {
  const definition = createGridDefinition(tab.preset);
  const index = tab.panes.findIndex((pane) => pane.id === activePaneId);
  if (index === -1) {
    return null;
  }

  const row = Math.floor(index / definition.columns);
  const column = index % definition.columns;

  const [nextRow, nextColumn] = (() => {
    switch (direction) {
      case "up":
        return [row - 1, column];
      case "down":
        return [row + 1, column];
      case "left":
        return [row, column - 1];
      case "right":
        return [row, column + 1];
    }
  })();

  if (
    nextRow < 0 ||
    nextColumn < 0 ||
    nextRow >= definition.rows ||
    nextColumn >= definition.columns
  ) {
    return null;
  }

  const nextIndex = nextRow * definition.columns + nextColumn;
  return tab.panes[nextIndex]?.id ?? null;
}
