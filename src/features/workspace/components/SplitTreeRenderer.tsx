import { createContext, useCallback, useContext, useMemo, useRef, useState } from "react";
import {
  Panel,
  PanelGroup,
  PanelResizeHandle,
} from "react-resizable-panels";
import { RefreshCw } from "lucide-react";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { Button } from "@/components/ui/Button";
import { DEFAULT_BROWSER_URL } from "@/features/workspace/domain/models";
import type {
  PaneSnapshotModel,
  TabSnapshotModel,
} from "@/features/workspace/model/workspaceSnapshot";
import type { SplitNode } from "@/features/workspace/domain/models";
import type { ResolvedTheme } from "@/features/workspace/theme";
import { BrowserPane, type BrowserPaneHandle } from "@/features/browser/components/BrowserPane";
import { BrowserToolbar } from "@/features/browser/components/BrowserToolbar";
import { PaneHeader } from "@/features/terminal/components/PaneHeader";
import { TerminalPane } from "@/features/terminal/components/TerminalPane";

// ---------------------------------------------------------------------------
// Context — holds cross-cutting state shared by all nodes in the tree
// ---------------------------------------------------------------------------

interface SplitTreeCtx {
  tab: TabSnapshotModel;
  fontSize: number;
  theme: ResolvedTheme;
  visible: boolean;
  modalOpen: boolean;
  onFocus: (tabId: string, paneId: string) => Promise<void>;
  onRestart: (paneId: string) => Promise<void>;
  onClosePane: (paneId: string) => void;
  onSwapPaneSlots: (paneIdA: string, paneIdB: string) => void;
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
  fontSize: number;
  theme: ResolvedTheme;
  visible: boolean;
  modalOpen?: boolean;
  onFocus: (tabId: string, paneId: string) => Promise<void>;
  onRestart: (paneId: string) => Promise<void>;
  onClosePane: (paneId: string) => void;
  onSwapPaneSlots: (paneIdA: string, paneIdB: string) => void;
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
    tab, fontSize, theme, visible, modalOpen,
    onFocus, onRestart, onClosePane, onSwapPaneSlots,
    dragSourceRef, dragOverPaneId, onDragOverChange,
  } = ctx;

  const pane = findPaneById(tab, paneId);
  const browserPaneRef = useRef<BrowserPaneHandle | null>(null);
  const isBrowser = pane?.paneKind === "browser";

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
      fallback={(_error, reset) => (
        <div className="flex h-full items-center justify-center gap-3 text-center">
          <p className="text-sm text-[var(--color-text-muted)]">
            Pane crashed
          </p>
          <Button
            variant="secondary"
            size="sm"
            onClick={() => {
              reset();
              void onRestart(pane.id);
            }}
          >
            <RefreshCw size={14} />
            Restart
          </Button>
        </div>
      )}
    >
      <div className={`flex h-full flex-col ${isDragSource ? "opacity-50" : ""}`}>
        {isBrowser ? (
          <BrowserToolbar
            url={pane.url ?? DEFAULT_BROWSER_URL}
            isActive={isActive}
            paneCount={tab.panes.length}
            onNavigate={(url) => {
              browserPaneRef.current?.navigate(url);
            }}
            onReload={() => {
              browserPaneRef.current?.navigate(pane.url ?? DEFAULT_BROWSER_URL);
            }}
            onClose={() => onClosePane(pane.id)}
            {...dragProps}
          />
        ) : (
          <PaneHeader
            profileLabel={pane.profileLabel}
            cwd={pane.cwd}
            isActive={isActive}
            paneCount={tab.panes.length}
            onClose={() => onClosePane(pane.id)}
            {...dragProps}
          />
        )}
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
          ) : (
            <TerminalPane
              pane={pane}
              fontSize={fontSize}
              theme={theme}
              active={isActive}
              visible={visible}
            />
          )}
        </div>
      </div>
    </ErrorBoundary>
  );
}

// ---------------------------------------------------------------------------
// NodeRenderer — recursive tree walker (now only takes `node`)
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
        className={`resize-handle ${
          direction === "horizontal" ? "w-[3px]" : "h-[3px]"
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
  fontSize,
  theme,
  visible,
  modalOpen = false,
  onFocus,
  onRestart,
  onClosePane,
  onSwapPaneSlots,
}: SplitTreeRendererProps) {
  const dragSourceRef = useRef<string | null>(null);
  const [dragOverPaneId, setDragOverPaneId] = useState<string | null>(null);

  const handleDragOverChange = useCallback((paneId: string | null) => {
    setDragOverPaneId(paneId);
  }, []);

  const ctx: SplitTreeCtx = useMemo(() => ({
    tab,
    fontSize,
    theme,
    visible,
    modalOpen,
    onFocus,
    onRestart,
    onClosePane,
    onSwapPaneSlots,
    dragSourceRef,
    dragOverPaneId,
    onDragOverChange: handleDragOverChange,
  }), [
    tab, fontSize, theme, visible, modalOpen,
    onFocus, onRestart, onClosePane, onSwapPaneSlots,
    dragSourceRef, dragOverPaneId, handleDragOverChange,
  ]);

  return (
    <TreeContext.Provider value={ctx}>
      <NodeRenderer node={tab.layout} />
    </TreeContext.Provider>
  );
}
