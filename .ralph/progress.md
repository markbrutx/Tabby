# Progress Log

## [2026-03-10 00:45] - DDD-014: LayoutPreset enum in tabby-kernel replaces stringly-typed default_layout
Thread:
Run: 20260310-000917-71928 (iteration 8)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-000917-71928-iter-8.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-000917-71928-iter-8.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: af4b004 refactor: LayoutPreset enum in tabby-kernel replaces stringly-typed default_layout (DDD-014)
- Post-commit status: clean
- Verification:
  - Command: bun run lint -> PASS
  - Command: bun run typecheck -> PASS
  - Command: bun run test -> PASS (203 tests)
  - Command: cargo fmt --all --check -> PASS
  - Command: cargo clippy --workspace --all-targets --all-features -- -D warnings -> PASS
  - Command: cargo test --workspace -> PASS (314 tests: 172 app-lib + 2 arch + 5 browser-cmd + 35 kernel + 11 runtime + 36 settings + 53 workspace)
- Files changed:
  - src-tauri/crates/tabby-kernel/src/lib.rs (export LayoutPreset)
  - src-tauri/crates/tabby-kernel/src/value_objects.rs (added LayoutPreset enum with parse, as_str, pane_count, Display, Default + 7 tests)
  - src-tauri/crates/tabby-settings/src/lib.rs (UserPreferences.default_layout: String → LayoutPreset, removed DEFAULT_LAYOUT_PRESET, is_known_layout_preset, string validation)
  - src-tauri/crates/tabby-settings/src/persistence.rs (String↔LayoutPreset conversion at persistence boundary, backward compat for unknown values)
  - src-tauri/crates/tabby-workspace/src/layout.rs (re-export LayoutPreset from tabby-kernel instead of defining own)
  - src-tauri/src/application/bootstrap_service.rs (simplified — no more LayoutPreset::parse on preferences)
  - src-tauri/src/mapping/dto_mappers.rs (direct enum mapping, removed layout_preset_to_string helper)
  - src-tauri/src/shell/mod.rs (simplified resolve_default_layout — direct field access)
- What was implemented:
  - Created LayoutPreset enum in tabby-kernel with all 5 variants (OneByOne, OneByTwo, TwoByTwo, TwoByThree, ThreeByThree)
  - Changed UserPreferences.default_layout from String to LayoutPreset — compile-time validation
  - Persistence layer (PersistedPreferences) keeps String on disk for backward compatibility, converts via parse/as_str
  - Unknown persisted layout values gracefully fall back to LayoutPreset::default() (OneByOne)
  - Removed all stringly-typed layout validation: is_known_layout_preset(), DEFAULT_LAYOUT_PRESET constant, LayoutPreset::parse calls at usage sites
  - tabby-workspace re-exports LayoutPreset from tabby-kernel (canonical definition in shared kernel)
- **Learnings for future iterations:**
  - When adding Default derive to an enum, use `#[default]` attribute on the variant — clippy rejects manual Default impl for derivable cases
  - clippy::unwrap_or_default catches `.unwrap_or(T::default())` patterns — use `.unwrap_or_default()`
  - Persistence boundary is the right place for String↔enum conversion — domain stays typed, disk format stays stable
  - LayoutPreset::parse now returns ValueObjectError (from tabby-kernel) instead of LayoutError (from tabby-workspace), but .to_string() works for both at error mapping sites
---

## [2026-03-10 00:35] - DDD-013: RuntimeStore.loadBootstrap accepts RuntimeReadModel[] not DTO
Thread:
Run: 20260310-000917-71928 (iteration 7)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-000917-71928-iter-7.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-000917-71928-iter-7.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: fa4511e refactor: RuntimeStore.loadBootstrap accepts RuntimeReadModel[] not DTO (DDD-013)
- Post-commit status: clean
- Verification:
  - Command: bun run lint -> PASS
  - Command: bun run typecheck -> PASS
  - Command: bun run test -> PASS (203 tests)
  - Command: cargo fmt --all --check -> PASS
  - Command: cargo clippy --workspace --all-targets --all-features -- -D warnings -> PASS
  - Command: cargo test --workspace -> PASS (172 + 2 + 5 + 29 + 11 + 35 + 53 tests)
