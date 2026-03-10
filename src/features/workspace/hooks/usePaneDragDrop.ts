import { useCallback, useRef, useState } from "react";
import type { DragSourceProps, DropTargetProps } from "@/features/workspace/paneRegistry";

export interface PaneDragDropState {
  readonly dragSourceRef: React.MutableRefObject<string | null>;
  readonly dragOverPaneId: string | null;
  readonly buildDragSourceProps: (paneId: string) => DragSourceProps;
  readonly buildDropTargetProps: (
    paneId: string,
    onSwapPaneSlots: (a: string, b: string) => void,
  ) => DropTargetProps;
}

export function usePaneDragDrop(): PaneDragDropState {
  const dragSourceRef = useRef<string | null>(null);
  const [dragOverPaneId, setDragOverPaneId] = useState<string | null>(null);

  const buildDragSourceProps = useCallback(
    (paneId: string): DragSourceProps => ({
      draggable: true as const,
      onDragStart: (e: React.DragEvent) => {
        dragSourceRef.current = paneId;
        e.dataTransfer.effectAllowed = "move";
        e.dataTransfer.setData("text/plain", paneId);
      },
      onDragEnd: () => {
        dragSourceRef.current = null;
        setDragOverPaneId(null);
      },
    }),
    [],
  );

  const buildDropTargetProps = useCallback(
    (paneId: string, onSwapPaneSlots: (a: string, b: string) => void): DropTargetProps => ({
      isDragOver: dragOverPaneId === paneId,
      onDragOver: (e: React.DragEvent) => {
        e.preventDefault();
        e.dataTransfer.dropEffect = "move";
      },
      onDragEnter: (e: React.DragEvent) => {
        e.preventDefault();
        if (dragSourceRef.current && dragSourceRef.current !== paneId) {
          setDragOverPaneId(paneId);
        }
      },
      onDragLeave: (e: React.DragEvent) => {
        const related = e.relatedTarget as Node | null;
        const current = e.currentTarget as Node;
        if (related && current.contains(related)) {
          return;
        }
        if (dragOverPaneId === paneId) {
          setDragOverPaneId(null);
        }
      },
      onDrop: (e: React.DragEvent) => {
        e.preventDefault();
        const sourceId = dragSourceRef.current;
        if (sourceId && sourceId !== paneId) {
          onSwapPaneSlots(sourceId, paneId);
        }
        dragSourceRef.current = null;
        setDragOverPaneId(null);
      },
    }),
    [dragOverPaneId],
  );

  return { dragSourceRef, dragOverPaneId, buildDragSourceProps, buildDropTargetProps };
}
