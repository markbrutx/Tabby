# Progress Log

## 2026-03-10 01:30 - GIT-001: Create tabby-git domain crate skeleton
Thread:
Run: 20260310-012951-93839 (iteration 1)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-012951-93839-iter-1.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-012951-93839-iter-1.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: cc50303 feat: add tabby-git domain crate skeleton (GIT-001)
- Post-commit status: clean
- Verification:
  - Command: bun run lint -> PASS
  - Command: bun run typecheck -> PASS
  - Command: bun run test -> PASS (203 tests)
  - Command: cargo fmt --all --check -> PASS
  - Command: cargo clippy --workspace --all-targets --all-features -- -D warnings -> PASS
  - Command: cargo test --workspace -> PASS (317 tests)
  - Command: cargo check --workspace -> PASS
- Files changed:
  - src-tauri/Cargo.toml (added tabby-git to workspace members)
  - src-tauri/Cargo.lock (auto-updated)
  - src-tauri/crates/tabby-git/Cargo.toml (new crate, depends only on tabby-kernel)
  - src-tauri/crates/tabby-git/src/lib.rs (module structure comments, no code yet)
- Created tabby-git domain crate skeleton with zero transport dependencies
- **Learnings for future iterations:**
  - Follow existing crate patterns (tabby-kernel as reference) for consistency
  - Arch tests in tests/arch_ddd_violations.rs automatically verify domain crates don't depend on tabby-contracts
---

## 2026-03-10 01:34 - GIT-002: Core value objects — BranchName, CommitHash, RemoteName, StashId
Thread:
Run: 20260310-012951-93839 (iteration 2)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-012951-93839-iter-2.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-012951-93839-iter-2.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: 5656e12 feat: add core value objects for tabby-git domain (GIT-002)
- Post-commit status: clean
- Verification:
  - Command: cargo test -p tabby-git -> PASS (28 tests)
  - Command: bun run lint -> PASS
  - Command: bun run typecheck -> PASS
  - Command: bun run test -> PASS (203 tests)
  - Command: cargo fmt --all --check -> PASS
  - Command: cargo clippy --workspace --all-targets --all-features -- -D warnings -> PASS
  - Command: cargo test --workspace -> PASS
- Files changed:
  - src-tauri/crates/tabby-git/src/lib.rs (added value_objects module and re-exports)
  - src-tauri/crates/tabby-git/src/value_objects.rs (new: 4 value objects + 28 tests)
- Implemented BranchName (non-empty, no spaces), CommitHash (4-40 hex, lowercase normalization), RemoteName (non-empty), StashId (usize newtype with stash@{N} display)
- **Learnings for future iterations:**
  - Run cargo fmt before fmt --check to avoid CI-style failures on first pass
  - CommitHash normalizes to lowercase for consistent equality comparison
  - Follow tabby-kernel patterns: try_new() + Display + AsRef<str> for string VOs
---

## 2026-03-10 01:37 - GIT-003: File status and diff domain types in tabby-git
Thread:
Run: 20260310-012951-93839 (iteration 3)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-012951-93839-iter-3.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-012951-93839-iter-3.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: b5fd351 feat: add file status and diff domain types for tabby-git (GIT-003)
- Post-commit status: clean
- Verification:
  - Command: cargo fmt --all --check -> PASS
  - Command: cargo clippy --workspace --all-targets --all-features -- -D warnings -> PASS
  - Command: cargo test -p tabby-git -> PASS (55 tests)
  - Command: cargo test --workspace -> PASS
  - Command: bun run lint -> PASS
  - Command: bun run typecheck -> PASS
  - Command: bun run test -> PASS (203 tests)
- Files changed:
  - src-tauri/crates/tabby-git/src/lib.rs (added diff and file_status modules + re-exports)
  - src-tauri/crates/tabby-git/src/file_status.rs (new: FileStatusKind enum, FileStatus struct + 8 tests)
  - src-tauri/crates/tabby-git/src/diff.rs (new: DiffLineKind, DiffLine, DiffHunk, DiffContent + 19 tests)
- Implemented all acceptance criteria:
  - FileStatusKind: Modified, Added, Deleted, Renamed, Copied, Untracked, Ignored, Conflicted
  - FileStatus: path, old_path, index_status, worktree_status with getters
  - DiffLineKind: Context, Addition, Deletion, HunkHeader
  - DiffLine: kind, old_line_no, new_line_no, content with getters
  - DiffHunk: old_start, old_count, new_start, new_count, header, lines with getters
  - DiffContent: file_path, old_path, hunks, is_binary, file_mode_change with getters
  - All types are Debug + Clone + PartialEq, no serde
  - 27 unit tests covering construction, field access, equality, and cloning
- **Learnings for future iterations:**
  - cargo needs `source ~/.zshrc` in this env, not just `export PATH`
  - Separate modules per concern (file_status.rs, diff.rs) keeps files small and focused
  - Copy derive only for small enums; structs with String fields use Clone only
---

