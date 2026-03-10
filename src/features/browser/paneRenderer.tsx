import { useRef } from "react";
import { registerPaneRenderer, type PaneRendererContext } from "@/features/workspace/paneRegistry";
import { DEFAULT_BROWSER_URL } from "@/features/workspace/domain/models";
import { BrowserToolbar } from "./components/BrowserToolbar";
import { BrowserPane, type BrowserPaneHandle } from "./components/BrowserPane";

function BrowserPaneHeader({ ctx }: { ctx: PaneRendererContext }) {
  const browserPaneRef = ctx.extras.browserPaneRef as React.RefObject<BrowserPaneHandle | null>;

  return (
    <BrowserToolbar
      url={ctx.pane.url ?? DEFAULT_BROWSER_URL}
      isActive={ctx.isActive}
      paneCount={ctx.paneCount}
      isCollapsed={ctx.isCollapsed}
      onToggleCollapse={ctx.onToggleCollapse}
      onNavigate={(url) => {
        browserPaneRef.current?.navigate(url);
      }}
      onReload={() => {
        browserPaneRef.current?.navigate(ctx.pane.url ?? DEFAULT_BROWSER_URL);
      }}
      onClose={ctx.onClose}
      {...ctx.dragProps}
    />
  );
}

function BrowserPaneContent({ ctx }: { ctx: PaneRendererContext }) {
  const browserPaneRef = ctx.extras.browserPaneRef as React.RefObject<BrowserPaneHandle>;

  return (
    <BrowserPane
      ref={browserPaneRef}
      pane={ctx.pane}
      active={ctx.isActive}
      visible={ctx.visible}
      modalOpen={ctx.modalOpen}
    />
  );
}

registerPaneRenderer("browser", {
  renderHeader(ctx: PaneRendererContext) {
    return <BrowserPaneHeader ctx={ctx} />;
  },
  renderContent(ctx: PaneRendererContext) {
    return <BrowserPaneContent ctx={ctx} />;
  },
});
