import { create } from "zustand";
import type { GitClient } from "@/app-shell/clients";
import type {
  BlameEntry,
  BranchInfo,
  CommitInfo,
  DiffContent,
  FileStatus,
  GitRepoState,
  StashEntry,
} from "@/features/git/domain/models";
import { createStatusActions } from "./actions/statusActions";
import { createStagingActions } from "./actions/stagingActions";
import { createCommitActions } from "./actions/commitActions";
import { createBranchActions } from "./actions/branchActions";
import { createHistoryActions } from "./actions/historyActions";
import { createBlameActions } from "./actions/blameActions";
import { createStashActions } from "./actions/stashActions";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type GitActiveView = "changes" | "history" | "branches" | "stash" | "blame";

export interface GitPaneState {
  readonly files: readonly FileStatus[];
  readonly selectedFile: string | null;
  readonly diffContent: DiffContent | null;
  readonly repoState: GitRepoState | null;
  readonly activeView: GitActiveView;
  readonly loading: boolean;
  readonly error: string | null;
  readonly stagedLines: ReadonlySet<string>;
  readonly branches: readonly BranchInfo[];
  readonly branchesLoading: boolean;
  readonly commitLog: readonly CommitInfo[];
  readonly commitLogLoading: boolean;
  readonly hasMoreCommits: boolean;
  readonly selectedCommitHash: string | null;
  readonly commitDiffContent: DiffContent | null;
  readonly blameEntries: readonly BlameEntry[];
  readonly blameFilePath: string | null;
  readonly blameLoading: boolean;
  readonly stashes: readonly StashEntry[];
  readonly stashesLoading: boolean;

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
  commit: (message: string, amend: boolean) => Promise<void>;
  fetchLastCommitInfo: () => Promise<CommitInfo | null>;
  listBranches: () => Promise<void>;
  checkoutBranch: (name: string) => Promise<void>;
  createBranch: (name: string, startPoint: string | null) => Promise<void>;
  deleteBranch: (name: string, force: boolean) => Promise<void>;
  fetchCommitLog: () => Promise<void>;
  fetchMoreCommits: () => Promise<void>;
  selectCommit: (hash: string | null) => Promise<void>;
  fetchBlame: (filePath: string) => Promise<void>;
  listStashes: () => Promise<void>;
  stashPush: (message: string | null) => Promise<void>;
  stashPop: (index: number) => Promise<void>;
  stashApply: (index: number) => Promise<void>;
  stashDrop: (index: number) => Promise<void>;
  pushAll: (message: string) => Promise<void>;
}

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------

interface GitPaneStoreDeps {
  readonly gitClient: GitClient;
  readonly paneId: string;
}

export function createGitPaneStore(deps: GitPaneStoreDeps) {
  const { gitClient, paneId } = deps;

  return create<GitPaneState>((set, get) => ({
    // Initial state
    files: [],
    selectedFile: null,
    diffContent: null,
    repoState: null,
    activeView: "changes",
    loading: true,
    error: null,
    stagedLines: new Set<string>(),
    branches: [],
    branchesLoading: false,
    commitLog: [],
    commitLogLoading: false,
    hasMoreCommits: true,
    selectedCommitHash: null,
    commitDiffContent: null,
    blameEntries: [],
    blameFilePath: null,
    blameLoading: false,
    stashes: [],
    stashesLoading: false,

    // Composed actions
    ...createStatusActions(gitClient, paneId, set),
    ...createStagingActions(gitClient, paneId, set, get),
    ...createCommitActions(gitClient, paneId, get),
    ...createBranchActions(gitClient, paneId, set, get),
    ...createHistoryActions(gitClient, paneId, set, get),
    ...createBlameActions(gitClient, paneId, set),
    ...createStashActions(gitClient, paneId, set, get),
  }));
}
