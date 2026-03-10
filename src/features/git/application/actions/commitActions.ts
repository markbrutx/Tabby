import type { GitClient } from "@/app-shell/clients";
import type { CommitInfo } from "@/features/git/domain/models";
import type { GitPaneState } from "../useGitPaneStore";

type Get = () => GitPaneState;

export function createCommitActions(
  gitClient: GitClient,
  paneId: string,
  get: Get,
) {
  return {
    async commit(message: string, amend: boolean) {
      const result = await gitClient.dispatch({
        kind: "commit",
        pane_id: paneId,
        message,
        amend,
      });
      if (result.kind !== "commit") {
        throw new Error("Unexpected commit result");
      }
    },

    async fetchLastCommitInfo(): Promise<CommitInfo | null> {
      const result = await gitClient.dispatch({
        kind: "log",
        pane_id: paneId,
        max_count: 1,
        skip: null,
        path: null,
      });
      if (result.kind === "log" && result.commits.length > 0) {
        const c = result.commits[0];
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
      return null;
    },

    async pushAll(message: string) {
      const commitMessage = message.trim() || "update";

      await gitClient.dispatch({
        kind: "stage",
        pane_id: paneId,
        paths: ["."],
      });

      const statusResult = await gitClient.dispatch({
        kind: "status",
        pane_id: paneId,
      });
      const files = statusResult.kind === "status" ? statusResult.files : [];
      const hasStagedFiles = files.some(
        (f) =>
          f.indexStatus === "modified" ||
          f.indexStatus === "added" ||
          f.indexStatus === "deleted" ||
          f.indexStatus === "renamed" ||
          f.indexStatus === "copied",
      );

      if (hasStagedFiles) {
        await gitClient.dispatch({
          kind: "commit",
          pane_id: paneId,
          message: commitMessage,
          amend: false,
        });
      }

      await gitClient.dispatch({
        kind: "push",
        pane_id: paneId,
        remote: null,
        branch: null,
      });

      await get().refreshStatus();
    },
  };
}