- Files changed:
  - src/features/runtime/application/store.ts
  - src/features/runtime/application/store.test.ts
  - src/app-shell/AppBootstrapCoordinator.ts
  - src/app-shell/AppBootstrapCoordinator.test.ts
- Changed RuntimeStore.loadBootstrap to accept RuntimeReadModel[] instead of PaneRuntimeView[] (DTO)
- Moved DTO→ReadModel mapping (mapRuntimeFromDto) to AppBootstrapCoordinator.initialize()
- Removed PaneRuntimeView import from runtime store; removed WorkspaceBootstrapView import from coordinator
- Updated BootstrapableRuntimeStore interface to use RuntimeReadModel[]
- Updated tests to use RuntimeReadModel for loadBootstrap calls; kept PaneRuntimeView for listener tests (correct: listeners receive live DTOs)
- **Learnings for future iterations:**
  - All three stores (Workspace, Settings, Runtime) now follow the same ACL pattern: coordinator maps DTOs, stores receive read models
  - The listener callback (listenStatusChanged) still correctly uses mapRuntimeFromDto since it receives live DTOs from the transport layer — this is the right boundary
  - Test that asserted `stored !== dto` needed updating since store now receives read models directly (no intermediate mapping creates a new object)
---

## [2026-03-10 00:25] - DDD-011: WorkspaceStore.loadBootstrap accepts WorkspaceReadModel not DTO
Thread:
Run: 20260310-000917-71928 (iteration 5)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-000917-71928-iter-5.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-000917-71928-iter-5.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: 3f24e02 refactor: WorkspaceStore.loadBootstrap accepts WorkspaceReadModel not DTO (DDD-011)
- Post-commit status: clean
- Verification:
  - Command: bun run lint -> PASS
  - Command: bun run typecheck -> PASS
  - Command: bun run test -> PASS (203 tests)
  - Command: cargo fmt --all --check -> PASS
  - Command: cargo clippy --workspace --all-targets --all-features -- -D warnings -> PASS
  - Command: cargo test --workspace -> PASS (172 + 2 + 5 + 29 + 11 + 35 + 53 tests)
- Files changed:
  - src/app-shell/AppBootstrapCoordinator.ts
  - src/app-shell/AppBootstrapCoordinator.test.ts
  - src/features/workspace/application/store.ts
  - src/features/workspace/application/store.test.ts
- Changed WorkspaceStore.loadBootstrap parameter from WorkspaceBootstrapView (DTO) to WorkspaceReadModel (internal read model)
- Moved DTO→ReadModel mapping (mapWorkspaceFromDto) into AppBootstrapCoordinator.initialize()
- Removed WorkspaceBootstrapView import from workspace store
- Updated BootstrapableWorkspaceStore interface in coordinator to match new signature
- Updated all tests to pass WorkspaceReadModel instead of WorkspaceBootstrapView
- **Learnings for future iterations:**
  - The store test helper makeBootstrapPayload needed complete replacement with makeWorkspaceReadModel using camelCase domain model fields
  - The coordinator test assertion for loadWorkspace needed updating to expect the mapped read model rather than the raw DTO payload
  - The listenProjectionUpdated callback in the store still correctly maps DTOs internally since projections arrive as DTOs from the wire
---

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

## [2026-03-10 00:12] - DDD-007: Test: browser commands dispatch through RuntimeApplicationService
Thread:
Run: 20260310-000917-71928 (iteration 1)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-000917-71928-iter-1.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-000917-71928-iter-1.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: 3fe8f71 test: add integration tests for browser commands through RuntimeApplicationService (DDD-007)
- Post-commit status: clean
- Verification:
  - Command: `cargo fmt --all --check` -> PASS
  - Command: `cargo clippy --workspace --all-targets --all-features -- -D warnings` -> PASS
  - Command: `cargo test --workspace` -> PASS (307 tests: 172 app-lib + 2 arch + 5 browser-cmd + 0 contracts + 29 kernel + 11 runtime + 35 settings + 53 workspace)
  - Command: `bun run lint` -> PASS
  - Command: `bun run typecheck` -> PASS
  - Command: `bun run test` -> PASS (19 files, 203 tests)
- Files changed:
  - src-tauri/tests/browser_commands_through_runtime_service.rs (created — 5 integration tests)
  - src-tauri/src/lib.rs (made application and shell modules pub for integration test access)
  - src-tauri/src/application/workspace_service.rs (added Default impl to fix pre-existing clippy warning)
