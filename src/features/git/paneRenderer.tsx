import { registerPaneRenderer, type PaneRendererContext } from "@/features/workspace/paneRegistry";
import type { GitClient } from "@/app-shell/clients";
import { GitPaneHeader } from "./components/GitPaneHeader";
import { GitPane } from "./components/GitPane";

registerPaneRenderer("git", {
  renderHeader(ctx: PaneRendererContext) {
    return (
      <GitPaneHeader
        repoPath={ctx.pane.gitRepoPath ?? ctx.pane.cwd}
        branch={null}
        isActive={ctx.isActive}
        paneCount={ctx.paneCount}
        onClose={ctx.onClose}
        {...ctx.dragProps}
      />
    );
  },

  renderContent(ctx: PaneRendererContext) {
    const gitClient = ctx.extras.gitClient as GitClient;

    return <GitPane pane={ctx.pane} gitClient={gitClient} />;
  },
});
