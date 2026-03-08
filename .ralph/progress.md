# Progress Log

## Codebase Patterns
- (add reusable patterns here)

---

## 2026-03-08T22:00 - US-001: Document verification baseline and current architecture state
Thread:
Run: 20260308-215923-84117 (iteration 1)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260308-215923-84117-iter-1.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260308-215923-84117-iter-1.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: none — all output is in gitignored `.ralph/` directory; zero source code modifications as required by story
- Post-commit status: clean
- Verification:
  - Command: `bun run lint` -> PASS
  - Command: `bun run typecheck` -> PASS
  - Command: `bun run test` -> PASS (162 tests)
  - Command: `cargo fmt --all --check` -> PASS
  - Command: `cargo clippy --workspace --all-targets --all-features -- -D warnings` -> PASS
  - Command: `cargo test --workspace` -> PASS (160 tests)
  - Command: `bun run test:e2e` -> PASS (15 tests)
- Files changed:
  - .ralph/baseline.md (created — gitignored)
- What was implemented:
  - Created comprehensive baseline document with quality gate results, test counts (337 total), full module structure, 7 transitional violations, 8-area architecture scorecard (50/80), bounded context summary, and dependency direction analysis
- **Learnings for future iterations:**
  - All quality gates currently green — clean starting point for refactor
  - 7 violations identified across 5 categories specified by PRD plus 2 additional (app layer Tauri imports, ghost events)
  - Domain crate isolation is clean (9/10) — the main domain violation is workspace owning runtime cwd
  - Frontend cross-context isolation is the weakest area (4/10) — workspace store bootstrapping other stores
  - Backend dependency direction (5/10) is second weakest — multiple app services import Tauri directly
---

## 2026-03-08T22:06 - US-001b: Fix pre-existing quality gate failures
Thread:
Run: 20260308-215923-84117 (iteration 2)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260308-215923-84117-iter-2.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260308-215923-84117-iter-2.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: 7742736 US-001b: no pre-existing failures to fix
- Post-commit status: clean
- Verification:
  - Command: `bun run lint` -> PASS
  - Command: `bun run typecheck` -> PASS
  - Command: `bun run test` -> PASS (162 tests)
  - Command: `cargo fmt --all --check` -> PASS
  - Command: `cargo clippy --workspace --all-targets --all-features -- -D warnings` -> PASS
  - Command: `cargo test --workspace` -> PASS (160 tests)
- Files changed:
  - (none — empty commit, no pre-existing failures found)
- What was implemented:
  - US-001 baseline documented zero quality gate failures. Per acceptance criteria, this story closes as a no-op with the required commit message.
- **Learnings for future iterations:**
  - All six quality gates remain green from US-001 baseline — no drift between iterations
  - Codebase has 322 total tests (162 TS + 160 Rust) all passing
---

## 2026-03-08T22:08 - US-002: Document architecture laws and dependency direction rules
Thread:
Run: 20260308-215923-84117 (iteration 3)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260308-215923-84117-iter-3.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260308-215923-84117-iter-3.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: a0aa6e7 docs: document architecture laws and dependency direction rules (US-002)
- Post-commit status: clean
- Verification:
  - Command: `bun run lint` -> PASS
  - Command: `bun run typecheck` -> PASS
  - Command: `bun run test` -> PASS (162 tests)
  - Command: `cargo fmt --all --check` -> PASS
  - Command: `cargo clippy --workspace --all-targets --all-features -- -D warnings` -> PASS
  - Command: `cargo test --workspace` -> PASS (160 tests)
- Files changed:
  - .ralph/architecture-laws.md (created — gitignored)
- What was implemented:
  - Created architecture laws document with 6 sections covering all acceptance criteria:
    1. Backend dependency direction rules with allowed/prohibited dependency tables
    2. Domain purity laws (no Tauri, no contracts DTOs, no infrastructure, no cross-crate deps, serde rules)
    3. Frontend DTO boundary law (generated types only in transport clients and mappers)
    4. Frontend cross-context isolation law (no cross-store bootstrapping)
    5. Infrastructure and transport boundary laws (thin commands, event flow)
    6. Violation reference mapping V1-V7 from baseline to specific broken laws
  - Zero source code modifications as required by story
