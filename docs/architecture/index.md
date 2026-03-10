# Architecture Overview

Tabby follows a layered architecture with five bounded contexts: **Workspace**, **Runtime**, **Settings**, **Git**, and **Shell/Transport**. Application services depend on abstract port traits; infrastructure adapters implement those ports.

## System Layers

```
+-----------------------------------------------------------+
|                    Presentation (src/)                     |
|  features/{workspace,terminal,browser,settings,git,theme} |
|  app-shell/  ·  components/  ·  hooks/                    |
+-----------------------------------------------------------+
|               Transport Boundary (IPC)                     |
|  contracts/tauri-bindings.ts  <->  tabby-contracts crate   |
|  app-shell/clients/ (createTauriShellClients)              |
|  src-tauri/src/mapping/ (DTO <-> domain mappers)           |
+-----------------------------------------------------------+
|              Application Services (Rust)                   |
|  src-tauri/src/application/                                |
|    workspace_service · settings_service · runtime_service  |
|    git_service · runtime_coordinator · bootstrap_service   |
|    ports (5 trait definitions)                             |
+-----------------------------------------------------------+
|              Infrastructure (Rust)                         |
|  src-tauri/src/infrastructure/                             |
|    TauriProjectionPublisher · TauriStorePreferencesRepo    |
|    TauriBrowserSurfaceAdapter · CliGitAdapter              |
|  src-tauri/src/shell/ (AppShell facade, PtyManager)        |
|  src-tauri/src/commands/ (thin Tauri IPC handlers)         |
+-----------------------------------------------------------+
|              Domain Model (Rust crates)                    |
|  tabby-workspace · tabby-runtime · tabby-settings         |
|  tabby-git                                                 |
|        | depend on                                         |
|  tabby-kernel (shared kernel: value objects, id types)     |
+-----------------------------------------------------------+
|              Transport DTOs (Rust crate)                   |
|  tabby-contracts (DTOs, view models, event structs)        |
|    ^ re-exports VOs from tabby-kernel for IPC compat       |
+-----------------------------------------------------------+
```

## Backend Structure

| Module | Path | Purpose |
|--------|------|---------|
| Application services | `src-tauri/src/application/` | Business logic orchestration |
| Port traits | `src-tauri/src/application/ports.rs` | Abstract interfaces for infrastructure |
| Infrastructure | `src-tauri/src/infrastructure/` | Concrete port implementations |
| IPC commands | `src-tauri/src/commands/` | Thin Tauri command handlers |
| DTO mappers | `src-tauri/src/mapping/` | Transport boundary conversions |
| Shell facade | `src-tauri/src/shell/` | AppShell + PtyManager |
| CLI | `src-tauri/src/cli.rs` | Argument parsing and launch requests |

## Domain Crates

| Crate | Purpose |
|-------|---------|
| `tabby-kernel` | Shared value objects (`PaneId`, `TabId`, `BrowserUrl`, `WorkingDirectory`, etc.) |
| `tabby-workspace` | Workspace aggregate: tabs, panes, split layouts, domain events |
| `tabby-runtime` | Runtime registry: pane runtime lifecycle and status tracking |
| `tabby-settings` | User preferences, terminal profiles, persistence |
| `tabby-git` | Git domain: repository state, commits, branches, diffs, blame, stash |
| `tabby-contracts` | Transport DTOs, view models, and IPC event structs |

## Frontend Structure

| Feature | Path | Purpose |
|---------|------|---------|
| Workspace | `src/features/workspace/` | Tab bar, split-tree renderer, layout presets, wizard |
| Terminal | `src/features/terminal/` | xterm.js rendering, PTY output dispatcher |
| Browser | `src/features/browser/` | Browser pane webview and toolbar |
| Git | `src/features/git/` | Repository status, diff viewer, blame, stash UI |
| Settings | `src/features/settings/` | Settings modal, shortcuts reference |
| Runtime | `src/features/runtime/` | Runtime status tracking store |
| Theme | `src/features/theme/` | Theme selection, editor, presets |
| App Shell | `src/app-shell/` | Transport clients, bootstrap coordinator |

## Dependency Rules

- **Presentation -> Application -> Domain** (never the reverse)
- Domain crates depend on `tabby-kernel` for shared value objects
- Domain crates never depend on `tabby-contracts` (transport layer)
- Domain crates never depend on each other
- Application services depend on port traits, not on concrete infrastructure
- Infrastructure implements port traits; all Tauri-specific code lives in `infrastructure/` or `shell/`
- Frontend features import domain models from their own `domain/` directory, not from other features
- Generated DTOs (`tauri-bindings.ts`) appear only in transport clients and snapshot-mappers

## Key Design Decisions

- **Port-adapter pattern**: application services define abstract ports; infrastructure provides concrete adapters. This keeps business logic testable without Tauri runtime.
- **Single runtime owner**: `RuntimeApplicationService` is the sole owner of runtime lifecycle. All start/stop/replace/restart flows go through it.
- **Workspace owns structure, not runtime**: the workspace aggregate manages tabs, panes, and layout. Runtime state (status, cwd) belongs to the runtime context.
- **Thin IPC handlers**: Tauri commands in `commands/` only deserialize, delegate to a service, and return. No business logic.
- **Projection-based frontend**: the backend publishes complete snapshots via `ProjectionPublisherPort`. Frontend stores consume these snapshots and derive local read models.
