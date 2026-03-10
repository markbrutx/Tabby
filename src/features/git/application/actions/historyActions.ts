import type { GitClient } from "@/app-shell/clients";
import type { CommitInfo } from "@/features/git/domain/models";
import type { GitPaneState } from "../useGitPaneStore";

type Set = (partial: Partial<GitPaneState>) => void;
type Get = () => GitPaneState;

const PAGE_SIZE = 50;

function mapCommit(c: CommitInfo): CommitInfo {
  return {
    hash: c.hash,
    shortHash: c.shortHash,
    authorName: c.authorName,
    authorEmail: c.authorEmail,
    date: c.date,
    message: c.message,
    parentHashes: [...c.parentHashes],
  };
}

export function createHistoryActions(
  gitClient: GitClient,
  paneId: string,
  set: Set,
  get: Get,
) {
  return {
    async fetchCommitLog() {
      set({ commitLogLoading: true });
      try {
        const result = await gitClient.dispatch({
          kind: "log",
          pane_id: paneId,
          max_count: PAGE_SIZE,
          skip: null,
          path: null,
        });
        if (result.kind === "log") {
          set({
            commitLog: result.commits.map(mapCommit),
            commitLogLoading: false,
            hasMoreCommits: result.commits.length >= PAGE_SIZE,
          });
        } else {
          set({ commitLogLoading: false });
        }
      } catch (err: unknown) {
        const message =
          err instanceof Error ? err.message : "Failed to fetch commit log";
        set({ error: message, commitLogLoading: false });
      }
    },

    async fetchMoreCommits() {
      const currentState = get();
      if (currentState.commitLogLoading || !currentState.hasMoreCommits) return;

      set({ commitLogLoading: true });
      try {
        const result = await gitClient.dispatch({
          kind: "log",
          pane_id: paneId,
          max_count: PAGE_SIZE,
          skip: currentState.commitLog.length,
          path: null,
        });
        if (result.kind === "log") {
          const newCommits: readonly CommitInfo[] = result.commits.map(mapCommit);
          set({
            commitLog: [...currentState.commitLog, ...newCommits],
            commitLogLoading: false,
            hasMoreCommits: result.commits.length >= PAGE_SIZE,
          });
        } else {
          set({ commitLogLoading: false });
        }
      } catch (err: unknown) {
        const message =
          err instanceof Error ? err.message : "Failed to fetch more commits";
        set({ error: message, commitLogLoading: false });
      }
    },

    async selectCommit(hash: string | null) {
      if (hash === null) {
        set({ selectedCommitHash: null, commitDiffContent: null });
        return;
      }

      set({ selectedCommitHash: hash });
      try {
        const result = await gitClient.dispatch({
          kind: "showCommit",
          pane_id: paneId,
          hash,
        });
        if (result.kind === "showCommit" && result.diffs.length > 0) {
          const d = result.diffs[0];
          set({
            commitDiffContent: {
              filePath: d.filePath,
              oldPath: d.oldPath,
              hunks: d.hunks.map((h) => ({
                oldStart: h.oldStart,
                oldCount: h.oldCount,
                newStart: h.newStart,
                newCount: h.newCount,
                header: h.header,
                lines: h.lines.map((l) => ({
                  kind: l.kind,
                  oldLineNo: l.oldLineNo,
                  newLineNo: l.newLineNo,
                  content: l.content,
                })),
              })),
              isBinary: d.isBinary,
              fileModeChange: d.fileModeChange,
            },
          });
        } else {
          set({ commitDiffContent: null });
        }
      } catch (err: unknown) {
        const message =
          err instanceof Error ? err.message : "Failed to load commit diff";
        set({ error: message });
      }
    },
  };
}
