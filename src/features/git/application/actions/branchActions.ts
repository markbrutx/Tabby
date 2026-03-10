import type { GitClient } from "@/app-shell/clients";
import type { GitPaneState } from "../useGitPaneStore";

type Set = (partial: Partial<GitPaneState>) => void;
type Get = () => GitPaneState;

export function createBranchActions(
  gitClient: GitClient,
  paneId: string,
  set: Set,
  get: Get,
) {
  return {
    async listBranches() {
      set({ branchesLoading: true });
      try {
        const result = await gitClient.dispatch({
          kind: "branches",
          pane_id: paneId,
        });
        if (result.kind === "branches") {
          set({ branches: result.branches, branchesLoading: false });
        } else {
          set({ branchesLoading: false });
        }
      } catch (err: unknown) {
        const message =
          err instanceof Error ? err.message : "Failed to list branches";
        set({ error: message, branchesLoading: false });
      }
    },

    async checkoutBranch(name: string) {
      try {
        await gitClient.dispatch({
          kind: "checkoutBranch",
          pane_id: paneId,
          name,
        });
        await Promise.all([get().listBranches(), get().refreshStatus()]);
      } catch (err: unknown) {
        const message =
          err instanceof Error ? err.message : "Failed to checkout branch";
        set({ error: message });
      }
    },

    async createBranch(name: string, startPoint: string | null) {
      try {
        await gitClient.dispatch({
          kind: "createBranch",
          pane_id: paneId,
          name,
          start_point: startPoint,
        });
        await Promise.all([get().listBranches(), get().refreshStatus()]);
      } catch (err: unknown) {
        const message =
          err instanceof Error ? err.message : "Failed to create branch";
        set({ error: message });
      }
    },

    async deleteBranch(name: string, force: boolean) {
      try {
        await gitClient.dispatch({
          kind: "deleteBranch",
          pane_id: paneId,
          name,
          force,
        });
        await get().listBranches();
      } catch (err: unknown) {
        const message =
          err instanceof Error ? err.message : "Failed to delete branch";
        set({ error: message });
      }
    },
  };
}
