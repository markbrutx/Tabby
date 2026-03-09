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

## [2026-03-09 07:47] - DDD-003: Switch domain crates from tabby-contracts to tabby-kernel
Thread:
Run: 20260309-073631-33953 (iteration 3)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260309-073631-33953-iter-3.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260309-073631-33953-iter-3.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: e34887d refactor: switch domain crates from tabby-contracts to tabby-kernel (DDD-003)
- Post-commit status: clean
- Verification:
  - Command: `bun run lint` -> PASS
  - Command: `bun run typecheck` -> PASS
  - Command: `bun run test` -> PASS (19 files, 203 tests)
  - Command: `cargo fmt --all --check` -> PASS
  - Command: `cargo clippy --workspace --all-targets --all-features -- -D warnings` -> PASS
  - Command: `cargo test --workspace` -> PASS (301 tests: 172 app-lib + 0 contracts + 29 kernel + 11 runtime + 35 settings + 53 workspace)
- Files changed:
  - src-tauri/crates/tabby-workspace/Cargo.toml (tabby-contracts → tabby-kernel)
  - src-tauri/crates/tabby-runtime/Cargo.toml (tabby-contracts → tabby-kernel)
  - src-tauri/crates/tabby-settings/Cargo.toml (tabby-contracts → tabby-kernel)
  - src-tauri/crates/tabby-workspace/src/ids.rs (use tabby_kernel instead of tabby_contracts)
  - src-tauri/crates/tabby-runtime/src/lib.rs (use tabby_kernel instead of tabby_contracts, both production and test code)
  - src-tauri/crates/tabby-settings/src/value_objects.rs (use tabby_kernel instead of tabby_contracts)
  - src-tauri/crates/tabby-settings/src/lib.rs (ValueObjectError import from tabby_kernel)
  - src-tauri/Cargo.lock (updated)
- All 3 domain crates now depend on tabby-kernel instead of tabby-contracts. Domain code has zero IPC/transport dependency. The re-exports in tabby-contracts keep application-layer code working unchanged.
- **Learnings for future iterations:**
  - tabby-settings had a hidden `From<tabby_contracts::ValueObjectError>` impl in lib.rs beyond the obvious value_objects.rs import — always grep broadly for the old crate name
  - Pure dependency swap with import updates — minimal risk, straightforward refactor
---

## [2026-03-09 07:49] - DDD-004: Arch test: domain crates must not depend on tabby-contracts
Thread:
Run: 20260309-073631-33953 (iteration 4)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260309-073631-33953-iter-4.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260309-073631-33953-iter-4.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: a2fde78 test: add arch test asserting domain crates do not depend on tabby-contracts (DDD-004)
- Post-commit status: clean
- Verification:
  - Command: `bun run lint` -> PASS
  - Command: `bun run typecheck` -> PASS
  - Command: `bun run test` -> PASS (19 files, 203 tests)
  - Command: `cargo fmt --all --check` -> PASS
  - Command: `cargo clippy --workspace --all-targets --all-features -- -D warnings` -> PASS
  - Command: `cargo test --workspace` -> PASS (303 tests: 172 app-lib + 2 arch + 0 contracts + 29 kernel + 11 runtime + 35 settings + 53 workspace)
- Files changed:
  - src-tauri/tests/arch_domain_crate_dependencies.rs (created)
- Created Rust integration test that parses Cargo.toml files for all 3 domain crates and asserts none list tabby-contracts in [dependencies]. Includes helper unit test for the TOML parser. Test lives in src-tauri/tests/ as an integration test.
- **Learnings for future iterations:**
  - Integration tests in src-tauri/tests/ are automatically discovered by cargo test
  - The tests/ directory did not exist before — had to create it
  - rustfmt reformatted multi-line unwrap_or_else closures — always run fmt before committing
---

