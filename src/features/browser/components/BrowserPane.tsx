import { forwardRef, useImperativeHandle } from "react";
import type { PaneSnapshot } from "@/features/workspace/domain";
import { useBrowserWebview } from "@/features/browser/hooks/useBrowserWebview";
import { isTauriRuntime } from "@/lib/runtime";

interface BrowserPaneProps {
  pane: PaneSnapshot;
  active: boolean;
  visible: boolean;
  modalOpen?: boolean;
  onUrlChange?: (url: string) => void;
}

export interface BrowserPaneHandle {
  navigate: (url: string) => void;
}

export const BrowserPane = forwardRef<BrowserPaneHandle, BrowserPaneProps>(
  function BrowserPane({ pane, active, visible, modalOpen = false, onUrlChange }, ref) {
    const effectiveVisible = visible && !modalOpen;
    const { containerRef, currentUrl, navigate } = useBrowserWebview({
      pane,
      visible: effectiveVisible,
      onUrlChange,
    });

    const isTauri = isTauriRuntime();

    useImperativeHandle(ref, () => ({ navigate }), [navigate]);

    // Show dimmed overlay when modal hides the native webview
    const showOverlay = isTauri && visible && modalOpen;

    return (
      <div
        ref={containerRef}
        data-testid={`browser-pane-${pane.id}`}
        data-active={active ? "true" : "false"}
        className="relative h-full bg-[var(--color-bg)]"
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
        {showOverlay && (
          <div className="absolute inset-0 flex items-center justify-center bg-[var(--color-bg)]/80">
            <span className="text-xs text-[var(--color-text-muted)]">
              {currentUrl}
            </span>
          </div>
        )}
      </div>
    );
  },
);
