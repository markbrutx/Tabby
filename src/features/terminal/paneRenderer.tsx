import { registerPaneRenderer, type PaneRendererContext } from "@/features/workspace/paneRegistry";
import { PaneHeader } from "./components/PaneHeader";
import { TerminalPane } from "./components/TerminalPane";

registerPaneRenderer("terminal", {
  renderHeader(ctx: PaneRendererContext) {
    const onOpenGitView = ctx.extras.onOpenGitView as
      | ((paneId: string, cwd: string) => void)
      | undefined;

    return (
      <PaneHeader
        profileLabel={ctx.pane.profileLabel}
        cwd={ctx.pane.cwd}
        isActive={ctx.isActive}
        paneCount={ctx.paneCount}
        onClose={ctx.onClose}
        onRestart={ctx.onRestart}
        onOpenGitView={
          onOpenGitView && ctx.pane.cwd
            ? () => onOpenGitView(ctx.pane.id, ctx.pane.cwd)
            : undefined
        }
        {...ctx.dragProps}
      />
    );
  },

  renderContent(ctx: PaneRendererContext) {
    return (
      <TerminalPane
        pane={ctx.pane}
        theme={ctx.theme}
        active={ctx.isActive}
        visible={ctx.visible}
      />
    );
  },
});