- **Learnings for future iterations:**
  - Architecture laws now provide a clear reference for all subsequent refactor stories
  - Each violation V1-V7 maps to a specific law, making fix stories unambiguous
  - The serde exception for tabby-settings (domain IS persisted state) is documented explicitly
  - Future stories should reference specific law sections when justifying changes
---

## 2026-03-08T22:12 - US-003: Create risk map for refactor phases
Thread:
Run: 20260308-215923-84117 (iteration 4)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260308-215923-84117-iter-4.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260308-215923-84117-iter-4.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: 7839b7b docs: create risk map for refactor phases with blast radius analysis (US-003)
- Post-commit status: clean
- Verification:
  - Command: `bun run lint` -> PASS
  - Command: `bun run typecheck` -> PASS
  - Command: `bun run test` -> PASS (162 tests)
  - Command: `cargo fmt --all --check` -> PASS
  - Command: `cargo clippy --workspace --all-targets --all-features -- -D warnings` -> PASS
  - Command: `cargo test --workspace` -> PASS (160 tests)
- Files changed:
  - .ralph/risk-map.md (created — force-added despite gitignore)
- What was implemented:
  - Created comprehensive risk map covering all 10 phases (0-9) of the v2 architecture refactor
  - Phase 1 (runtime lifecycle) marked CRITICAL with 4 detailed risk scenarios:
    1. Pane replace — old runtime leak if event missed during event-driven transition
    2. Tab close — orphaned PTY processes if PaneRemoved events are missed
    3. Terminal natural exit — frontend never learns terminal exited if observation receiver not wired
    4. Browser visibility restore — browser pane reloads on tab switch if lifecycle mishandles ActiveTabChanged
  - Each risk entry includes: description, affected files, blast radius, mitigation strategy, required pre-existing tests
  - Risk severity scale: CRITICAL > HIGH > MEDIUM > LOW
  - Phase execution order with dependency gates
  - Risk-ordered priority summary table
  - Zero source code modifications as required by story
- **Learnings for future iterations:**
  - `.ralph/` is gitignored — use `git add -f` to force-track documentation files
  - Phase 1 is the most dangerous — touches every PTY spawn/exit/replace/restart flow
  - Phases 2-3 (workspace slimming) are HIGH risk because they change the core data model
  - Phases 4-7 are MEDIUM risk — port traits, ACL, coordination are safer due to type system protection
  - Phases 8-9 are LOW risk — test-only and cleanup with quality gate protection
---

## 2026-03-08 22:18 - US-004: Define RuntimeObservationReceiver trait as application-owned callback interface
Thread:
Run: 20260308-215923-84117 (iteration 5)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260308-215923-84117-iter-5.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260308-215923-84117-iter-5.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: fc0cff5 feat: define RuntimeObservationReceiver trait as application-owned callback interface (US-004)
- Post-commit status: clean
- Verification:
  - Command: bun run lint -> PASS
  - Command: bun run typecheck -> PASS
  - Command: bun run test -> PASS (162 tests)
  - Command: cargo fmt --all --check -> PASS
  - Command: cargo clippy --workspace --all-targets --all-features -- -D warnings -> PASS
  - Command: cargo test --workspace -> PASS (106 Rust tests + 61 crate tests)
- Files changed:
  - src-tauri/src/application/runtime_observation_receiver.rs (new — trait + 7 tests)
  - src-tauri/src/application/mod.rs (register module, public export)
  - src-tauri/src/application/runtime_service.rs (impl RuntimeObservationReceiver for RuntimeApplicationService)
