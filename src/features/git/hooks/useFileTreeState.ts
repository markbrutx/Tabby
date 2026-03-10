import { useState, useCallback } from "react";

interface ContextMenuState {
  readonly x: number;
  readonly y: number;
  readonly filePath: string;
}

export interface FileTreeState {
  readonly stagedExpanded: boolean;
  readonly unstagedExpanded: boolean;
  readonly discardTarget: string | null;
  readonly contextMenu: ContextMenuState | null;
  readonly toggleStaged: () => void;
  readonly toggleUnstaged: () => void;
  readonly openContextMenu: (e: React.MouseEvent, filePath: string) => void;
  readonly closeContextMenu: () => void;
  readonly requestDiscard: (filePath: string) => void;
  readonly confirmDiscard: (onDiscardChanges: (paths: readonly string[]) => void) => void;
  readonly cancelDiscard: () => void;
}

export function useFileTreeState(hasBlameSupport: boolean): FileTreeState {
  const [stagedExpanded, setStagedExpanded] = useState(true);
  const [unstagedExpanded, setUnstagedExpanded] = useState(true);
  const [discardTarget, setDiscardTarget] = useState<string | null>(null);
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null);

  const toggleStaged = useCallback(() => {
    setStagedExpanded((prev) => !prev);
  }, []);

  const toggleUnstaged = useCallback(() => {
    setUnstagedExpanded((prev) => !prev);
  }, []);

  const openContextMenu = useCallback(
    (e: React.MouseEvent, filePath: string) => {
      if (hasBlameSupport) {
        setContextMenu({ x: e.clientX, y: e.clientY, filePath });
      }
    },
    [hasBlameSupport],
  );

  const closeContextMenu = useCallback(() => {
    setContextMenu(null);
  }, []);

  const requestDiscard = useCallback((filePath: string) => {
    setDiscardTarget(filePath);
  }, []);

  const confirmDiscard = useCallback(
    (onDiscardChanges: (paths: readonly string[]) => void) => {
      if (discardTarget !== null) {
        onDiscardChanges([discardTarget]);
        setDiscardTarget(null);
      }
    },
    [discardTarget],
  );

  const cancelDiscard = useCallback(() => {
    setDiscardTarget(null);
  }, []);

  return {
    stagedExpanded,
    unstagedExpanded,
    discardTarget,
    contextMenu,
    toggleStaged,
    toggleUnstaged,
    openContextMenu,
    closeContextMenu,
    requestDiscard,
    confirmDiscard,
    cancelDiscard,
  };
}
