import type { ReactNode } from "react";
import type { PaneSnapshotModel, TabSnapshotModel } from "./model/workspaceSnapshot";
import type { ThemeDefinition } from "@/features/theme/domain/models";

export interface PaneRendererContext {
  readonly pane: PaneSnapshotModel;
  readonly tab: TabSnapshotModel;
  readonly theme: ThemeDefinition;
  readonly isActive: boolean;
  readonly visible: boolean;
  readonly modalOpen: boolean;
  readonly isCollapsed: boolean;
  readonly paneCount: number;
  readonly onToggleCollapse: () => void;
  readonly onClose: () => void;
  readonly onRestart: () => void;
  readonly onFocus: () => void;
  readonly dragProps: DragProps;
  readonly extras: Readonly<Record<string, unknown>>;
}

export interface DragProps {
  readonly draggable: true;
  readonly isDragOver: boolean;
  readonly onDragStart: React.DragEventHandler;
  readonly onDragOver: React.DragEventHandler;
  readonly onDragEnter: React.DragEventHandler;
  readonly onDragLeave: React.DragEventHandler;
  readonly onDrop: React.DragEventHandler;
  readonly onDragEnd: React.DragEventHandler;
}

export interface PaneRenderer {
  renderHeader: (ctx: PaneRendererContext) => ReactNode;
  renderContent: (ctx: PaneRendererContext) => ReactNode;
}

type PaneKind = "terminal" | "browser" | "git";

const registry = new Map<PaneKind, PaneRenderer>();

export function registerPaneRenderer(kind: PaneKind, renderer: PaneRenderer): void {
  registry.set(kind, renderer);
}

export function getPaneRenderer(kind: PaneKind): PaneRenderer | undefined {
  return registry.get(kind);
}
