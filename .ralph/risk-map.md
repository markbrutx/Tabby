# Risk Map — Architecture Foundation v2 Refactor

> Created as part of US-003. Maps blast radius, mitigation strategy, and required
> pre-existing tests for each refactor phase. Use this document to ensure dangerous
> changes have test coverage before they begin.

---

## Risk Severity Scale

| Level | Meaning |
|-------|---------|
| **CRITICAL** | Can break running terminals, lose user data, or prevent app launch |
| **HIGH** | Can break runtime lifecycle or cross-context state consistency |
| **MEDIUM** | Can break projections, UI sync, or persistence round-trip |
| **LOW** | Isolated structural change with narrow blast radius |

---

## Phase 0: Documentation & Baseline (US-001, US-001b, US-002, US-003)

**Risk: LOW**

| Item | Detail |
|------|--------|
| Description | Read-only documentation stories. No source code modifications. |
| Affected files | `.ralph/baseline.md`, `.ralph/architecture-laws.md`, `.ralph/risk-map.md` |
| Mitigation | Quality gates catch accidental source changes. |
| Required pre-existing tests | None — documentation only. |

---

## Phase 1: Runtime Lifecycle Single-Ownership (US-004 → US-008)

**Risk: CRITICAL — Highest risk phase**

This phase restructures who owns runtime lifecycle transitions. Every PTY spawn, exit, replace, and restart flow is touched. A regression here means terminals silently fail to start, fail to clean up, or lose output.

### Specific Risk Scenarios

#### 1.1 Pane Replace (terminal→browser, browser→terminal)

| Item | Detail |
|------|--------|
| Description | `replace_pane_spec` currently manually calls `stop_runtime` in `AppShell` before workspace mutation. Moving this to event-driven `RuntimeCoordinator` creates a window where the old runtime could leak if the event is missed or processed out of order. |
| Affected files | `src-tauri/src/shell/mod.rs` (AppShell::replace_pane_spec), `src-tauri/src/application/runtime_coordinator.rs`, `src-tauri/src/application/runtime_service.rs`, `src-tauri/src/shell/pty.rs`, `src-tauri/src/shell/browser_surface.rs` |
| Blast radius | Old PTY process leaked (zombie process), new runtime fails to start, user sees blank pane |
| Mitigation | 1) Write regression test for replace flow BEFORE moving to event-driven: assert old runtime stopped + new runtime started. 2) Add a "no orphaned runtimes" invariant check after each workspace mutation. 3) Keep manual stop as fallback until event-driven path is proven. |
| Required pre-existing tests | E2E: `runtime-lifecycle.spec.ts` — pane replace scenario. Unit: RuntimeCoordinator handles PaneSpecReplaced event. |

#### 1.2 Tab Close (multi-pane cleanup)

| Item | Detail |
|------|--------|
| Description | Closing a tab with N panes must stop N runtimes. If `RuntimeCoordinator` misses a `PaneRemoved` event, orphaned PTY processes accumulate. |
| Affected files | `src-tauri/src/application/workspace_service.rs` (close_tab), `src-tauri/src/application/runtime_coordinator.rs`, `src-tauri/src/application/runtime_service.rs`, `src-tauri/src/shell/pty.rs` |
| Blast radius | Zombie PTY processes consuming CPU/memory, runtime registry out of sync with workspace state |
| Mitigation | 1) Write test: close_tab with 3 panes → all 3 runtimes stopped. 2) Add registry cleanup sweep that detects orphaned entries. 3) Log warnings when registry contains entries for panes not in workspace. |
| Required pre-existing tests | E2E: tab close cleans up runtime. Unit: workspace `close_tab` emits PaneRemoved for every pane in the tab. |

#### 1.3 Terminal Natural Exit

| Item | Detail |
|------|--------|
| Description | When a shell process exits naturally, PTY infrastructure currently emits `RuntimeStatusChangedEvent` directly via `app.emit()`. Rerouting through `RuntimeObservationReceiver` → `RuntimeApplicationService` adds indirection. If the receiver is not wired correctly, the frontend never learns the terminal exited. |
| Affected files | `src-tauri/src/shell/pty.rs` (read thread exit path), `src-tauri/src/application/runtime_service.rs`, `src-tauri/src/application/projection_publisher.rs` |
| Blast radius | Terminal shows as "running" forever after shell exits, user cannot restart or replace the pane |
| Mitigation | 1) Write test: PTY exit → `on_terminal_exited` called → registry updated → projection emitted with Exited status. 2) Add a timeout watchdog: if no projection is emitted within 5s of PTY exit, log error. 3) Keep terminal output (`TERMINAL_OUTPUT_RECEIVED_EVENT`) as direct emit since it's raw I/O — only status changes go through the new path. |
| Required pre-existing tests | E2E: `runtime-lifecycle.spec.ts` — terminal exit detection. Unit: RuntimeApplicationService handles terminal exit observation. |

