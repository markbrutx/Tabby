# Test Strategy — Tabby Backend Integration Tests

## Overview

The Rust backend uses a layered testing strategy that verifies the full command-to-side-effect dispatch pipeline without requiring a running Tauri app, real PTY processes, or browser surfaces.

## Test Layers

### 1. Unit Tests (domain crates)

Each domain crate (`tabby-workspace`, `tabby-runtime`, `tabby-settings`, `tabby-contracts`) contains unit tests for its internal logic:
- `WorkspaceSession` layout and pane management
- `RuntimeRegistry` state transitions
- `UserPreferences` serialization and defaults
- DTO mapping correctness

### 2. Application Service Tests

`workspace_service.rs`, `settings_service.rs`, and `runtime_service.rs` each have `#[cfg(test)] mod tests` that verify individual service behavior in isolation.

### 3. Runtime Integration Tests (US-029)

**File:** `runtime_integration_tests.rs`

Tests the `RuntimeCoordinator` → `RuntimeApplicationService` → mock ports flow. These tests start from pre-built `WorkspaceDomainEvent` vectors and verify that the correct port methods (`spawn`, `kill`, `close_surface`) are called with correct arguments.

### 4. Command Dispatch Integration Tests (US-031)

**File:** `command_dispatch_integration_tests.rs`

Tests the **full dispatch pipeline**: workspace commands (`open_tab`, `close_pane`, `close_tab`, `split_pane`, `replace_pane_spec`) → domain events → `RuntimeCoordinator` → `RuntimeApplicationService` → mock infrastructure ports. This is the highest-level backend integration test and catches regressions that span the workspace-to-runtime boundary.

**Why this matters:** The command dispatch tests exercise the same code path that Tauri command handlers use in production. If `WorkspaceApplicationService.open_tab()` produces incorrect events, or `RuntimeCoordinator` misroutes them, these tests will catch it — without needing a running Tauri app.

## Mock Port Design

All four infrastructure ports are mocked:

| Port | Mock Records |
|------|-------------|
| `TerminalProcessPort` | `spawn_calls` (pane_id, cwd, command), `kill_calls` (session_id) |
| `BrowserSurfacePort` | `ensure_surface_calls`, `close_calls` |
| `ProjectionPublisherPort` | `runtime_statuses` (pane_id, kind, status) |
| `PreferencesRepository` | `stored` (serialized preferences) |

Mocks use `Arc<Mutex<Vec<...>>>` for thread-safe recording. Arc wrappers delegate to the shared mock instances so both the service and the test can access recorded calls.

## Test Harness Pattern

The `DispatchHarness` (US-031) wires all three real application services (`WorkspaceApplicationService`, `SettingsApplicationService`, `RuntimeApplicationService`) with mock ports. The `coordinate()` method feeds workspace events through `RuntimeCoordinator`, exactly as `AppShell` does in production.

## What Is NOT Tested Here

- **Tauri IPC serialization** — tested via specta type generation and frontend contract tests
- **Real PTY/browser lifecycle** — infeasible without a running OS process; covered by manual QA
- **Frontend rendering** — covered by Vitest component tests and Playwright E2E
- **Tauri command handlers** — intentionally thin; delegate to application services which are tested here

## Coverage Summary

| Test File | Test Count | Scope |
|-----------|-----------|-------|
| `runtime_coordinator.rs` (unit) | 16 | Event classification, registry operations |
| `runtime_integration_tests.rs` | 18 | Events → coordinator → ports |
| `command_dispatch_integration_tests.rs` | 9 | Commands → events → coordinator → ports |
| `workspace_service.rs` (unit) | 12 | Workspace command behavior |
| `runtime_service.rs` (unit) | 18+ | Runtime service operations |
| `settings_service.rs` (unit) | 6+ | Settings persistence |
