use serde::{Deserialize, Serialize};
use specta::Type;

// ---------------------------------------------------------------------------
// Supporting DTOs — mirror tabby-git domain types for IPC serialization
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum FileStatusKindDto {
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Ignored,
    Conflicted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FileStatusDto {
    pub path: String,
    pub old_path: Option<String>,
    pub index_status: FileStatusKindDto,
    pub worktree_status: FileStatusKindDto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum DiffLineKindDto {
    Context,
    Addition,
    Deletion,
    HunkHeader,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DiffLineDto {
    pub kind: DiffLineKindDto,
    pub old_line_no: Option<u32>,
    pub new_line_no: Option<u32>,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DiffHunkDto {
    pub old_start: u32,
    pub old_count: u32,
    pub new_start: u32,
    pub new_count: u32,
    pub header: String,
    pub lines: Vec<DiffLineDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DiffContentDto {
    pub file_path: String,
    pub old_path: Option<String>,
    pub hunks: Vec<DiffHunkDto>,
    pub is_binary: bool,
    pub file_mode_change: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CommitInfoDto {
    pub hash: String,
    pub short_hash: String,
    pub author_name: String,
    pub author_email: String,
    pub date: String,
    pub message: String,
    pub parent_hashes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BranchInfoDto {
    pub name: String,
    pub is_current: bool,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct BlameEntryDto {
    pub hash: String,
    pub author: String,
    pub date: String,
    pub line_start: u32,
    pub line_count: u32,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StashEntryDto {
    pub index: u32,
    pub message: String,
    pub date: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GitRepoStateDto {
    pub repo_path: String,
    pub head_branch: Option<String>,
    pub is_detached: bool,
    pub status_clean: bool,
}

// ---------------------------------------------------------------------------
// GitCommandDto — all Git commands that can be dispatched over IPC
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum GitCommandDto {
    Status {
        pane_id: String,
    },
    Diff {
        pane_id: String,
        path: Option<String>,
        staged: bool,
    },
    Stage {
        pane_id: String,
        paths: Vec<String>,
    },
    Unstage {
        pane_id: String,
        paths: Vec<String>,
    },
    StageLines {
        pane_id: String,
        path: String,
        line_ranges: Vec<String>,
    },
    Commit {
        pane_id: String,
        message: String,
    },
    Push {
        pane_id: String,
        remote: Option<String>,
        branch: Option<String>,
    },
    Pull {
        pane_id: String,
        remote: Option<String>,
        branch: Option<String>,
    },
    Fetch {
        pane_id: String,
        remote: Option<String>,
    },
    Branches {
        pane_id: String,
    },
    CheckoutBranch {
        pane_id: String,
        name: String,
    },
    CreateBranch {
        pane_id: String,
        name: String,
        start_point: Option<String>,
    },
    DeleteBranch {
        pane_id: String,
        name: String,
        force: bool,
    },
    MergeBranch {
        pane_id: String,
        name: String,
    },
    Log {
        pane_id: String,
        max_count: Option<u32>,
        path: Option<String>,
    },
    Blame {
        pane_id: String,
        path: String,
    },
    StashPush {
        pane_id: String,
        message: Option<String>,
    },
    StashPop {
        pane_id: String,
        index: Option<u32>,
    },
    StashList {
        pane_id: String,
    },
    StashDrop {
        pane_id: String,
        index: u32,
    },
    DiscardChanges {
        pane_id: String,
        paths: Vec<String>,
    },
    RepoState {
        pane_id: String,
    },
}

// ---------------------------------------------------------------------------
// GitResultDto — result variants corresponding to each command
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum GitResultDto {
    Status { files: Vec<FileStatusDto> },
    Diff { diffs: Vec<DiffContentDto> },
    Stage,
    Unstage,
    StageLines,
    Commit { hash: String },
    Push,
    Pull,
    Fetch,
    Branches { branches: Vec<BranchInfoDto> },
    CheckoutBranch,
    CreateBranch,
    DeleteBranch,
    MergeBranch { message: String },
    Log { commits: Vec<CommitInfoDto> },
    Blame { entries: Vec<BlameEntryDto> },
    StashPush,
    StashPop,
    StashList { entries: Vec<StashEntryDto> },
    StashDrop,
    DiscardChanges,
    RepoState { state: GitRepoStateDto },
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_command_dto_status_serializes_as_tagged() {
        let cmd = GitCommandDto::Status {
            pane_id: "p1".to_string(),
        };
        let json = serde_json::to_string(&cmd).expect("serialize");
        assert!(json.contains("\"kind\":\"status\""));
        assert!(json.contains("\"pane_id\":\"p1\""));
    }

    #[test]
    fn git_command_dto_commit_roundtrip() {
        let cmd = GitCommandDto::Commit {
            pane_id: "p1".to_string(),
            message: "feat: hello".to_string(),
        };
        let json = serde_json::to_string(&cmd).expect("serialize");
        let deserialized: GitCommandDto = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(cmd, deserialized);
    }

    #[test]
    fn git_command_dto_diff_with_options() {
        let cmd = GitCommandDto::Diff {
            pane_id: "p2".to_string(),
            path: Some("src/main.rs".to_string()),
            staged: true,
        };
        let json = serde_json::to_string(&cmd).expect("serialize");
        assert!(json.contains("\"staged\":true"));
        let deserialized: GitCommandDto = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(cmd, deserialized);
    }

    #[test]
    fn git_command_dto_create_branch_roundtrip() {
        let cmd = GitCommandDto::CreateBranch {
            pane_id: "p1".to_string(),
            name: "feature/test".to_string(),
            start_point: Some("main".to_string()),
        };
        let json = serde_json::to_string(&cmd).expect("serialize");
        let deserialized: GitCommandDto = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(cmd, deserialized);
    }

    #[test]
    fn git_command_dto_stage_lines_roundtrip() {
        let cmd = GitCommandDto::StageLines {
            pane_id: "p1".to_string(),
            path: "file.rs".to_string(),
            line_ranges: vec!["1-5".to_string(), "10-15".to_string()],
        };
        let json = serde_json::to_string(&cmd).expect("serialize");
        let deserialized: GitCommandDto = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(cmd, deserialized);
    }

    #[test]
    fn git_result_dto_status_roundtrip() {
        let result = GitResultDto::Status {
            files: vec![FileStatusDto {
                path: "src/main.rs".to_string(),
                old_path: None,
                index_status: FileStatusKindDto::Modified,
                worktree_status: FileStatusKindDto::Modified,
            }],
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let deserialized: GitResultDto = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(result, deserialized);
    }

    #[test]
    fn git_result_dto_diff_roundtrip() {
        let result = GitResultDto::Diff {
            diffs: vec![DiffContentDto {
                file_path: "src/lib.rs".to_string(),
                old_path: None,
                hunks: vec![DiffHunkDto {
                    old_start: 1,
                    old_count: 3,
                    new_start: 1,
                    new_count: 5,
                    header: "@@ -1,3 +1,5 @@".to_string(),
                    lines: vec![
                        DiffLineDto {
                            kind: DiffLineKindDto::Context,
                            old_line_no: Some(1),
                            new_line_no: Some(1),
                            content: "use std;".to_string(),
                        },
                        DiffLineDto {
                            kind: DiffLineKindDto::Addition,
                            old_line_no: None,
                            new_line_no: Some(2),
                            content: "use serde;".to_string(),
                        },
                    ],
                }],
                is_binary: false,
                file_mode_change: None,
            }],
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let deserialized: GitResultDto = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(result, deserialized);
    }

    #[test]
    fn git_result_dto_unit_variants_roundtrip() {
        let variants = [
            GitResultDto::Stage,
            GitResultDto::Unstage,
            GitResultDto::StageLines,
            GitResultDto::Push,
            GitResultDto::Pull,
            GitResultDto::Fetch,
            GitResultDto::CheckoutBranch,
            GitResultDto::CreateBranch,
            GitResultDto::DeleteBranch,
            GitResultDto::StashPush,
            GitResultDto::StashPop,
            GitResultDto::StashDrop,
            GitResultDto::DiscardChanges,
        ];
        for variant in &variants {
            let json = serde_json::to_string(variant).expect("serialize");
            let deserialized: GitResultDto = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(variant, &deserialized);
        }
    }

    #[test]
    fn git_result_dto_branches_roundtrip() {
        let result = GitResultDto::Branches {
            branches: vec![BranchInfoDto {
                name: "main".to_string(),
                is_current: true,
                upstream: Some("origin/main".to_string()),
                ahead: 2,
                behind: 0,
            }],
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let deserialized: GitResultDto = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(result, deserialized);
    }

    #[test]
    fn git_result_dto_log_roundtrip() {
        let result = GitResultDto::Log {
            commits: vec![CommitInfoDto {
                hash: "abc123def456".to_string(),
                short_hash: "abc123d".to_string(),
                author_name: "Alice".to_string(),
                author_email: "alice@example.com".to_string(),
                date: "2026-03-10T01:00:00Z".to_string(),
                message: "feat: add feature".to_string(),
                parent_hashes: vec!["1111111".to_string()],
            }],
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let deserialized: GitResultDto = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(result, deserialized);
    }

    #[test]
    fn git_result_dto_blame_roundtrip() {
        let result = GitResultDto::Blame {
            entries: vec![BlameEntryDto {
                hash: "deadbeef".to_string(),
                author: "Alice".to_string(),
                date: "2026-03-10".to_string(),
                line_start: 1,
                line_count: 5,
                content: "fn main() {}".to_string(),
            }],
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let deserialized: GitResultDto = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(result, deserialized);
    }

    #[test]
    fn git_result_dto_stash_list_roundtrip() {
        let result = GitResultDto::StashList {
            entries: vec![StashEntryDto {
                index: 0,
                message: "WIP on main".to_string(),
                date: "2026-03-10".to_string(),
            }],
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let deserialized: GitResultDto = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(result, deserialized);
    }

    #[test]
    fn git_result_dto_repo_state_roundtrip() {
        let result = GitResultDto::RepoState {
            state: GitRepoStateDto {
                repo_path: "/home/user/project".to_string(),
                head_branch: Some("main".to_string()),
                is_detached: false,
                status_clean: true,
            },
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let deserialized: GitResultDto = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(result, deserialized);
    }

    #[test]
    fn git_result_dto_merge_branch_roundtrip() {
        let result = GitResultDto::MergeBranch {
            message: "Merge branch 'feature' into main".to_string(),
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let deserialized: GitResultDto = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(result, deserialized);
    }

    #[test]
    fn git_result_dto_commit_roundtrip() {
        let result = GitResultDto::Commit {
            hash: "abc1234".to_string(),
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let deserialized: GitResultDto = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(result, deserialized);
    }

    #[test]
    fn file_status_dto_renamed_with_old_path() {
        let status = FileStatusDto {
            path: "new_name.rs".to_string(),
            old_path: Some("old_name.rs".to_string()),
            index_status: FileStatusKindDto::Renamed,
            worktree_status: FileStatusKindDto::Renamed,
        };
        let json = serde_json::to_string(&status).expect("serialize");
        let deserialized: FileStatusDto = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(status, deserialized);
    }

    #[test]
    fn all_git_command_variants_are_distinct() {
        let commands = vec![
            GitCommandDto::Status {
                pane_id: "p".to_string(),
            },
            GitCommandDto::Diff {
                pane_id: "p".to_string(),
                path: None,
                staged: false,
            },
            GitCommandDto::Stage {
                pane_id: "p".to_string(),
                paths: vec![],
            },
            GitCommandDto::Unstage {
                pane_id: "p".to_string(),
                paths: vec![],
            },
            GitCommandDto::StageLines {
                pane_id: "p".to_string(),
                path: "f".to_string(),
                line_ranges: vec![],
            },
            GitCommandDto::Commit {
                pane_id: "p".to_string(),
                message: "m".to_string(),
            },
            GitCommandDto::Push {
                pane_id: "p".to_string(),
                remote: None,
                branch: None,
            },
            GitCommandDto::Pull {
                pane_id: "p".to_string(),
                remote: None,
                branch: None,
            },
            GitCommandDto::Fetch {
                pane_id: "p".to_string(),
                remote: None,
            },
            GitCommandDto::Branches {
                pane_id: "p".to_string(),
            },
            GitCommandDto::CheckoutBranch {
                pane_id: "p".to_string(),
                name: "b".to_string(),
            },
            GitCommandDto::CreateBranch {
                pane_id: "p".to_string(),
                name: "b".to_string(),
                start_point: None,
            },
            GitCommandDto::DeleteBranch {
                pane_id: "p".to_string(),
                name: "b".to_string(),
                force: false,
            },
            GitCommandDto::MergeBranch {
                pane_id: "p".to_string(),
                name: "b".to_string(),
            },
            GitCommandDto::Log {
                pane_id: "p".to_string(),
                max_count: None,
                path: None,
            },
            GitCommandDto::Blame {
                pane_id: "p".to_string(),
                path: "f".to_string(),
            },
            GitCommandDto::StashPush {
                pane_id: "p".to_string(),
                message: None,
            },
            GitCommandDto::StashPop {
                pane_id: "p".to_string(),
                index: None,
            },
            GitCommandDto::StashList {
                pane_id: "p".to_string(),
            },
            GitCommandDto::StashDrop {
                pane_id: "p".to_string(),
                index: 0,
            },
            GitCommandDto::DiscardChanges {
                pane_id: "p".to_string(),
                paths: vec![],
            },
            GitCommandDto::RepoState {
                pane_id: "p".to_string(),
            },
        ];
        // Each variant serializes to a unique "kind" tag
        let kinds: Vec<String> = commands
            .iter()
            .map(|c| {
                let json = serde_json::to_string(c).expect("serialize");
                let val: serde_json::Value = serde_json::from_str(&json).expect("parse");
                val["kind"].as_str().expect("kind tag").to_string()
            })
            .collect();
        let unique: std::collections::HashSet<&String> = kinds.iter().collect();
        assert_eq!(
            kinds.len(),
            unique.len(),
            "All command variants must have unique kind tags"
        );
    }
}