#### 1.4 Browser Visibility Restore

| Item | Detail |
|------|--------|
| Description | Browser panes use `set_visible` / `set_bounds` to show/hide the webview on tab switch. If runtime lifecycle changes accidentally stop/restart the browser runtime on `ActiveTabChanged`, the user loses their browsing session. |
| Affected files | `src-tauri/src/shell/browser_surface.rs`, `src-tauri/src/application/runtime_coordinator.rs`, `src-tauri/src/application/runtime_service.rs` |
| Blast radius | Browser pane reloads on every tab switch, losing form data, scroll position, and navigation history |
| Mitigation | 1) Write explicit test: `ActiveTabChanged` event does NOT trigger runtime stop/start — only visibility update. 2) Ensure `RuntimeCoordinator` has no handler for `ActiveTabChanged` related to lifecycle. 3) Browser visibility remains a shell concern (not a lifecycle concern). |
| Required pre-existing tests | E2E: browser pane survives tab switch. Unit: RuntimeCoordinator ignores ActiveTabChanged for lifecycle purposes. |

---

## Phase 2: Workspace Slimming — PaneSlot + PaneContentDefinition (US-009, US-010, US-011)

**Risk: HIGH**

### Description

Introduces `PaneContentDefinition` as a new domain type and migrates workspace `Pane` to use `PaneSlot` with a content reference. This changes the core workspace data model that every workspace operation touches.

| Item | Detail |
|------|--------|
| Affected files | `src-tauri/crates/tabby-workspace/src/lib.rs` (Pane struct, WorkspaceSession methods), `src-tauri/crates/tabby-workspace/src/ids.rs` (new PaneContentId), `src-tauri/src/application/workspace_service.rs`, `src-tauri/src/application/runtime_coordinator.rs`, `src-tauri/src/mapping/dto_mappers.rs`, `src/features/workspace/application/snapshot-mappers.ts` |
| Blast radius | Every workspace mutation (open_tab, close_tab, split_pane, close_pane, replace_pane_spec) could break. Domain events change payload shape, breaking RuntimeCoordinator and frontend mappers. |
| Mitigation | 1) Introduce `PaneContentDefinition` as a standalone type first (US-009) with full tests before touching Pane. 2) Migrate Pane to PaneSlot atomically — all workspace tests must pass before moving to events/mappers. 3) Update mapper layer and frontend mappers in the same story (US-011). 4) Assert 1:1 ownership invariant: no orphaned PaneContentDefinitions after any mutation. |
| Required pre-existing tests | Workspace domain: all operations (open_tab, close_tab, split_pane, close_pane, focus_pane, replace_pane_spec). Frontend: snapshot-mappers.test.ts. RuntimeCoordinator: event handling. |

---

## Phase 3: Workspace Slimming — CWD Removal + Event Separation (US-012, US-013, US-014)

**Risk: HIGH**

### Description

Removes `track_terminal_working_directory` from workspace domain and separates structural events from content events. The cwd data source changes from workspace projection to runtime projection.

| Item | Detail |
|------|--------|
| Affected files | `src-tauri/crates/tabby-workspace/src/lib.rs` (remove track_terminal_working_directory), `src-tauri/src/application/runtime_service.rs` (absorb cwd tracking), `src-tauri/src/application/workspace_service.rs`, `src-tauri/src/application/projection_publisher.rs`, `src/features/workspace/application/snapshot-mappers.ts`, `src/features/workspace/model/workspaceSnapshot.ts`, `src/features/terminal/components/PaneHeader.tsx` |
| Blast radius | PaneHeader loses cwd display (shows stale or empty path). Settings fail to persist last-used cwd per profile. Frontend workspace snapshot model shape changes — components reading cwd break. |
| Mitigation | 1) Move cwd tracking to RuntimeApplicationService first, verify it updates registry correctly. 2) Update frontend to read cwd from runtime store, not workspace store. 3) Verify settings persistence still saves last-used cwd. 4) Keep old cwd path working alongside new path during transition within the story. |
| Required pre-existing tests | Unit: workspace cwd tracking works. Frontend: PaneHeader displays cwd. Settings: cwd persists across sessions. |

---

## Phase 4: Explicit Ports (US-015, US-016, US-017, US-018)

**Risk: MEDIUM**

### Description