- Created 5 integration tests with mock BrowserSurfacePort verifying the single-owner invariant: all 4 BrowserSurfaceCommandDto variants (Ensure, SetBounds, SetVisible, Close) dispatch through RuntimeApplicationService to BrowserSurfacePort. Sequential multi-command test verifies ordering. Full path confirmed: commands/shell.rs → AppShell → RuntimeApplicationService → BrowserSurfacePort.
- **Learnings for future iterations:**
  - Integration tests in src-tauri/tests/ need pub visibility on application and shell modules in lib.rs
  - Pre-existing clippy warnings may surface when recompiling — fix them as part of the story
  - The lib crate is named `tabby_app_lib` (not `tabby`) in Cargo.toml [lib] section
---

## [2026-03-10 00:23] - DDD-010: ADR: terminal output hot-path exemption
Thread:
Run: 20260310-000917-71928 (iteration 4)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-000917-71928-iter-4.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-000917-71928-iter-4.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: cf09663 docs: add ADR-001 for terminal output hot-path exemption (DDD-010)
- Post-commit status: clean
- Verification:
  - Command: `bun run lint` -> PASS
  - Command: `bun run typecheck` -> PASS
  - Command: `bun run test` -> PASS (19 files, 203 tests)
  - Command: `cargo fmt --all --check` -> PASS
  - Command: `cargo clippy --workspace --all-targets --all-features -- -D warnings` -> PASS
  - Command: `cargo test --workspace` -> PASS (307 tests: 172 app-lib + 2 arch + 5 browser-cmd + 29 kernel + 11 runtime + 35 settings + 53 workspace)
- Files changed:
  - docs/adr/001-terminal-output-hot-path.md (created — ADR document)
  - src-tauri/src/shell/pty.rs (added doc comment on emit call referencing ADR)
  - src-tauri/src/application/runtime_observation_receiver.rs (added doc comment on trait method referencing ADR)
  - src-tauri/src/application/runtime_service.rs (updated comment on no-op implementation referencing ADR)
- Created ADR-001 explaining why terminal output bypasses RuntimeObservationReceiver: high-frequency byte stream, no domain state change, latency sensitivity. Documented that on_terminal_output_received is reserved for future OSC sequence detection. Added doc comments on pty.rs emit call, trait definition, and service implementation all referencing the ADR.
- **Learnings for future iterations:**
  - ADR documents belong in docs/adr/ directory (standard convention)
  - Documentation-only stories still need all quality gates run
  - The docs/ directory did not exist before — had to create it
---

## Codebase Patterns
- (add reusable patterns here)

## [2026-03-10 00:14] - DDD-008: Remove SettingsApplicationService from runtime_service observe_terminal_cwd
Thread:
Run: 20260310-000917-71928 (iteration 2)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-000917-71928-iter-2.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-000917-71928-iter-2.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: 1eea5c1 refactor: remove SettingsApplicationService from runtime_service observe_terminal_cwd (DDD-008)
- Post-commit status: clean
- Verification:
  - Command: bun run lint -> PASS
  - Command: bun run typecheck -> PASS
  - Command: bun run test -> PASS (203 tests)
  - Command: cargo fmt --all --check -> PASS
  - Command: cargo clippy --workspace --all-targets --all-features -- -D warnings -> PASS
  - Command: cargo test --workspace -> PASS (172 tests)
- Files changed:
  - src-tauri/src/application/runtime_service.rs
  - src-tauri/src/application/runtime_integration_tests.rs
  - src-tauri/src/shell/mod.rs
- What was implemented:
  - Removed `&SettingsApplicationService` parameter from `observe_terminal_cwd` in RuntimeApplicationService
  - Removed `SettingsApplicationService` import from runtime_service.rs
  - Moved settings persistence (last_working_directory) to AppShell's `dispatch_runtime_command` coordinator
  - Updated integration test to verify runtime_service does NOT touch settings (cross-context decoupling)
  - Removed dead code: `last_saved_preferences` helper and `last_saved` field from test mock
