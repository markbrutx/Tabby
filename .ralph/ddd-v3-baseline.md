# DDD v3 Baseline — 7 Audit Violations

**Date:** 2026-03-09
**DDD Audit Score:** 6.8/10
**Story:** DDD-001

This document records the 7 specific DDD violations identified by external audit, with exact file:line references. This is the baseline for tracking remediation progress through stories DDD-002 to DDD-018.

---

## Violation #1: Domain crates depend on tabby-contracts (IPC crate)

**Severity:** CRITICAL
**Remediation:** DDD-002, DDD-003, DDD-004

All three domain crates import `tabby-contracts` directly. Domain crates should depend on a shared kernel (pure value objects), not on an IPC/serialization crate that carries `serde`, `specta`, and Tauri-specific derives.

| File | Line | Content |
|------|------|---------|
| `src-tauri/crates/tabby-workspace/Cargo.toml` | 7 | `tabby-contracts = { path = "../tabby-contracts" }` |
| `src-tauri/crates/tabby-runtime/Cargo.toml` | 7 | `tabby-contracts = { path = "../tabby-contracts" }` |
| `src-tauri/crates/tabby-settings/Cargo.toml` | 7 | `tabby-contracts = { path = "../tabby-contracts" }` |

**Why it matters:** Domain crates pull in transport/serialization concerns (serde, specta) transitively. The dependency arrow goes Domain → IPC, violating the rule that domain code must be infrastructure-agnostic.

---

## Violation #2: Browser surface commands bypass RuntimeApplicationService

**Severity:** HIGH
**Remediation:** DDD-005, DDD-006, DDD-007

The `dispatch_browser_surface_command` Tauri command calls the infrastructure function `execute_browser_surface_command` directly, bypassing `AppShell → RuntimeApplicationService`. This violates the single-owner invariant: RuntimeApplicationService should be the sole owner of all runtime lifecycle operations.

| File | Line | Content |
|------|------|---------|
| `src-tauri/src/commands/shell.rs` | 51–56 | `pub fn dispatch_browser_surface_command(…) { execute_browser_surface_command(&window, command) }` |
| `src-tauri/src/shell/browser_surface.rs` | 11–29 | `execute_browser_surface_command` — infrastructure function called directly |

**Why it matters:** Browser runtime lifecycle is managed outside the RuntimeApplicationService, creating a second entry point for runtime mutations. Observation callbacks (line 94 of browser_surface.rs) route through the shell, but commands do not.

---

## Violation #3: Cross-context coupling — Runtime mutates Settings

**Severity:** CRITICAL
**Remediation:** DDD-008

`observe_terminal_cwd` in `RuntimeApplicationService` takes `&SettingsApplicationService` as a parameter and directly mutates settings (persists `last_working_directory`). The Runtime context should not reach into the Settings context.

| File | Line | Content |
|------|------|---------|
| `src-tauri/src/application/runtime_service.rs` | 178–194 | `pub fn observe_terminal_cwd(&self, …, settings_service: &SettingsApplicationService)` |
| `src-tauri/src/application/runtime_service.rs` | 182 | `settings_service: &SettingsApplicationService` — cross-context parameter |
| `src-tauri/src/application/runtime_service.rs` | 190–192 | `settings_service.preferences()? … settings_service.persist_preferences()` — mutation |
| `src-tauri/src/shell/mod.rs` | 136–140 | Caller passes `&self.settings_service` across context boundary |

**Why it matters:** Runtime context directly depends on and mutates Settings context, violating bounded context independence. Cross-context side effects should be handled by the AppShell coordinator.

---

## Violation #4: ProjectionPublisherPort accepts DTO instead of domain type

**Severity:** HIGH
**Remediation:** DDD-009

`publish_workspace_projection` on `ProjectionPublisherPort` accepts `&WorkspaceView` (a DTO from tabby-contracts) instead of `&WorkspaceSession` (the domain aggregate). The port sits at the application layer and should accept domain types; DTO mapping belongs in the infrastructure adapter.

| File | Line | Content |
|------|------|---------|
| `src-tauri/src/application/ports.rs` | 28 | `fn publish_workspace_projection(&self, workspace: &WorkspaceView);` |

**Why it matters:** The application-layer port trait is coupled to transport DTOs. Callers must construct a `WorkspaceView` DTO before publishing, which means domain-to-DTO mapping leaks into the application/service layer rather than being encapsulated in the infrastructure adapter.

---

## Violation #5: Frontend stores accept DTOs at bootstrap (ACL boundary incomplete)

**Severity:** MEDIUM
**Remediation:** DDD-011, DDD-012, DDD-013

All three frontend stores accept DTO types directly in their `loadBootstrap` signatures. The anti-corruption layer (snapshot-mappers) exists but is called inside the store. The store interface should accept internal read models only; DTO→ReadModel mapping should happen in `AppBootstrapCoordinator.initialize()`.