Introduces port traits (`PreferencesRepository`, `TerminalProcessPort`, `BrowserSurfacePort`, `ProjectionPublisherPort`) and moves concrete infrastructure behind them. Application services switch from direct Tauri imports to trait objects.

| Item | Detail |
|------|--------|
| Affected files | `src-tauri/src/application/settings_service.rs`, `src-tauri/src/application/runtime_service.rs`, `src-tauri/src/application/projection_publisher.rs`, `src-tauri/src/shell/pty.rs`, `src-tauri/src/shell/browser_surface.rs`, `src-tauri/src/shell/mod.rs`, `src-tauri/src/lib.rs` (DI wiring) |
| Blast radius | DI wiring error → app fails to start. Trait mismatch → compile error (caught by clippy). Incorrect port implementation → settings not saved, PTY not spawned, projections not emitted. |
| Mitigation | 1) Introduce each port one at a time (US-015 → US-018) — never change all at once. 2) Each port story adds a mock-based test proving the app service works through the trait. 3) Wire real implementations in `lib.rs` and verify `cargo test` + `bun run tauri dev` still works. 4) Persistence port (US-015) is lowest risk — start there to build confidence. |
| Required pre-existing tests | Settings: load/save round-trip. Runtime: spawn/stop lifecycle. Projection: events emitted correctly. All existing cargo tests and E2E tests. |

---

## Phase 5: Frontend ACL Completion (US-019, US-020, US-021)

**Risk: MEDIUM**

### Description

Creates `RuntimeReadModel` and `RuntimeSnapshotMapper`, then removes `PaneRuntimeView` DTO from stores and composed snapshot. Adds automated DTO boundary enforcement.

| Item | Detail |
|------|--------|
| Affected files | `src/features/runtime/domain/models.ts`, `src/features/runtime/application/store.ts`, `src/features/workspace/model/workspaceSnapshot.ts`, `src/features/terminal/components/PaneHeader.tsx`, `src/features/terminal/components/TerminalPane.tsx`, `src/features/browser/components/BrowserPane.tsx` |
| Blast radius | Field naming changes (snake_case → camelCase) break every component that reads runtime data. Missing mapper field → undefined runtime status in UI. |
| Mitigation | 1) Create mapper and read model first (US-019) without wiring — verify with tests. 2) Wire into store (US-020) and update all consumers in the same story. 3) Use TypeScript compiler as safety net — type errors catch missed consumers. 4) Add grep-based enforcement script (US-021) to prevent regression. |
| Required pre-existing tests | Runtime store tests. Workspace snapshot tests. Frontend component tests (PaneHeader, BrowserToolbar). TypeScript typecheck as implicit coverage. |

---

## Phase 6: Frontend Cross-Context Coordination (US-022, US-023, US-024)

**Risk: MEDIUM**

### Description

Extracts bootstrap orchestration from workspace store to `AppBootstrapCoordinator`. Moves onboarding cross-store updates to coordinator. Decouples all feature stores.

| Item | Detail |
|------|--------|
| Affected files | `src/contexts/stores.ts`, `src/features/workspace/application/store.ts`, `src/features/settings/application/store.ts`, `src/features/runtime/application/store.ts`, `src/App.tsx` or `src/app-shell/context/AppShellContext.tsx` |
| Blast radius | Bootstrap sequencing error → app shows blank screen or stale data on launch. Onboarding flow breaks → new users stuck on setup wizard. Store initialization race condition → undefined state. |
| Mitigation | 1) Create coordinator and wire it BEFORE removing cross-store calls from workspace store. 2) Run the app manually after each sub-story to verify bootstrap still works. 3) Test coordinator with mock stores to verify sequencing. 4) Verify onboarding flow end-to-end after US-023. |
| Required pre-existing tests | Workspace store test: initialize() loads all data. Settings store test: loadBootstrap(). E2E: app launches correctly. E2E: setup wizard completes. |

---

## Phase 7: Contract Hygiene + Value Objects + Naming (US-025, US-026, US-027, US-028)

**Risk: MEDIUM**

### Description

Unifies browser location into runtime projection, removes ghost events, introduces value objects for raw strings, and audits naming against ubiquitous language.

