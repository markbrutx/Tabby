import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from "react";
import {
  Panel,
  PanelGroup,
  PanelResizeHandle,
  type ImperativePanelHandle,
} from "react-resizable-panels";
import { RefreshCw } from "lucide-react";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { PaneErrorState } from "@/components/PaneErrorState";
import { Button } from "@/components/ui/Button";
import { DEFAULT_BROWSER_URL } from "@/features/workspace/domain/models";
import type {
  PaneSnapshotModel,
  TabSnapshotModel,
} from "@/features/workspace/model/workspaceSnapshot";
import type { SplitNode } from "@/features/workspace/domain/models";
import type { ThemeDefinition } from "@/features/theme/domain/models";
import { BrowserPane, type BrowserPaneHandle } from "@/features/browser/components/BrowserPane";
import { BrowserToolbar } from "@/features/browser/components/BrowserToolbar";
import { GitPane } from "@/features/git/components/GitPane";
import { GitPaneHeader } from "@/features/git/components/GitPaneHeader";
import { PaneHeader } from "@/features/terminal/components/PaneHeader";
import { TerminalPane } from "@/features/terminal/components/TerminalPane";
import type { GitClient } from "@/app-shell/clients";

// ---------------------------------------------------------------------------
// Context — holds cross-cutting state shared by all nodes in the tree
// ---------------------------------------------------------------------------

interface SplitTreeCtx {
  tab: TabSnapshotModel;
  theme: ThemeDefinition;
  visible: boolean;
  modalOpen: boolean;
  gitClient: GitClient;
  collapsedPaneIds: ReadonlySet<string>;
  onFocus: (tabId: string, paneId: string) => Promise<void>;
  onRestart: (paneId: string) => Promise<void>;
  onClosePane: (paneId: string) => void;
  onSwapPaneSlots: (paneIdA: string, paneIdB: string) => void;
  onOpenGitView: (paneId: string, cwd: string) => void;
  onToggleCollapse: (paneId: string) => void;
  dragSourceRef: React.MutableRefObject<string | null>;
  dragOverPaneId: string | null;
  onDragOverChange: (paneId: string | null) => void;
}

const TreeContext = createContext<SplitTreeCtx | null>(null);