## [2026-03-09 07:51] - DDD-005: Route browser surface commands through RuntimeApplicationService
Thread:
Run: 20260309-073631-33953 (iteration 5)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260309-073631-33953-iter-5.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260309-073631-33953-iter-5.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: 5bfcd24 refactor: route browser surface commands through RuntimeApplicationService (DDD-005)
- Post-commit status: clean
- Verification:
  - Command: `bun run lint` -> PASS
  - Command: `bun run typecheck` -> PASS
  - Command: `bun run test` -> PASS (19 files, 203 tests)
  - Command: `cargo fmt --all --check` -> PASS
  - Command: `cargo clippy --workspace --all-targets --all-features -- -D warnings` -> PASS
  - Command: `cargo test --workspace` -> PASS (303 tests: 172 app-lib + 2 arch + 0 contracts + 29 kernel + 11 runtime + 35 settings + 53 workspace)
- Files changed:
  - src-tauri/src/commands/shell.rs (dispatch_browser_surface_command now routes through AppShell state instead of direct infra call)
  - src-tauri/src/shell/mod.rs (added dispatch_browser_surface_command delegating to RuntimeApplicationService)
  - src-tauri/src/application/runtime_service.rs (added dispatch_browser_surface_command dispatching to BrowserSurfacePort)
  - src-tauri/src/application/ports.rs (removed #[allow(dead_code)] and stale doc comment from BrowserSurfacePort)
  - src-tauri/src/infrastructure/tauri_browser_surface_adapter.rs (removed #[allow(dead_code)] from bounds_dto helper)
- Browser surface commands now flow: Tauri command → AppShell → RuntimeApplicationService → BrowserSurfacePort. RuntimeApplicationService is the single owner of all runtime lifecycle operations (terminal and browser).
- **Learnings for future iterations:**
  - The TauriBrowserSurfaceAdapter already implements BrowserSurfacePort and gets the window from AppHandle internally — no need to pass window from the command handler
  - Removing #[allow(dead_code)] from a trait definition also activates warnings on unused items in implementations — check adapter too
  - The command handler signature changed from taking `window: tauri::Window` to `state: State<'_, Arc<AppShell>>` — Tauri injects both automatically
---

## [2026-03-09 07:54] - DDD-006: Wire TauriBrowserSurfaceAdapter to BrowserSurfacePort fully
Thread:
Run: 20260309-073631-33953 (iteration 6)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260309-073631-33953-iter-6.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260309-073631-33953-iter-6.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: de5a287 chore: update progress log for DDD-006 completion
- Post-commit status: clean
- Verification:
  - Command: `bun run lint` -> PASS
  - Command: `bun run typecheck` -> PASS
  - Command: `bun run test` -> PASS (19 files, 203 tests)
  - Command: `cargo fmt --all --check` -> PASS
  - Command: `cargo clippy --workspace --all-targets --all-features -- -D warnings` -> PASS
  - Command: `cargo test --workspace` -> PASS (303 tests: 172 app-lib + 2 arch + 0 contracts + 29 kernel + 11 runtime + 35 settings + 53 workspace)
- Files changed:
  - .ralph/progress.md (updated — progress entry only, no source code changes)
- All acceptance criteria were already satisfied by DDD-005:
  1. TauriBrowserSurfaceAdapter implements all 5 BrowserSurfacePort methods (ensure_surface, set_bounds, set_visible, close_surface, navigate)
  2. No stub/unimplemented methods remain
  3. RuntimeApplicationService uses BrowserSurfacePort for all browser operations (dispatch_browser_surface_command + stop_runtime)
  4. All quality gates pass (203 frontend + 303 backend tests)
- **Learnings for future iterations:**
  - DDD-005 fully wired the adapter as part of routing commands through RuntimeApplicationService — DDD-006 was a verification-only story
  - When a story's work is already done by a dependency, still run all quality gates to confirm before marking complete
---

## Codebase Patterns
- (add reusable patterns here)
