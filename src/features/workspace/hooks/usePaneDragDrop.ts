import { useCallback, useRef, useState } from "react";
import type { DragProps } from "@/features/workspace/paneRegistry";

export interface PaneDragDropState {
  readonly dragSourceRef: React.MutableRefObject<string | null>;
  readonly dragOverPaneId: string | null;
  readonly onDragOverChange: (paneId: string | null) => void;
  readonly buildDragProps: (
    paneId: string,
    onSwapPaneSlots: (a: string, b: string) => void,
  ) => DragProps;
}

export function usePaneDragDrop(): PaneDragDropState {
  const dragSourceRef = useRef<string | null>(null);
  const [dragOverPaneId, setDragOverPaneId] = useState<string | null>(null);

  const onDragOverChange = useCallback((paneId: string | null) => {
    setDragOverPaneId(paneId);
  }, []);

  const buildDragProps = useCallback(
    (paneId: string, onSwapPaneSlots: (a: string, b: string) => void): DragProps => ({
      draggable: true as const,
      isDragOver: dragOverPaneId === paneId,
      onDragStart: (e: React.DragEvent) => {
        dragSourceRef.current = paneId;
        e.dataTransfer.effectAllowed = "move";
        e.dataTransfer.setData("text/plain", paneId);
      },
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
      onDragLeave: () => {
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
      onDragEnd: () => {
        dragSourceRef.current = null;
        setDragOverPaneId(null);
      },
    }),
    [dragOverPaneId],
  );

  return { dragSourceRef, dragOverPaneId, onDragOverChange, buildDragProps };
}