- **Learnings for future iterations:**
  - When removing cross-context parameters, move the side effect to the coordinator (AppShell) to preserve behavior
  - Removing test helpers that were specific to old behavior may cascade to removing mock struct fields — check clippy for dead_code warnings
  - All 4 acceptance criteria met: param removed, AppShell handles persistence, Runtime only updates its own state, all quality gates pass
---

## [2026-03-10 00:19] - DDD-009: ProjectionPublisherPort accepts domain type instead of DTO
Thread:
Run: 20260310-000917-71928 (iteration 3)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-000917-71928-iter-3.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-000917-71928-iter-3.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: c073159 refactor: ProjectionPublisherPort accepts &WorkspaceSession instead of &WorkspaceView (DDD-009)
- Post-commit status: clean
- Verification:
  - Command: cargo fmt --all --check -> PASS
  - Command: cargo clippy --workspace --all-targets --all-features -- -D warnings -> PASS
  - Command: cargo test --workspace -> PASS (172 lib + 7 integration + 29 kernel + 11 runtime + 35 settings + 53 workspace = 307 tests)
  - Command: bun run lint -> PASS
  - Command: bun run typecheck -> PASS
  - Command: bun run test -> PASS (203 tests)
- Files changed:
  - src-tauri/src/application/ports.rs
  - src-tauri/src/infrastructure/tauri_projection_publisher.rs
  - src-tauri/src/shell/mod.rs
  - src-tauri/src/application/runtime_service.rs
  - src-tauri/src/application/runtime_integration_tests.rs
  - src-tauri/src/application/command_dispatch_integration_tests.rs
  - src-tauri/tests/browser_commands_through_runtime_service.rs
- Changed `publish_workspace_projection` in `ProjectionPublisherPort` to accept `&WorkspaceSession` (domain type) instead of `&WorkspaceView` (DTO)
- `TauriProjectionPublisher` now maps domain→DTO internally via `dto_mappers::workspace_view_from_session`
- `AppShell::dispatch_workspace_command` passes session directly to publisher, then maps to view for return value
- All mock implementations in 4 test files updated to use `&WorkspaceSession`
- Removed unused `WorkspaceView` import from browser integration test
- **Learnings for future iterations:**
  - Port traits should accept domain types; infrastructure adapters own the domain→DTO mapping
  - When changing a trait signature, grep for all mock implementations across test files (unit tests, integration tests, and external test crates)
  - `cargo fmt` catches formatting issues from multi-line → single-line struct construction after refactoring
---

## [2026-03-10 00:30] - DDD-012: SettingsStore.loadBootstrap accepts SettingsReadModel not DTO
Thread:
Run: 20260310-000917-71928 (iteration 6)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-000917-71928-iter-6.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260310-000917-71928-iter-6.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: 100c5f9 refactor: SettingsStore.loadBootstrap accepts SettingsReadModel not DTO (DDD-012)
- Post-commit status: clean
- Verification:
  - Command: bun run lint -> PASS
  - Command: bun run typecheck -> PASS
  - Command: bun run test -> PASS (203 tests)
  - Command: cargo fmt --all --check -> PASS
  - Command: cargo clippy --workspace --all-targets --all-features -- -D warnings -> PASS
  - Command: cargo test --workspace -> PASS (172+ tests)
- Files changed:
  - src/features/settings/application/store.ts
  - src/features/settings/application/store.test.ts
  - src/app-shell/AppBootstrapCoordinator.ts
  - src/app-shell/AppBootstrapCoordinator.test.ts
- What was implemented:
  - Changed SettingsStore.loadBootstrap signature from (SettingsView, ProfileDTO[]) to (SettingsReadModel, ProfileReadModel[])
  - loadBootstrap now sets read models directly instead of mapping from DTOs
  - Moved DTO→ReadModel mapping (mapSettingsFromDto, mapProfileFromDto) into AppBootstrapCoordinator.initialize()
  - Updated BootstrapableSettingsStore interface to use domain read model types
  - Updated all tests to pass read models instead of DTOs to loadBootstrap
- **Learnings for future iterations:**
  - SettingsView and SettingsReadModel are structurally identical (same field names/types), so TypeScript structural typing allows them interchangeably. The change is about enforcing the correct semantic boundary.
  - The store still needs DTO imports for runtime event listeners and dispatch responses — this is correct transport-boundary usage, not a violation.
  - Pattern from DDD-011 (WorkspaceStore) was directly applicable here.
---