| File | Line | Content |
|------|------|---------|
| `src/features/workspace/application/store.ts` | 28 | `loadBootstrap: (payload: WorkspaceBootstrapView) => Promise<void>` |
| `src/features/settings/application/store.ts` | 17 | `loadBootstrap: (settings: SettingsView, profiles: readonly {…}[]) => void` |
| `src/features/runtime/application/store.ts` | 10 | `loadBootstrap: (runtimes: PaneRuntimeView[]) => void` |

**Why it matters:** DTO types from `tauri-bindings.ts` penetrate into store interfaces. The ACL boundary is technically present (mappers run inside loadBootstrap) but architecturally incomplete — stores should be DTO-unaware.

---

## Violation #6: Terminal output bypasses RuntimeObservationReceiver

**Severity:** HIGH (acknowledged design trade-off)
**Remediation:** DDD-010 (ADR documenting exemption)

Terminal output is emitted directly via `app.emit()` in the PTY reader loop, bypassing the `RuntimeObservationReceiver` trait. Exit events use the receiver trait (line 121), creating an asymmetric pattern. This is a deliberate performance trade-off (high-frequency byte stream) that needs documented justification.

| File | Line | Content |
|------|------|---------|
| `src-tauri/src/shell/pty.rs` | 100–110 | `app.emit(TERMINAL_OUTPUT_RECEIVED_EVENT, TerminalOutputEvent { … })` |
| `src-tauri/src/shell/pty.rs` | 121 | `observation_receiver.on_terminal_exited(…)` — exit uses the trait |
| `src-tauri/src/shell/mod.rs` | 35 | `pub const TERMINAL_OUTPUT_RECEIVED_EVENT` — hardcoded event name |

**Why it matters:** The abstraction is inconsistent: exit events flow through the port trait but output events bypass it entirely, coupling the PTY implementation directly to the Tauri event system.

---

## Violation #7: Stringly-typed fields where value objects should be used

**Severity:** MEDIUM
**Remediation:** DDD-014, DDD-015

Several domain model fields use raw `String` instead of typed value objects, deferring validation to runtime rather than compile-time.

### 7a: UserPreferences.default_layout

| File | Line | Content |
|------|------|---------|
| `src-tauri/crates/tabby-settings/src/lib.rs` | 36 | `pub default_layout: String` |
| `src-tauri/crates/tabby-settings/src/lib.rs` | 68 | `default_layout: String::from(DEFAULT_LAYOUT_PRESET)` |
| `src-tauri/crates/tabby-settings/src/lib.rs` | 127–128 | Runtime validation via `is_known_layout_preset()` instead of type safety |

Should be a `LayoutPreset` enum (Single, SplitHorizontal, SplitVertical, Grid2x2).

### 7b: PaneRuntime string fields

| File | Line | Content |
|------|------|---------|
| `src-tauri/crates/tabby-runtime/src/lib.rs` | 31 | `pub browser_location: Option<String>` |
| `src-tauri/crates/tabby-runtime/src/lib.rs` | 32 | `pub terminal_cwd: Option<String>` |

Should be `Option<BrowserUrl>` and `Option<WorkingDirectory>` respectively. Value objects `BrowserUrl` and `WorkingDirectory` already exist in `tabby-contracts` but are not used in the domain crate.

---

## Quality Gate Results (Baseline)

All quality gates pass as of 2026-03-09.

| Gate | Command | Result | Details |
|------|---------|--------|---------|
| Lint | `bun run lint` | PASS | ESLint + DTO boundary check clean |
| TypeCheck | `bun run typecheck` | PASS | `tsc --noEmit` clean |
| Unit Tests (Frontend) | `bun run test` | PASS | 19 test files, **203 tests** passed |
| Cargo Format | `cargo fmt --all --check` | PASS | No formatting issues |
| Cargo Clippy | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS | No warnings |
| Cargo Tests (Backend) | `cargo test --workspace` | PASS | **300 tests** passed (172 app-lib + 29 contracts + 11 runtime + 35 settings + 53 workspace) |

**Total tests:** 503 (203 frontend + 300 backend)

---

## Summary

| # | Violation | Severity | Remediation Stories |
|---|-----------|----------|-------------------|
| 1 | Domain crates depend on tabby-contracts | CRITICAL | DDD-002, DDD-003, DDD-004, DDD-017 |
| 2 | Browser commands bypass RuntimeApplicationService | HIGH | DDD-005, DDD-006, DDD-007 |
| 3 | Runtime mutates Settings (cross-context) | CRITICAL | DDD-008 |
| 4 | ProjectionPublisherPort accepts DTO | HIGH | DDD-009 |
| 5 | Frontend stores accept DTOs at bootstrap | MEDIUM | DDD-011, DDD-012, DDD-013 |
| 6 | Terminal output bypasses observation receiver | HIGH | DDD-010 (ADR) |
| 7 | Stringly-typed fields in domain models | MEDIUM | DDD-014, DDD-015 |
