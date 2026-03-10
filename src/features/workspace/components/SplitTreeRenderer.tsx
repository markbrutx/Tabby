import { createContext, useContext, useMemo, useRef } from "react";
import {
  Panel,
  PanelGroup,
  PanelResizeHandle,
} from "react-resizable-panels";
import { RefreshCw } from "lucide-react";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { PaneErrorState } from "@/components/PaneErrorState";
import { Button } from "@/components/ui/Button";
import type {
  PaneSnapshotModel,
  TabSnapshotModel,
} from "@/features/workspace/model/workspaceSnapshot";
import type { DragSourceProps, DropTargetProps } from "@/features/workspace/paneRegistry";
import type { SplitNode } from "@/features/workspace/domain/models";
import type { ThemeDefinition } from "@/features/theme/domain/models";
import type { GitClient } from "@/app-shell/clients";
import { getPaneRenderer, type PaneRendererContext } from "@/features/workspace/paneRegistry";
import { usePaneDragDrop } from "@/features/workspace/hooks/usePaneDragDrop";

// ---------------------------------------------------------------------------
// Context — holds cross-cutting state shared by all nodes in the tree
// ---------------------------------------------------------------------------

interface SplitTreeCtx {
  tab: TabSnapshotModel;
  theme: ThemeDefinition;
  visible: boolean;
  modalOpen: boolean;
  gitClient: GitClient;
  onFocus: (tabId: string, paneId: string) => Promise<void>;
  onRestart: (paneId: string) => Promise<void>;
  onClosePane: (paneId: string) => void;
  onSwapPaneSlots: (paneIdA: string, paneIdB: string) => void;
  onOpenGitView: (paneId: string, cwd: string) => void;
  dragSourceRef: React.MutableRefObject<string | null>;
  buildDragSourceProps: (paneId: string) => DragSourceProps;
  buildDropTargetProps: (paneId: string, onSwapPaneSlots: (a: string, b: string) => void) => DropTargetProps;
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
  onFocus: (tabId: string, paneId: string) => Promise<void>;
  onRestart: (paneId: string) => Promise<void>;
  onClosePane: (paneId: string) => void;
  onSwapPaneSlots: (paneIdA: string, paneIdB: string) => void;
  onOpenGitView: (paneId: string, cwd: string) => void;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function findPaneById(tab: TabSnapshotModel, paneId: string): PaneSnapshotModel | undefined {
  return tab.panes.find((p) => p.id === paneId);
}

// ---------------------------------------------------------------------------
// PaneLeaf — renders a single pane via the renderer registry
// ---------------------------------------------------------------------------

function PaneLeaf({ paneId }: { paneId: string }) {
  const ctx = useTreeContext();
  const {
    tab, theme, visible, modalOpen,
    onFocus, onRestart, onClosePane, onSwapPaneSlots,
    dragSourceRef, buildDragSourceProps, buildDropTargetProps,
  } = ctx;

  const pane = findPaneById(tab, paneId);
  const browserPaneRef = useRef(null);

  if (!pane) return null;

  const isActive = tab.activePaneId === pane.id;
  const isDragSource = dragSourceRef.current === pane.id;

  const dragProps = buildDragSourceProps(pane.id);
  const dropProps = buildDropTargetProps(pane.id, onSwapPaneSlots);

  const renderer = getPaneRenderer(pane.paneKind);
  if (!renderer) return null;

  const rendererCtx: PaneRendererContext = {
    pane,
    tab,
    theme,
    isActive,
    visible,
    modalOpen,
    paneCount: tab.panes.length,
    onClose: () => onClosePane(pane.id),
    onRestart: () => void onRestart(pane.id),
    onFocus: () => void onFocus(tab.id, pane.id),
    dragProps,
    extras: {
      gitClient: ctx.gitClient,
      onOpenGitView: ctx.onOpenGitView,
      browserPaneRef,
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
      <div
        className={`relative flex h-full flex-col ${isDragSource ? "opacity-50" : ""}`}
        onDragOver={dropProps.onDragOver}
        onDragEnter={dropProps.onDragEnter}
        onDragLeave={dropProps.onDragLeave}
        onDrop={dropProps.onDrop}
      >
        {dropProps.isDragOver && (
          <div className="pointer-events-none absolute inset-0 z-50 rounded border-2 border-[var(--color-accent)] bg-[var(--color-accent)]/10" />
        )}
        {renderer.renderHeader(rendererCtx)}
        <div
          className="min-h-0 flex-1"
          onMouseDown={rendererCtx.onFocus}
        >
          {renderer.renderContent(rendererCtx)}
        </div>
      </div>
    </ErrorBoundary>
  );
}

// ---------------------------------------------------------------------------
// NodeRenderer — recursive tree walker
// ---------------------------------------------------------------------------

function NodeRenderer({ node }: { node: SplitNode }) {
  if (node.type === "pane") {
    return <PaneLeaf paneId={node.paneId} />;
  }

  const direction = node.direction === "horizontal" ? "horizontal" : "vertical";
  const firstSize = (node.ratio / 1000) * 100;
  const secondSize = 100 - firstSize;

  return (
    <PanelGroup direction={direction} className="h-full">
      <Panel defaultSize={firstSize} minSize={5}>
        <NodeRenderer node={node.first} />
      </Panel>
      <PanelResizeHandle
        className={`resize-handle ${direction === "horizontal" ? "w-[3px]" : "h-[3px]"
          } shrink-0 bg-[var(--color-border)] transition-colors hover:bg-[var(--color-accent)]`}
      />
      <Panel defaultSize={secondSize} minSize={5}>
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
  onFocus,
  onRestart,
  onClosePane,
  onSwapPaneSlots,
  onOpenGitView,
}: SplitTreeRendererProps) {
  const { dragSourceRef, buildDragSourceProps, buildDropTargetProps } = usePaneDragDrop();

  const ctx: SplitTreeCtx = useMemo(() => ({
    tab,
    theme,
    visible,
    modalOpen,
    gitClient,
    onFocus,
    onRestart,
    onClosePane,
    onSwapPaneSlots,
    onOpenGitView,
    dragSourceRef,
    buildDragSourceProps,
    buildDropTargetProps,
  }), [
    tab, theme, visible, modalOpen, gitClient,
    onFocus, onRestart, onClosePane, onSwapPaneSlots, onOpenGitView,
    dragSourceRef, buildDragSourceProps, buildDropTargetProps,
  ]);

  return (
    <TreeContext.Provider value={ctx}>
      <NodeRenderer node={tab.layout} />
    </TreeContext.Provider>
  );
}
