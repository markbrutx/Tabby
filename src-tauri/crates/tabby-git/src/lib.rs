// tabby-git — Git bounded context domain crate.
//
// Module structure:
// - value_objects: Git-specific value objects (branch names, commit hashes, etc.)
// - entities: Git domain entities (repository state, diff, file status, etc.)
// - events: Domain events for the Git context
//
// Dependencies: tabby-kernel only. No serde, specta, or tauri.

pub mod blame;
pub mod branch;
pub mod commit;
pub mod diff;
pub mod file_status;
pub mod repository_state;
pub mod stash;
pub mod value_objects;

pub use blame::BlameEntry;
pub use branch::BranchInfo;
pub use commit::CommitInfo;
pub use diff::{DiffContent, DiffHunk, DiffLine, DiffLineKind};
pub use file_status::{FileStatus, FileStatusKind};
pub use repository_state::GitRepositoryState;
pub use stash::StashEntry;
pub use value_objects::{BranchName, CommitHash, RemoteName, StashId};
