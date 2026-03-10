import type { GitClient } from "@/app-shell/clients";
import type { GitPaneState } from "../useGitPaneStore";

type Set = (partial: Partial<GitPaneState>) => void;

export function createBlameActions(
  gitClient: GitClient,
  paneId: string,
  set: Set,
) {
  return {
    async fetchBlame(filePath: string) {
      set({ blameLoading: true, blameFilePath: filePath, activeView: "blame" });
      try {
        const result = await gitClient.dispatch({
          kind: "blame",
          pane_id: paneId,
          path: filePath,
        });
        if (result.kind === "blame") {
          set({
            blameEntries: result.entries.map((e) => ({
              hash: e.hash,
              author: e.author,
              date: e.date,
              lineStart: e.lineStart,
              lineCount: e.lineCount,
              content: e.content,
            })),
            blameLoading: false,
          });
        } else {
          set({ blameLoading: false });
        }
      } catch (err: unknown) {
        const message =
          err instanceof Error ? err.message : "Failed to fetch blame";
        set({ error: message, blameLoading: false });
      }
    },
  };
}
