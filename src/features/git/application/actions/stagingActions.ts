import type { GitClient } from "@/app-shell/clients";
import type { GitPaneState } from "../useGitPaneStore";
import { hunkLineRanges } from "@/features/git/domain/stagingHelpers";

type Set = (partial: Partial<GitPaneState>) => void;
type Get = () => GitPaneState;

export function createStagingActions(
  gitClient: GitClient,
  paneId: string,
  set: Set,
  get: Get,
) {
  async function refreshAfterStaging(filePath: string) {
    const [statusResult, diffResult] = await Promise.all([
      gitClient.dispatch({ kind: "status", pane_id: paneId }),
      gitClient.dispatch({ kind: "diff", pane_id: paneId, path: filePath, staged: false }),
    ]);
    const files = statusResult.kind === "status" ? statusResult.files : [];
    const diff = diffResult.kind === "diff"
      ? diffResult.diffs.find((d) => d.filePath === filePath) ?? null
      : null;
    set({ files, diffContent: diff, stagedLines: new Set<string>() });
  }

  return {
    async stageFiles(paths: readonly string[]) {
      try {
        await gitClient.dispatch({
          kind: "stage",
          pane_id: paneId,
          paths: [...paths],
        });
        const statusResult = await gitClient.dispatch({
          kind: "status",
          pane_id: paneId,
        });
        const files =
          statusResult.kind === "status" ? statusResult.files : [];
        set({ files });
      } catch (err: unknown) {
        const message =
          err instanceof Error ? err.message : "Failed to stage files";
        set({ error: message });
      }
    },

    async unstageFiles(paths: readonly string[]) {
      try {
        await gitClient.dispatch({
          kind: "unstage",
          pane_id: paneId,
          paths: [...paths],
        });
        const statusResult = await gitClient.dispatch({
          kind: "status",
          pane_id: paneId,
        });
        const files =
          statusResult.kind === "status" ? statusResult.files : [];
        set({ files });
      } catch (err: unknown) {
        const message =
          err instanceof Error ? err.message : "Failed to unstage files";
        set({ error: message });
      }
    },

    async discardChanges(paths: readonly string[]) {
      try {
        await gitClient.dispatch({
          kind: "discardChanges",
          pane_id: paneId,
          paths: [...paths],
        });
        const statusResult = await gitClient.dispatch({
          kind: "status",
          pane_id: paneId,
        });
        const files =
          statusResult.kind === "status" ? statusResult.files : [];
        set({ files });
      } catch (err: unknown) {
        const message =
          err instanceof Error ? err.message : "Failed to discard changes";
        set({ error: message });
      }
    },

    async stageLines(filePath: string, lineRanges: string[]) {
      try {
        await gitClient.dispatch({
          kind: "stageLines",
          pane_id: paneId,
          path: filePath,
          line_ranges: lineRanges,
        });
        await refreshAfterStaging(filePath);
      } catch (err: unknown) {
        const message = err instanceof Error ? err.message : "Failed to stage lines";
        set({ error: message });
      }
    },

    async unstageLines(filePath: string, lineRanges: string[]) {
      try {
        await gitClient.dispatch({
          kind: "stageLines",
          pane_id: paneId,
          path: filePath,
          line_ranges: lineRanges,
        });
        await refreshAfterStaging(filePath);
      } catch (err: unknown) {
        const message = err instanceof Error ? err.message : "Failed to unstage lines";
        set({ error: message });
      }
    },

    async stageHunk(filePath: string, hunkIndex: number) {
      const currentState = get();
      const hunk = currentState.diffContent?.hunks[hunkIndex];
      if (hunk === undefined) return;
      const ranges = hunkLineRanges(hunk);
      if (ranges.length === 0) return;
      await currentState.stageLines(filePath, ranges);
    },

    async unstageHunk(filePath: string, hunkIndex: number) {
      const currentState = get();
      const hunk = currentState.diffContent?.hunks[hunkIndex];
      if (hunk === undefined) return;
      const ranges = hunkLineRanges(hunk);
      if (ranges.length === 0) return;
      await currentState.unstageLines(filePath, ranges);
    },
  };
}
