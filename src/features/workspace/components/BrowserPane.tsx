import { forwardRef, useImperativeHandle } from "react";
import type { PaneSnapshot } from "@/features/workspace/domain";
import { useBrowserWebview } from "@/features/workspace/hooks/useBrowserWebview";
import { isTauriRuntime } from "@/lib/runtime";

interface BrowserPaneProps {
  pane: PaneSnapshot;
  active: boolean;
  visible: boolean;
  onUrlChange?: (url: string) => void;
}

export interface BrowserPaneHandle {
  navigate: (url: string) => void;
}

export const BrowserPane = forwardRef<BrowserPaneHandle, BrowserPaneProps>(
  function BrowserPane({ pane, active, visible, onUrlChange }, ref) {
    const { containerRef, currentUrl, navigate } = useBrowserWebview({
      pane,
      visible,
      onUrlChange,
    });

    const isTauri = isTauriRuntime();

    useImperativeHandle(ref, () => ({ navigate }), [navigate]);

    return (
      <div
        ref={containerRef}
        data-testid={`browser-pane-${pane.id}`}
        data-active={active ? "true" : "false"}
        className="h-full bg-[var(--color-bg)]"
      >
        {!isTauri && (
          <iframe
            src={currentUrl}
            title={`Browser pane ${pane.id}`}
            className="h-full w-full border-none"
            sandbox="allow-scripts allow-same-origin allow-forms allow-popups"
            data-testid="browser-iframe"
          />
        )}
      </div>
    );
  },
);
