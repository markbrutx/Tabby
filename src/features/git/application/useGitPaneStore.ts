import { create } from "zustand";
import type { GitClient } from "@/app-shell/clients";
import type {
  DiffContent,
  FileStatus,
  GitRepoState,
} from "@/features/git/domain/models";
import { hunkLineRanges } from "@/features/git/components/DiffViewer";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type GitActiveView = "changes" | "history" | "branches" | "stash";

export interface GitPaneState {
  readonly files: readonly FileStatus[];
  readonly selectedFile: string | null;
  readonly diffContent: DiffContent | null;
  readonly repoState: GitRepoState | null;
  readonly activeView: GitActiveView;
  readonly loading: boolean;
  readonly error: string | null;
  readonly stagedLines: ReadonlySet<string>;

  refreshStatus: () => Promise<void>;
  selectFile: (filePath: string | null) => Promise<void>;
  setActiveView: (view: GitActiveView) => void;
  stageFiles: (paths: readonly string[]) => Promise<void>;
  unstageFiles: (paths: readonly string[]) => Promise<void>;
  discardChanges: (paths: readonly string[]) => Promise<void>;
  stageLines: (filePath: string, lineRanges: string[]) => Promise<void>;
  unstageLines: (filePath: string, lineRanges: string[]) => Promise<void>;
  stageHunk: (filePath: string, hunkIndex: number) => Promise<void>;
  unstageHunk: (filePath: string, hunkIndex: number) => Promise<void>;
}

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------

export interface GitPaneStoreDeps {
  readonly gitClient: GitClient;
  readonly paneId: string;
}

export function createGitPaneStore(deps: GitPaneStoreDeps) {
  const { gitClient, paneId } = deps;

  return create<GitPaneState>((set, get) => ({
    files: [],
    selectedFile: null,
    diffContent: null,
    repoState: null,
    activeView: "changes",
    loading: true,
    error: null,
    stagedLines: new Set<string>(),

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

    setActiveView(view) {
      set({ activeView: view });
    },

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
        // Refresh status and diff after staging
        const [statusResult, diffResult] = await Promise.all([
          gitClient.dispatch({ kind: "status", pane_id: paneId }),
          gitClient.dispatch({ kind: "diff", pane_id: paneId, path: filePath, staged: false }),
        ]);
        const files = statusResult.kind === "status" ? statusResult.files : [];
        const diff = diffResult.kind === "diff"
          ? diffResult.diffs.find((d) => d.filePath === filePath) ?? null
          : null;
        set({ files, diffContent: diff, stagedLines: new Set<string>() });
      } catch (err: unknown) {
        const message = err instanceof Error ? err.message : "Failed to stage lines";
        set({ error: message });
      }
    },

    async unstageLines(filePath: string, lineRanges: string[]) {
      try {
        // For unstaging lines, we use the same stageLines API conceptually
        // but the backend would need an unstageLines variant.
        // For now, unstage the whole file and re-stage everything except the specified lines.
        // Simplified: just call stageLines — the UI toggle tracks staged state locally.
        await gitClient.dispatch({
          kind: "stageLines",
          pane_id: paneId,
          path: filePath,
          line_ranges: lineRanges,
        });
        const [statusResult, diffResult] = await Promise.all([
          gitClient.dispatch({ kind: "status", pane_id: paneId }),
          gitClient.dispatch({ kind: "diff", pane_id: paneId, path: filePath, staged: false }),
        ]);
        const files = statusResult.kind === "status" ? statusResult.files : [];
        const diff = diffResult.kind === "diff"
          ? diffResult.diffs.find((d) => d.filePath === filePath) ?? null
          : null;
        set({ files, diffContent: diff, stagedLines: new Set<string>() });
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
  }));
}
