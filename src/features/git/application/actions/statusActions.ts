import type { GitClient } from "@/app-shell/clients";
import type { GitPaneState } from "../useGitPaneStore";

type Set = (partial: Partial<GitPaneState>) => void;

export function createStatusActions(
  gitClient: GitClient,
  paneId: string,
  set: Set,
) {
  return {
    async refreshStatus() {
      set({ loading: true, error: null });
      try {
        const [statusResult, repoStateResult] = await Promise.all([
          gitClient.dispatch({ kind: "status", pane_id: paneId }),
          gitClient.dispatch({ kind: "repoState", pane_id: paneId }),
        ]);

        const files =
          statusResult.kind === "status" ? statusResult.files : [];
        const repoState =
          repoStateResult.kind === "repoState"
            ? repoStateResult.state
            : null;

        set({ files, repoState, loading: false });
      } catch (err: unknown) {
        const message =
          err instanceof Error ? err.message : "Failed to refresh status";
        set({ error: message, loading: false });
      }
    },

    async selectFile(filePath: string | null) {
      if (filePath === null) {
        set({ selectedFile: null, diffContent: null });
        return;
      }

      set({ selectedFile: filePath });

      try {
        const result = await gitClient.dispatch({
          kind: "diff",
          pane_id: paneId,
          path: filePath,
          staged: false,
        });

        if (result.kind === "diff") {
          const diff = result.diffs.find((d) => d.filePath === filePath) ?? null;
          set({ diffContent: diff });
        }
      } catch (err: unknown) {
        const message =
          err instanceof Error ? err.message : "Failed to load diff";
        set({ error: message });
      }
    },

    setActiveView(view: GitPaneState["activeView"]) {
      set({ activeView: view });
    },
  };
}