- Implemented RuntimeObservationReceiver trait with 4 methods: on_terminal_output_received, on_terminal_exited, on_browser_location_changed, on_terminal_cwd_changed
- Trait uses domain types only (PaneId from tabby-workspace), no Tauri/DTO/transport types
- RuntimeApplicationService implements the trait — on_terminal_exited and on_browser_location_changed delegate to RuntimeRegistry + ProjectionPublisher
- on_terminal_output_received and on_terminal_cwd_changed log observations (full wiring in future stories)
- 7 unit tests verify mock infra can call all trait methods, object safety behind Arc, and accumulation ordering
- Existing PTY and browser code unchanged — trait introduced but not yet wired to infrastructure
- **Learnings for future iterations:**
  - Rust dead_code lint fires on traits that are impl'd but never used as trait bounds — use #[allow(dead_code)] for intentionally introduced-but-not-yet-wired ports
  - The trait must be Send + Sync for Arc<dyn RuntimeObservationReceiver> usage by infra threads
  - Terminal output uses raw bytes (&[u8]) while exit uses domain-friendly Option<i32> exit codes
---

## 2026-03-08 22:27 - US-005: Stop PTY infrastructure from directly emitting runtime status events
Thread:
Run: 20260308-215923-84117 (iteration 6)
Run log: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260308-215923-84117-iter-6.log
Run summary: /Users/markbrutx/pet/Tabby/.ralph/runs/run-20260308-215923-84117-iter-6.md
- Guardrails reviewed: yes
- No-commit run: false
- Commit: 117b4cd feat: wire PTY exit to RuntimeObservationReceiver instead of direct event emit (US-005)
- Post-commit status: clean
- Verification:
  - Command: `bun run lint` -> PASS
  - Command: `bun run typecheck` -> PASS
  - Command: `bun run test` -> PASS (162 tests)
  - Command: `cargo fmt --all --check` -> PASS
  - Command: `cargo clippy --workspace --all-targets --all-features -- -D warnings` -> PASS
  - Command: `cargo test --workspace` -> PASS (109 app tests + 61 crate tests = 170 total)
- Files changed:
  - src-tauri/src/shell/pty.rs (PTY read thread now calls observation_receiver.on_terminal_exited() instead of emitting RuntimeStatusChangedEvent)
  - src-tauri/src/application/runtime_service.rs (start_runtime/restart_runtime accept Arc<dyn RuntimeObservationReceiver>)
  - src-tauri/src/application/runtime_observation_receiver.rs (removed dead_code allow, added 3 integration tests)
  - src-tauri/src/application/runtime_coordinator.rs (passes observation_receiver through to start_runtime)
  - src-tauri/src/application/bootstrap_service.rs (passes observation_receiver through to coordinator)
  - src-tauri/src/shell/mod.rs (AppShell stores Arc<RuntimeApplicationService>, adds observation_receiver() helper)
- What was implemented:
  - PTY read thread no longer builds RuntimeStatusChangedEvent DTOs or emits via app.emit(RUNTIME_STATUS_CHANGED_EVENT)
  - Instead, PTY thread resolves exit code and calls RuntimeObservationReceiver.on_terminal_exited(pane_id, exit_code)
  - RuntimeApplicationService (which implements the trait) receives the observation, updates registry, emits projection
  - Terminal output (TERMINAL_OUTPUT_RECEIVED_EVENT) remains as direct emit — it's raw I/O, not domain state
  - Removed build_terminal_exit_event function, replaced with simpler resolve_exit_code
  - Removed unused imports: PaneRuntimeView, RuntimeKindDto, RuntimeStatusChangedEvent, RuntimeStatusDto from pty.rs
  - AppShell now stores runtime_service as Arc<RuntimeApplicationService> for trait object coercion
  - 3 new integration-style tests: normal exit → Exited, non-zero exit → Failed with error message, unknown exit → Exited
- **Learnings for future iterations:**
  - Arc<ConcreteType> does not auto-coerce to Arc<dyn Trait> via Arc::clone — need explicit cast: `Arc::clone(&x) as Arc<dyn Trait>`
  - Created observation_receiver() helper method on AppShell to centralize the coercion
  - The observation_receiver parameter threads through: AppShell → BootstrapService → RuntimeCoordinator → RuntimeApplicationService → PtyManager::spawn
  - portable_pty exit_code() returns u32, converted via i32::try_from with unwrap_or(i32::MAX) fallback
---