## 2026-03-10 01:39 - GIT-004: Commit, branch, blame, and repo state types in tabby-git
Thread:
Run: 20260310-012951-93839 (iteration 4)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-012951-93839-iter-4.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-012951-93839-iter-4.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: cf37f44 feat: add commit, branch, blame, stash, and repo state types for tabby-git (GIT-004)
- Post-commit status: clean
- Verification:
  - Command: cargo fmt --all --check -> PASS
  - Command: cargo clippy --workspace --all-targets --all-features -- -D warnings -> PASS
  - Command: cargo test -p tabby-git -> PASS (80 tests)
  - Command: cargo test --workspace -> PASS
  - Command: bun run lint -> PASS
  - Command: bun run typecheck -> PASS
  - Command: bun run test -> PASS (203 tests)
- Files changed:
  - src-tauri/crates/tabby-git/src/lib.rs (added 5 new modules + re-exports)
  - src-tauri/crates/tabby-git/src/commit.rs (new: CommitInfo struct + 6 tests)
  - src-tauri/crates/tabby-git/src/branch.rs (new: BranchInfo struct + 6 tests)
  - src-tauri/crates/tabby-git/src/blame.rs (new: BlameEntry struct + 4 tests)
  - src-tauri/crates/tabby-git/src/stash.rs (new: StashEntry struct + 4 tests)
  - src-tauri/crates/tabby-git/src/repository_state.rs (new: GitRepositoryState struct + 5 tests)
- Implemented all acceptance criteria:
  - CommitInfo: hash, short_hash, author_name, author_email, date, message, parent_hashes
  - BranchInfo: name, is_current, upstream, ahead, behind
  - BlameEntry: hash, author, date, line_start, line_count, content
  - StashEntry: index (StashId), message, date
  - GitRepositoryState: repo_path (WorkingDirectory), head_branch (Option<BranchName>), is_detached, status_clean
  - All types Debug + Clone + PartialEq with unit tests
  - 25 new tests (80 total in tabby-git)
- **Learnings for future iterations:**
  - One module per domain type keeps files small and focused (~100 lines each)
  - Reuse value objects from value_objects.rs and tabby-kernel (WorkingDirectory) as field types
  - No validation needed in struct constructors when fields use already-validated value objects
---

## 2026-03-10 01:46 - GIT-005: Add GitPaneSpec and PaneSpec::Git to tabby-workspace
Thread:
Run: 20260310-012951-93839 (iteration 5)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-012951-93839-iter-5.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-012951-93839-iter-5.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: 0288996 feat: add GitPaneSpec and PaneSpec::Git to tabby-workspace (GIT-005)
- Post-commit status: clean
- Verification:
  - Command: cargo fmt --all --check -> PASS
  - Command: cargo clippy --workspace --all-targets --all-features -- -D warnings -> PASS
  - Command: cargo test --workspace -> PASS (402 tests)
  - Command: bun run lint -> PASS
  - Command: bun run typecheck -> PASS
  - Command: bun run test -> PASS (203 tests)
- Files changed:
  - src-tauri/crates/tabby-workspace/src/lib.rs (GitPaneSpec struct, PaneSpec::Git variant, updated match arms)
  - src-tauri/crates/tabby-workspace/src/content.rs (PaneContentDefinition::Git variant, git() constructor, updated match arms)
  - src-tauri/crates/tabby-contracts/src/lib.rs (PaneSpecDto::Git variant)
  - src-tauri/src/mapping/dto_mappers.rs (Git handling in all mapper functions)
  - src-tauri/src/application/runtime_service.rs (Git pane returns early from start_runtime — no runtime yet)
  - src-tauri/src/application/runtime_coordinator.rs (Git arm in test match blocks)
  - src-tauri/src/application/runtime_lifecycle_tests.rs (Git arm in test helper)
  - src/contracts/tauri-bindings.ts (PaneSpecDto Git variant)
  - src/features/workspace/domain/models.ts (GitPaneSpec interface, PaneSpec union)
  - src/features/workspace/application/snapshot-mappers.ts (Git handling in mappers)
  - src/features/workspace/model/workspaceSnapshot.ts (Git pane kind + snapshot builder)
- Implemented all acceptance criteria:
  - GitPaneSpec struct with working_directory: String in tabby-workspace
  - PaneSpec::Git(GitPaneSpec) variant added
  - All match arms updated for exhaustiveness across 7 Rust files
  - spec_from_content handles Git content definition
  - PaneContentDefinition::Git variant added with constructor and field access
  - Frontend types and mappers updated for full-stack consistency
  - All 402 Rust tests + 203 frontend tests pass
- **Learnings for future iterations:**
  - Adding a PaneSpec variant ripples across both Rust and TypeScript — need to update contracts, domain models, mappers, and snapshot builders
  - Git panes have no runtime process yet; return early from start_runtime to avoid registering a runtime
  - Use wildcard `other => panic!()` in test match arms to be future-proof when new variants are added
  - Auto-generated tauri-bindings.ts must be manually updated in sync until specta regeneration runs
---