| Item | Detail |
|------|--------|
| Affected files | `src-tauri/crates/tabby-contracts/src/lib.rs`, `src-tauri/crates/tabby-workspace/src/ids.rs`, `src-tauri/crates/tabby-runtime/src/ids.rs`, `src-tauri/src/shell/browser_surface.rs`, `src-tauri/src/application/runtime_service.rs`, `src/features/browser/hooks/useBrowserWebview.ts`, `src/features/runtime/application/store.ts` |
| Blast radius | Removing BrowserLocationObservedEvent breaks frontend browser location display if the unified path isn't wired first. Value object newtypes can cascade compile errors across all crates. Naming renames break imports across the entire codebase. |
| Mitigation | 1) Wire unified browser location path BEFORE removing the old event (US-025). 2) Introduce value objects incrementally — one type at a time, verify all crates compile. 3) Use IDE rename / cargo clippy to catch missed references. 4) Naming audit (US-028) should be last — rename after all structural changes are stable. |
| Required pre-existing tests | Browser pane: location display works. Runtime store: browser_location field populated. All cargo tests (catch compile errors from value object changes). |

---

## Phase 8: Integration Tests (US-029, US-030, US-031)

**Risk: LOW**

### Description

Adds backend runtime lifecycle integration tests, frontend mapper/coordinator tests, and a full command-to-side-effect dispatch integration test. Test-only stories — no production code changes.

| Item | Detail |
|------|--------|
| Affected files | New test files only. No production code changes. |
| Blast radius | Minimal — tests that fail reveal bugs but don't break the app. Test infrastructure setup (mock ports, test wiring) could be incorrect. |
| Mitigation | 1) Build mock port implementations that record calls for assertion. 2) Wire real application services with mock ports — this validates the DI graph. 3) Document test strategy in `.ralph/test-strategy.md`. |
| Required pre-existing tests | All quality gates green. All port traits defined and implemented. |

---

## Phase 9: Cleanup + Documentation Sync (US-032, US-033)

**Risk: LOW**

### Description

Removes transitional adapters, compatibility shims, and TODO comments. Updates CLAUDE.md and AGENTS.md to match final architecture.

| Item | Detail |
|------|--------|
| Affected files | Various (removing dead code), `CLAUDE.md`, `AGENTS.md`, `.ralph/baseline.md` |
| Blast radius | Removing code that is actually still used → compile errors (caught by quality gates). Missing documentation updates → stale architecture map. |
| Mitigation | 1) Use `cargo clippy` dead code warnings and `knip` for TypeScript to identify truly unused code. 2) Remove one module at a time, verify quality gates pass. 3) Read the codebase before removing — never assume something is unused. |
| Required pre-existing tests | All quality gates. All integration tests from Phase 8. Full `cargo test --workspace` and `bun run test`. |

---

## Phase Execution Order and Dependencies

```
Phase 0 (docs) ──────────────────────────────────────────────►
Phase 1 (runtime lifecycle) ─────► CRITICAL GATE: all lifecycle tests pass
Phase 2 (workspace model) ───────► HIGH GATE: workspace domain tests pass
Phase 3 (cwd + events) ─────────► HIGH GATE: cwd display + persistence verified
Phase 4 (ports) ─────────────────► MEDIUM GATE: app starts, all services work through traits
Phase 5 (frontend ACL) ─────────► MEDIUM GATE: no DTOs in stores, typecheck passes
Phase 6 (coordination) ─────────► MEDIUM GATE: bootstrap + onboarding work
Phase 7 (contracts + naming) ────► MEDIUM GATE: all crates compile, browser location works
Phase 8 (integration tests) ────► LOW: test-only, no production changes
Phase 9 (cleanup) ──────────────► LOW: dead code removal with quality gate protection
```

---

## Summary: Risk-Ordered Priority

| Priority | Phase | Risk | Key Danger |
|----------|-------|------|-----------|
| 1 | Phase 1 — Runtime lifecycle | **CRITICAL** | Zombie PTY processes, lost terminal state, broken replace/restart |
| 2 | Phase 2 — PaneSlot migration | **HIGH** | Every workspace operation touches the changed data model |
| 3 | Phase 3 — CWD + event separation | **HIGH** | CWD display and persistence data source changes |
| 4 | Phase 4 — Explicit ports | **MEDIUM** | DI wiring errors prevent app launch |
| 5 | Phase 5 — Frontend ACL | **MEDIUM** | Field naming changes cascade through all components |
| 6 | Phase 6 — Coordination | **MEDIUM** | Bootstrap sequencing errors cause blank screen |
| 7 | Phase 7 — Contracts + naming | **MEDIUM** | Cross-crate cascading renames |
| 8 | Phase 8 — Integration tests | **LOW** | Test-only changes |
| 9 | Phase 9 — Cleanup | **LOW** | Protected by quality gates |
| — | Phase 0 — Documentation | **LOW** | No source code changes |

---

*This risk map should be consulted before starting each phase. The mitigation strategies define what tests must exist BEFORE the risky changes begin.*