function useTreeContext(): SplitTreeCtx {
  const ctx = useContext(TreeContext);
  if (!ctx) throw new Error("NodeRenderer must be used within SplitTreeRenderer");
  return ctx;
}

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface SplitTreeRendererProps {
  tab: TabSnapshotModel;
  theme: ThemeDefinition;
  visible: boolean;
  modalOpen?: boolean;
  gitClient: GitClient;
  collapsedPaneIds: ReadonlySet<string>;
  onFocus: (tabId: string, paneId: string) => Promise<void>;
  onRestart: (paneId: string) => Promise<void>;
  onClosePane: (paneId: string) => void;
  onSwapPaneSlots: (paneIdA: string, paneIdB: string) => void;
  onOpenGitView: (paneId: string, cwd: string) => void;
  onToggleCollapse: (paneId: string) => void;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function findPaneById(tab: TabSnapshotModel, paneId: string): PaneSnapshotModel | undefined {
  return tab.panes.find((p) => p.id === paneId);
}

// ---------------------------------------------------------------------------
// PaneLeaf — renders a single pane (terminal or browser)
// ---------------------------------------------------------------------------

function PaneLeaf({ paneId }: { paneId: string }) {
  const ctx = useTreeContext();
  const {
    tab, theme, visible, modalOpen,
    onFocus, onRestart, onClosePane, onSwapPaneSlots, onToggleCollapse,
    collapsedPaneIds,
    dragSourceRef, dragOverPaneId, onDragOverChange,
  } = ctx;

  const isCollapsed = collapsedPaneIds.has(paneId);

  const pane = findPaneById(tab, paneId);
  const browserPaneRef = useRef<BrowserPaneHandle | null>(null);
  const isBrowser = pane?.paneKind === "browser";
  const isGit = pane?.paneKind === "git";

  if (!pane) return null;

  const isActive = tab.activePaneId === pane.id;
  const isDragOver = dragOverPaneId === pane.id;
  const isDragSource = dragSourceRef.current === pane.id;

  const dragProps = {
    draggable: true as const,
    isDragOver,
    onDragStart: (e: React.DragEvent) => {
      dragSourceRef.current = pane.id;
      e.dataTransfer.effectAllowed = "move";
      e.dataTransfer.setData("text/plain", pane.id);
    },
    onDragOver: (e: React.DragEvent) => {
      e.preventDefault();
      e.dataTransfer.dropEffect = "move";
    },
    onDragEnter: (e: React.DragEvent) => {
      e.preventDefault();
      if (dragSourceRef.current && dragSourceRef.current !== pane.id) {
        onDragOverChange(pane.id);
      }
    },
    onDragLeave: () => {
      if (dragOverPaneId === pane.id) {
        onDragOverChange(null);
      }
    },
    onDrop: (e: React.DragEvent) => {
      e.preventDefault();
      const sourceId = dragSourceRef.current;
      if (sourceId && sourceId !== pane.id) {
        onSwapPaneSlots(sourceId, pane.id);
      }
      dragSourceRef.current = null;
      onDragOverChange(null);
    },
    onDragEnd: () => {
      dragSourceRef.current = null;
      onDragOverChange(null);
    },
  };

  return (
    <ErrorBoundary
      fallback={(error, reset) => (
        <PaneErrorState
          title="Pane Crashed"
          message={error.message || "An unexpected error occurred in this pane."}
          action={
            <Button
              variant="secondary"
              onClick={() => {
                reset();
                void onRestart(pane.id);
              }}
            >
              <RefreshCw size={14} className="mr-2" />
              Restart
            </Button>
          }
        />
      )}
    >
      <div className={`flex h-full flex-col ${isDragSource ? "opacity-50" : ""}`}>
        {isBrowser ? (
          <BrowserToolbar
            url={pane.url ?? DEFAULT_BROWSER_URL}
            isActive={isActive}
            paneCount={tab.panes.length}
            isCollapsed={isCollapsed}
            onToggleCollapse={() => onToggleCollapse(pane.id)}
            onNavigate={(url) => {
              browserPaneRef.current?.navigate(url);
            }}
            onReload={() => {
              browserPaneRef.current?.navigate(pane.url ?? DEFAULT_BROWSER_URL);
            }}
            onClose={() => onClosePane(pane.id)}
            {...dragProps}
          />
        ) : isGit ? (
          <GitPaneHeader
            repoPath={pane.gitRepoPath ?? pane.cwd}
            branch={null}
            isActive={isActive}
            paneCount={tab.panes.length}
            isCollapsed={isCollapsed}
            onToggleCollapse={() => onToggleCollapse(pane.id)}
            onClose={() => onClosePane(pane.id)}
            {...dragProps}
          />
        ) : (
          <PaneHeader
            profileLabel={pane.profileLabel}
            cwd={pane.cwd}
            isActive={isActive}
            paneCount={tab.panes.length}
            isCollapsed={isCollapsed}
            onToggleCollapse={() => onToggleCollapse(pane.id)}
            onClose={() => onClosePane(pane.id)}
            onRestart={() => void onRestart(pane.id)}
            onOpenGitView={pane.cwd ? () => ctx.onOpenGitView(pane.id, pane.cwd) : undefined}
            {...dragProps}
          />
        )}
        {isCollapsed ? null : (
          <div
            className="min-h-0 flex-1"
            onMouseDown={() => void onFocus(tab.id, pane.id)}
          >
            {isBrowser ? (
              <BrowserPane
                ref={browserPaneRef}
                pane={pane}
                active={isActive}
                visible={visible}
                modalOpen={modalOpen}
              />
            ) : isGit ? (
              <GitPane pane={pane} gitClient={ctx.gitClient} />
            ) : (
              <TerminalPane
                pane={pane}
                theme={theme}
                active={isActive}
                visible={visible}
              />
            )}
          </div>
        )}
      </div>
    </ErrorBoundary>
  );
}

// ---------------------------------------------------------------------------
// NodeRenderer — recursive tree walker (now only takes `node`)
// ---------------------------------------------------------------------------

function NodeRenderer({ node }: { node: SplitNode }) {
  const ctx = useTreeContext();
  const firstPanelRef = useRef<ImperativePanelHandle>(null);
  const secondPanelRef = useRef<ImperativePanelHandle>(null);

  const isSplit = node.type === "split";
  const firstChild = isSplit ? node.first : null;
  const secondChild = isSplit ? node.second : null;
  const firstIsLeaf = firstChild?.type === "pane";
  const secondIsLeaf = secondChild?.type === "pane";
  const firstCollapsed = firstIsLeaf && ctx.collapsedPaneIds.has(firstChild.paneId);
  const secondCollapsed = secondIsLeaf && ctx.collapsedPaneIds.has(secondChild.paneId);

  // Guard: if both siblings are collapsed, auto-expand the second one
  useEffect(() => {
    if (firstCollapsed && secondCollapsed && secondIsLeaf && secondChild) {
      ctx.onToggleCollapse(secondChild.paneId);
    }
  }, [firstCollapsed, secondCollapsed, secondIsLeaf, secondChild, ctx]);

  useEffect(() => {
    if (!firstIsLeaf) return;
    if (firstCollapsed) {
      firstPanelRef.current?.collapse();
    } else {
      firstPanelRef.current?.expand();
    }
  }, [firstCollapsed, firstIsLeaf]);

  useEffect(() => {
    if (!secondIsLeaf) return;
    if (secondCollapsed) {
      secondPanelRef.current?.collapse();
    } else {
      secondPanelRef.current?.expand();
    }
  }, [secondCollapsed, secondIsLeaf]);

  // Handlers to sync drag-collapse/expand with the store
  // NOTE: these must be called unconditionally (before the early return)
  // to satisfy React's rules of hooks.
  const handleFirstCollapse = useCallback(() => {
    if (firstIsLeaf && firstChild) {
      ctx.onToggleCollapse(firstChild.paneId);
    }
  }, [firstIsLeaf, firstChild, ctx]);

  const handleFirstExpand = useCallback(() => {
    if (firstIsLeaf && firstChild && ctx.collapsedPaneIds.has(firstChild.paneId)) {
      ctx.onToggleCollapse(firstChild.paneId);
    }
  }, [firstIsLeaf, firstChild, ctx]);

  const handleSecondCollapse = useCallback(() => {
    if (secondIsLeaf && secondChild) {
      ctx.onToggleCollapse(secondChild.paneId);
    }
  }, [secondIsLeaf, secondChild, ctx]);

  const handleSecondExpand = useCallback(() => {
    if (secondIsLeaf && secondChild && ctx.collapsedPaneIds.has(secondChild.paneId)) {
      ctx.onToggleCollapse(secondChild.paneId);
    }
  }, [secondIsLeaf, secondChild, ctx]);

  if (node.type === "pane") {
    return <PaneLeaf paneId={node.paneId} />;
  }

  const direction = node.direction === "horizontal" ? "horizontal" : "vertical";
  const firstSize = (node.ratio / 1000) * 100;
  const secondSize = 100 - firstSize;

  return (
    <PanelGroup direction={direction} className="h-full">
      <Panel
        ref={firstIsLeaf ? firstPanelRef : undefined}
        defaultSize={firstSize}
        minSize={firstIsLeaf ? 0 : 5}
        collapsible={firstIsLeaf}
        collapsedSize={0}
        onCollapse={firstIsLeaf ? handleFirstCollapse : undefined}
        onExpand={firstIsLeaf ? handleFirstExpand : undefined}
      >
        <NodeRenderer node={node.first} />
      </Panel>
      <PanelResizeHandle
        className={`resize-handle ${direction === "horizontal" ? "w-[3px]" : "h-[3px]"
          } shrink-0 bg-[var(--color-border)] transition-colors hover:bg-[var(--color-accent)]`}
      />
      <Panel
        ref={secondIsLeaf ? secondPanelRef : undefined}
        defaultSize={secondSize}
        minSize={secondIsLeaf ? 0 : 5}
        collapsible={secondIsLeaf}
        collapsedSize={0}
        onCollapse={secondIsLeaf ? handleSecondCollapse : undefined}
        onExpand={secondIsLeaf ? handleSecondExpand : undefined}
      >
        <NodeRenderer node={node.second} />
      </Panel>
    </PanelGroup>
  );
}

// ---------------------------------------------------------------------------
// SplitTreeRenderer — public component, provides context
// ---------------------------------------------------------------------------

export function SplitTreeRenderer({
  tab,
  theme,
  visible,
  modalOpen = false,
  gitClient,
  collapsedPaneIds,
  onFocus,
  onRestart,
  onClosePane,
  onSwapPaneSlots,
  onOpenGitView,
  onToggleCollapse,
}: SplitTreeRendererProps) {
  const dragSourceRef = useRef<string | null>(null);
  const [dragOverPaneId, setDragOverPaneId] = useState<string | null>(null);

  const handleDragOverChange = useCallback((paneId: string | null) => {
    setDragOverPaneId(paneId);
  }, []);

  const ctx: SplitTreeCtx = useMemo(() => ({
    tab,
    theme,
    visible,
    modalOpen,
    gitClient,
    collapsedPaneIds,
    onFocus,
    onRestart,
    onClosePane,
    onSwapPaneSlots,
    onOpenGitView,
    onToggleCollapse,
    dragSourceRef,
    dragOverPaneId,
    onDragOverChange: handleDragOverChange,
  }), [
    tab, theme, visible, modalOpen, gitClient, collapsedPaneIds,
    onFocus, onRestart, onClosePane, onSwapPaneSlots, onOpenGitView, onToggleCollapse,
    dragSourceRef, dragOverPaneId, handleDragOverChange,
  ]);

  return (
    <TreeContext.Provider value={ctx}>
      <NodeRenderer node={tab.layout} />
    </TreeContext.Provider>
  );
}
