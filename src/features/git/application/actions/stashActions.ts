import type { GitClient } from "@/app-shell/clients";
import type { GitPaneState } from "../useGitPaneStore";

type Set = (partial: Partial<GitPaneState>) => void;
type Get = () => GitPaneState;

export function createStashActions(
  gitClient: GitClient,
  paneId: string,
  set: Set,
  get: Get,
) {
  return {
    async listStashes() {
      set({ stashesLoading: true });
      try {
        const result = await gitClient.dispatch({
          kind: "stashList",
          pane_id: paneId,
        });
        if (result.kind === "stashList") {
          set({
            stashes: result.entries.map((e) => ({
              index: e.index,
              message: e.message,
              date: e.date,
            })),
            stashesLoading: false,
          });
        } else {
          set({ stashesLoading: false });
        }
      } catch (err: unknown) {
        const message =
          err instanceof Error ? err.message : "Failed to list stashes";
        set({ error: message, stashesLoading: false });
      }
    },

    async stashPush(message: string | null) {
      try {
        await gitClient.dispatch({
          kind: "stashPush",
          pane_id: paneId,
          message,
        });
        await Promise.all([get().listStashes(), get().refreshStatus()]);
      } catch (err: unknown) {
        const msg =
          err instanceof Error ? err.message : "Failed to push stash";
        set({ error: msg });
      }
    },

    async stashPop(index: number) {
      try {
        await gitClient.dispatch({
          kind: "stashPop",
          pane_id: paneId,
          index,
        });
        await Promise.all([get().listStashes(), get().refreshStatus()]);
      } catch (err: unknown) {
        const msg =
          err instanceof Error ? err.message : "Failed to pop stash";
        set({ error: msg });
      }
    },

    async stashApply(index: number) {
      try {
        await gitClient.dispatch({
          kind: "stashPop",
          pane_id: paneId,
          index,
        });
        await Promise.all([get().listStashes(), get().refreshStatus()]);
      } catch (err: unknown) {
        const msg =
          err instanceof Error ? err.message : "Failed to apply stash";
        set({ error: msg });
      }
    },

    async stashDrop(index: number) {
      try {
        await gitClient.dispatch({
          kind: "stashDrop",
          pane_id: paneId,
          index,
        });
        await get().listStashes();
      } catch (err: unknown) {
        const msg =
          err instanceof Error ? err.message : "Failed to drop stash";
        set({ error: msg });
      }
    },
  };
}
