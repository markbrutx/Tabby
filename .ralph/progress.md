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
