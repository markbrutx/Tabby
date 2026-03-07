import {
  Panel,
  PanelGroup,
  PanelResizeHandle,
} from "react-resizable-panels";
import { RefreshCw } from "lucide-react";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { Button } from "@/components/ui/Button";
import type { PaneSnapshot, SplitNode, TabSnapshot } from "@/features/workspace/domain";
import type { ResolvedTheme } from "@/features/workspace/theme";
import { TerminalPane } from "./TerminalPane";

interface SplitTreeRendererProps {
  tab: TabSnapshot;
  fontSize: number;
  theme: ResolvedTheme;
  visible: boolean;
  onFocus: (tabId: string, paneId: string) => Promise<void>;
  onRestart: (paneId: string) => Promise<void>;
}

interface NodeRendererProps {
  node: SplitNode;
  tab: TabSnapshot;
  fontSize: number;
  theme: ResolvedTheme;
  visible: boolean;
  onFocus: (tabId: string, paneId: string) => Promise<void>;
  onRestart: (paneId: string) => Promise<void>;
}

function findPaneById(tab: TabSnapshot, paneId: string): PaneSnapshot | undefined {
  return tab.panes.find((p) => p.id === paneId);
}

function NodeRenderer({
  node,
  tab,
  fontSize,
  theme,
  visible,
  onFocus,
  onRestart,
}: NodeRendererProps) {
  if (node.type === "pane") {
    const pane = findPaneById(tab, node.paneId);
    if (!pane) return null;

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
        <TerminalPane
          pane={pane}
          fontSize={fontSize}
          theme={theme}
          active={tab.activePaneId === pane.id}
          visible={visible}
          onFocus={(paneId) => onFocus(tab.id, paneId)}
        />
      </ErrorBoundary>
    );
  }

  const direction = node.direction === "horizontal" ? "horizontal" : "vertical";
  const firstSize = (node.ratio / 1000) * 100;
  const secondSize = 100 - firstSize;

  return (
    <PanelGroup direction={direction} className="h-full">
      <Panel defaultSize={firstSize} minSize={5}>
        <NodeRenderer
          node={node.first}
          tab={tab}
          fontSize={fontSize}
          theme={theme}
          visible={visible}
          onFocus={onFocus}
          onRestart={onRestart}
        />
      </Panel>
      <PanelResizeHandle
        className={`resize-handle ${
          direction === "horizontal" ? "w-[3px]" : "h-[3px]"
        } shrink-0 bg-[var(--color-border)] transition-colors hover:bg-[var(--color-accent)]`}
      />
      <Panel defaultSize={secondSize} minSize={5}>
        <NodeRenderer
          node={node.second}
          tab={tab}
          fontSize={fontSize}
          theme={theme}
          visible={visible}
          onFocus={onFocus}
          onRestart={onRestart}
        />
      </Panel>
    </PanelGroup>
  );
}

export function SplitTreeRenderer({
  tab,
  fontSize,
  theme,
  visible,
  onFocus,
  onRestart,
}: SplitTreeRendererProps) {
  return (
    <NodeRenderer
      node={tab.layout}
      tab={tab}
      fontSize={fontSize}
      theme={theme}
      visible={visible}
      onFocus={onFocus}
      onRestart={onRestart}
    />
  );
}
