import { createContext, useCallback, useContext, useRef, useState } from "react";
import {
  Panel,
  PanelGroup,
  PanelResizeHandle,
} from "react-resizable-panels";
import { RefreshCw } from "lucide-react";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { Button } from "@/components/ui/Button";
import type { PaneSnapshot, SplitNode, TabSnapshot } from "@/features/workspace/domain";
import { DEFAULT_BROWSER_URL } from "@/features/workspace/domain";
import type { ResolvedTheme } from "@/features/workspace/theme";
import { BrowserPane, type BrowserPaneHandle } from "./BrowserPane";
import { PaneHeader } from "./PaneHeader";
import { TerminalPane } from "./TerminalPane";

// ---------------------------------------------------------------------------
// Context — holds cross-cutting state shared by all nodes in the tree
// ---------------------------------------------------------------------------

interface SplitTreeCtx {
  tab: TabSnapshot;
  fontSize: number;
  theme: ResolvedTheme;
  visible: boolean;
  onFocus: (tabId: string, paneId: string) => Promise<void>;
  onRestart: (paneId: string) => Promise<void>;
  onClosePane: (paneId: string) => void;
  onSwapPanes: (paneIdA: string, paneIdB: string) => void;
  dragSourceRef: React.MutableRefObject<string | null>;
  dragOverPaneId: string | null;
  onDragOverChange: (paneId: string | null) => void;
  browserUrls: Record<string, string>;
  onBrowserUrlChange: (paneId: string, url: string) => void;
  browserPaneRefs: React.MutableRefObject<Record<string, BrowserPaneHandle | null>>;
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
  tab: TabSnapshot;
  fontSize: number;
  theme: ResolvedTheme;
  visible: boolean;
  onFocus: (tabId: string, paneId: string) => Promise<void>;
  onRestart: (paneId: string) => Promise<void>;
  onClosePane: (paneId: string) => void;
  onSwapPanes: (paneIdA: string, paneIdB: string) => void;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function findPaneById(tab: TabSnapshot, paneId: string): PaneSnapshot | undefined {
  return tab.panes.find((p) => p.id === paneId);
}

// ---------------------------------------------------------------------------
// PaneLeaf — renders a single pane (terminal or browser)
// ---------------------------------------------------------------------------

function PaneLeaf({ paneId }: { paneId: string }) {
  const ctx = useTreeContext();
  const {
    tab, fontSize, theme, visible,
    onFocus, onRestart, onClosePane, onSwapPanes,
    dragSourceRef, dragOverPaneId, onDragOverChange,
    browserUrls, onBrowserUrlChange, browserPaneRefs,
  } = ctx;

  const pane = findPaneById(tab, paneId);
  if (!pane) return null;

  const isActive = tab.activePaneId === pane.id;
  const isDragOver = dragOverPaneId === pane.id;
  const isDragSource = dragSourceRef.current === pane.id;
  const isBrowser = pane.paneKind === "browser";

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
        <PaneHeader
          profileLabel={pane.profileLabel}
          cwd={pane.cwd}
          isActive={isActive}
          paneCount={tab.panes.length}
          onClose={() => onClosePane(pane.id)}
          draggable
          isDragOver={isDragOver}
          isBrowser={isBrowser}
          browserUrl={isBrowser ? (browserUrls[pane.id] ?? pane.url ?? DEFAULT_BROWSER_URL) : undefined}
          onBrowserNavigate={isBrowser ? (url) => {
            browserPaneRefs.current[pane.id]?.navigate(url);
          } : undefined}
          onBrowserReload={isBrowser ? () => {
            const currentUrl = browserUrls[pane.id] ?? pane.url ?? DEFAULT_BROWSER_URL;
            browserPaneRefs.current[pane.id]?.navigate(currentUrl);
          } : undefined}
          onDragStart={(e) => {
            dragSourceRef.current = pane.id;
            e.dataTransfer.effectAllowed = "move";
            e.dataTransfer.setData("text/plain", pane.id);
          }}
          onDragOver={(e) => {
            e.preventDefault();
            e.dataTransfer.dropEffect = "move";
          }}
          onDragEnter={(e) => {
            e.preventDefault();
            if (dragSourceRef.current && dragSourceRef.current !== pane.id) {
              onDragOverChange(pane.id);
            }
          }}
          onDragLeave={() => {
            if (dragOverPaneId === pane.id) {
              onDragOverChange(null);
            }
          }}
          onDrop={(e) => {
            e.preventDefault();
            const sourceId = dragSourceRef.current;
            if (sourceId && sourceId !== pane.id) {
              onSwapPanes(sourceId, pane.id);
            }
            dragSourceRef.current = null;
            onDragOverChange(null);
          }}
          onDragEnd={() => {
            dragSourceRef.current = null;
            onDragOverChange(null);
          }}
        />
        <div
          className="min-h-0 flex-1"
          onMouseDown={() => void onFocus(tab.id, pane.id)}
        >
          {isBrowser ? (
            <BrowserPane
              ref={(handle) => {
                browserPaneRefs.current[pane.id] = handle;
              }}
              pane={pane}
              active={isActive}
              visible={visible}
              onUrlChange={(url) => onBrowserUrlChange(pane.id, url)}
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
  onFocus,
  onRestart,
  onClosePane,
  onSwapPanes,
}: SplitTreeRendererProps) {
  const dragSourceRef = useRef<string | null>(null);
  const [dragOverPaneId, setDragOverPaneId] = useState<string | null>(null);
  const [browserUrls, setBrowserUrls] = useState<Record<string, string>>({});
  const browserPaneRefs = useRef<Record<string, BrowserPaneHandle | null>>({});

  const handleDragOverChange = useCallback((paneId: string | null) => {
    setDragOverPaneId(paneId);
  }, []);

  const handleBrowserUrlChange = useCallback((paneId: string, url: string) => {
    setBrowserUrls((prev) => ({ ...prev, [paneId]: url }));
  }, []);

  const handleClosePane = useCallback((paneId: string) => {
    setBrowserUrls((prev) => {
      const { [paneId]: _, ...rest } = prev;
      return rest;
    });
    delete browserPaneRefs.current[paneId];
    onClosePane(paneId);
  }, [onClosePane]);

  const ctx: SplitTreeCtx = {
    tab,
    fontSize,
    theme,
    visible,
    onFocus,
    onRestart,
    onClosePane: handleClosePane,
    onSwapPanes,
    dragSourceRef,
    dragOverPaneId,
    onDragOverChange: handleDragOverChange,
    browserUrls,
    onBrowserUrlChange: handleBrowserUrlChange,
    browserPaneRefs,
  };

  return (
    <TreeContext.Provider value={ctx}>
      <NodeRenderer node={tab.layout} />
    </TreeContext.Provider>
  );
}
