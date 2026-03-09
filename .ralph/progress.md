# Progress Log

## [2026-03-09 07:38] - DDD-001: Document 7 DDD violations with exact file:line references
Thread:
Run: 20260309-073631-33953 (iteration 1)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260309-073631-33953-iter-1.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260309-073631-33953-iter-1.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: 0fa14f1 docs: document 7 DDD violations with exact file:line references (DDD-001)
- Post-commit status: clean
- Verification:
  - Command: `bun run lint` -> PASS
  - Command: `bun run typecheck` -> PASS
  - Command: `bun run test` -> PASS (19 files, 203 tests)
  - Command: `cargo fmt --all --check` -> PASS
  - Command: `cargo clippy --workspace --all-targets --all-features -- -D warnings` -> PASS
  - Command: `cargo test --workspace` -> PASS (300 tests: 172 app-lib + 29 contracts + 11 runtime + 35 settings + 53 workspace)
- Files changed:
  - .ralph/ddd-v3-baseline.md (created)
- Created baseline documentation of all 7 DDD violations with exact file:line references, severity ratings, and remediation story mappings. Recorded quality gate results: 503 total tests (203 frontend + 300 backend), all passing.
- **Learnings for future iterations:**
  - .ralph/ is gitignored; files need `git add -f` to track
  - progress.md is already force-tracked
  - All quality gates are green; no pre-existing failures
  - 7 violations map cleanly to stories DDD-002 through DDD-018
---

## [2026-03-09 07:45] - DDD-002: Create tabby-kernel crate with shared kernel value objects
Thread:
Run: 20260309-073631-33953 (iteration 2)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260309-073631-33953-iter-2.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260309-073631-33953-iter-2.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: 15993cc refactor: create tabby-kernel crate with shared kernel value objects (DDD-002)
- Post-commit status: clean
- Verification:
  - Command: `bun run lint` -> PASS
  - Command: `bun run typecheck` -> PASS
  - Command: `bun run test` -> PASS (19 files, 203 tests)
  - Command: `cargo fmt --all --check` -> PASS
  - Command: `cargo clippy --workspace --all-targets --all-features -- -D warnings` -> PASS
  - Command: `cargo test --workspace` -> PASS (301 tests: 172 app-lib + 0 contracts + 29 kernel + 11 runtime + 35 settings + 53 workspace)
- Files changed:
  - src-tauri/crates/tabby-kernel/Cargo.toml (created)
  - src-tauri/crates/tabby-kernel/src/lib.rs (created)
  - src-tauri/crates/tabby-kernel/src/value_objects.rs (created)
  - src-tauri/Cargo.toml (added tabby-kernel to workspace members)
  - src-tauri/Cargo.lock (updated)
  - src-tauri/crates/tabby-contracts/Cargo.toml (added tabby-kernel dependency)
  - src-tauri/crates/tabby-contracts/src/value_objects.rs (replaced with re-exports from tabby-kernel)
- Created tabby-kernel crate with zero transport dependencies containing PaneId, TabId, BrowserUrl, WorkingDirectory, ValueObjectError, and id_newtype! macro. tabby-contracts now re-exports all symbols from tabby-kernel for backward compatibility. All 29 VO tests now run in tabby-kernel; contracts tests dropped to 0 (expected — tests moved to kernel).
- **Learnings for future iterations:**
  - `#[macro_export]` macros are exported at the crate root, not the module — cannot re-export via `pub use module::macro_name`
  - The id_newtype! macro is only used internally in tabby-kernel; no external crate invokes it directly
  - Re-exports in tabby-contracts preserve full backward compatibility — all 301 Rust tests pass unchanged
---

## Codebase Patterns
- (add reusable patterns here)
