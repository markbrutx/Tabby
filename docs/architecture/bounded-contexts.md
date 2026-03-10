# Bounded Contexts

Tabby is organized into five bounded contexts. Each context owns its domain model and exposes it only through application services.

## Context Map

| Context | Domain Crate | Application Service | Frontend Feature |
|---------|-------------|---------------------|-----------------|
| Workspace | `tabby-workspace` | `WorkspaceApplicationService` | `features/workspace/` |
| Runtime | `tabby-runtime` | `RuntimeApplicationService`, `RuntimeCoordinator` | `features/runtime/` |
| Settings | `tabby-settings` | `SettingsApplicationService` | `features/settings/` |
| Git | `tabby-git` | `GitApplicationService` | `features/git/` |
| Shell/Transport | (infrastructure) | `AppShell`, `BootstrapService` | `app-shell/` |

## Port Traits

Application services depend on abstract port traits defined in `src-tauri/src/application/ports.rs`. Infrastructure adapters implement these traits.

| Port | Defined In | Implemented By |
|------|-----------|---------------|
| `PreferencesRepository` | `application/ports.rs` | `TauriStorePreferencesRepository` |
| `ProjectionPublisherPort` | `application/ports.rs` | `TauriProjectionPublisher` |
| `TerminalProcessPort` | `application/ports.rs` | `PtyManager` |
| `BrowserSurfacePort` | `application/ports.rs` | `TauriBrowserSurfaceAdapter` |
| `GitOperationsPort` | `application/ports.rs` | `CliGitAdapter` |

## Workspace Context

Owns structural concerns: tabs, panes, split layouts, and layout presets.

**Does not own**: runtime state (status, cwd, process lifecycle). This belongs to the Runtime context.

Key types: `WorkspaceSession`, `Tab`, `PaneSlot`, `SplitNode`, `PaneSpec`, `PaneContentDefinition`

Domain events include: `PaneAdded`, `PaneRemoved`, `ActivePaneChanged`, `ActiveTabChanged`, `PaneContentChanged`

## Runtime Context

Owns pane runtime lifecycle: registration, status tracking, CWD observation, and process control.

`RuntimeCoordinator` reacts to workspace domain events to start/stop runtimes when panes are added or removed.

Key types: `RuntimeRegistry`, `PaneRuntime`, `RuntimeStatus`, `RuntimeKind`, `RuntimeSessionId`

## Settings Context

Owns user preferences: font size, default layout, default profile, working directory, terminal profiles.

Persistence uses `tauri-plugin-store` for JSON-backed storage, with the `PreferencesRepository` port abstracting the storage mechanism.

Key types: `UserPreferences`, `TerminalProfile`, `ProfileCatalog`, `FontSize`, `ProfileId`

## Git Context

Owns Git repository queries. All operations are stateless request/response -- each query spawns a short-lived CLI call via `GitOperationsPort`.

Key types: `GitRepositoryState`, `CommitInfo`, `BranchInfo`, `FileStatus`, `DiffContent`, `DiffHunk`, `BlameEntry`, `StashEntry`

## Shell/Transport Context

Infrastructure-level context that wires everything together:

- **AppShell** -- facade coordinating all services and handling cross-context side effects
- **BootstrapService** -- processes CLI arguments and constructs initial state
- **PtyManager** -- implements `TerminalProcessPort` for PTY process lifecycle
- **Transport clients** -- frontend `createTauriShellClients` factory producing typed IPC clients

## Cross-Context Rules

- Services do not reach into each other directly
- Cross-context side effects are coordinated by `AppShell`
- Frontend features do not import models from other features
- `RuntimeObservationReceiver` is the callback port connecting infrastructure observations (PTY exit, CWD change) back to `RuntimeApplicationService`
