# CLAUDE.md

Root memory for Claude Code in this repository. Keep this file high-signal. If it starts growing, move path-specific guidance into `.claude/rules/` and keep task-specific workflows in `.claude/skills/`.

## Project Snapshot

Tabby is a macOS-first terminal workspace app. It combines browser-style tabs, split-pane layouts, terminal runtimes, browser panes, per-pane working directories, and launch profiles for Terminal, Claude Code, Codex, or custom commands.

Core stack:

- Tauri v2 desktop shell
- Rust workspace under `src-tauri/`
- React 18 + TypeScript frontend under `src/`
- Bun + Vite toolchain
- xterm.js for terminal rendering
- Zustand for frontend state
- specta / tauri-specta for typed IPC contracts
- Tailwind CSS v4 for styling

## Architecture Map

The codebase follows a layered architecture with four bounded contexts: **Workspace**, **Runtime**, **Settings**, and **Shell/Transport**. Application services depend on abstract port traits; infrastructure adapters implement those ports.

```
┌─────────────────────────────────────────────────────────┐
│                    Presentation (src/)                   │
│  features/{workspace,terminal,browser,settings,runtime} │
│  app-shell/  ·  components/  ·  hooks/                  │
├─────────────────────────────────────────────────────────┤
│              Transport Boundary (IPC)                    │
│  contracts/tauri-bindings.ts  ↔  tabby-contracts crate  │
│  app-shell/clients/ (createTauriShellClients)            │
│  src-tauri/src/mapping/ (DTO ↔ domain mappers)          │
├─────────────────────────────────────────────────────────┤
│              Application Services (Rust)                 │
│  src-tauri/src/application/                              │
│    workspace_service · settings_service · runtime_service│
│    runtime_coordinator · bootstrap_service               │
│    ports (4 trait definitions)                            │
│    runtime_observation_receiver (callback port)          │
│    commands (internal command enums)                      │
├─────────────────────────────────────────────────────────┤
│              Infrastructure (Rust)                       │
│  src-tauri/src/infrastructure/                           │
│    TauriProjectionPublisher · TauriStorePreferencesRepo  │
│    TauriBrowserSurfaceAdapter                            │
│  src-tauri/src/shell/ (AppShell facade, PtyManager)      │
│  src-tauri/src/commands/ (thin Tauri IPC handlers)       │
├─────────────────────────────────────────────────────────┤
│              Domain Model (Rust crates)                  │
│  tabby-workspace · tabby-runtime · tabby-settings       │
│  tabby-contracts (shared DTOs and event structs)         │
└─────────────────────────────────────────────────────────┘
```

### Backend (Rust)

- `src-tauri/src/lib.rs` — bootstraps Tauri, tracing, specta exports, single-instance plugin, and `AppShell`.
- `src-tauri/src/application/` — application services and port definitions:
  - `workspace_service.rs` — `WorkspaceApplicationService` (workspace aggregate operations)
  - `settings_service.rs` — `SettingsApplicationService` (preferences load/save)
  - `runtime_service.rs` — `RuntimeApplicationService` (runtime registry management)
  - `runtime_coordinator.rs` — `RuntimeCoordinator` (reacts to workspace domain events for runtime lifecycle)
  - `bootstrap_service.rs` — `BootstrapService` (CLI launch overrides, initial state)
  - `ports.rs` — four port traits: `PreferencesRepository`, `ProjectionPublisherPort`, `TerminalProcessPort`, `BrowserSurfacePort`
  - `runtime_observation_receiver.rs` — callback port for PTY output and terminal exit events
  - `commands.rs` — internal command enums (`WorkspaceCommand`, `SettingsCommand`, `RuntimeCommand`)
- `src-tauri/src/infrastructure/` — concrete port implementations:
  - `tauri_projection_publisher.rs` — implements `ProjectionPublisherPort` (emits events to frontend)
  - `tauri_store_preferences_repository.rs` — implements `PreferencesRepository` (Tauri plugin-store)
  - `tauri_browser_surface_adapter.rs` — implements `BrowserSurfacePort` (webview lifecycle)
- `src-tauri/src/commands/` — thin Tauri IPC command handlers that delegate to application services.
- `src-tauri/src/mapping/` — DTO-to-domain and domain-to-DTO mappers at the transport boundary.
- `src-tauri/src/shell/` — `AppShell` facade coordinating all services; `PtyManager` (implements `TerminalProcessPort`).
- `src-tauri/src/cli.rs` — CLI argument parsing and launch request types.
- `src-tauri/src/menu.rs` — macOS menu bar setup and event handling.

### Domain Crates (Rust)

- `src-tauri/crates/tabby-workspace/` — Workspace context: `WorkspaceSession`, `Tab`, `PaneSlot`, `SplitNode`, `PaneSpec` (Terminal | Browser), `PaneContentDefinition`, layout presets, domain events.
- `src-tauri/crates/tabby-runtime/` — Runtime context: `RuntimeRegistry`, `PaneRuntime`, `RuntimeStatus`, `RuntimeKind`, `RuntimeSessionId`.
- `src-tauri/crates/tabby-settings/` — Settings context: `UserPreferences`, `TerminalProfile`, `ProfileCatalog`, value objects (`FontSize`, `ProfileId`, `WorkingDirectory`), persistence helpers.
- `src-tauri/crates/tabby-contracts/` — shared DTOs, view models, command enums, event structs, and value objects consumed by both Rust and TypeScript via specta.

### Frontend (TypeScript/React)

- `src/app-shell/` — transport infrastructure: `createTauriShellClients` factory producing `WorkspaceClient`, `SettingsClient`, `RuntimeClient`; `AppShellContext` provider; `AppBootstrapCoordinator` for initial load orchestration.
- `src/features/workspace/` — workspace UI: tab bar, split-tree renderer, setup wizard, pane layout. Contains `domain/`, `application/` (store, snapshot-mappers), `components/`, `hooks/`, `model/`, and utility modules (`selectors`, `layoutReadModel`, `theme`).
- `src/features/terminal/` — terminal pane rendering via xterm.js, PTY output dispatcher (`ptyOutputDispatcher.ts`).
- `src/features/browser/` — browser pane webview container and toolbar.
- `src/features/settings/` — settings modal, shortcuts reference. Contains `domain/`, `application/` (store, snapshot-mappers), and `components/`.
- `src/features/runtime/` — runtime status tracking store and domain models. Contains `domain/` and `application/` (store, snapshot-mappers).
- `src/contexts/` — app-level Zustand store factories (`useWorkspaceStore`, `useSettingsStore`, `useRuntimeStore`).
- `src/contracts/tauri-bindings.ts` — auto-generated TypeScript bindings from specta (do not edit).
- `src/components/` — shared UI components (`Button`, `Input`, `Select`, `ErrorBoundary`, `RecoveryScreen`).

### Bounded Contexts & Dependency Rules

Each bounded context owns its domain model and exposes it only through application services:

| Context | Domain Crate | Application Service | Frontend Feature |
|---------|-------------|--------------------|-----------------|
| Workspace | `tabby-workspace` | `WorkspaceApplicationService` | `features/workspace/` |
| Runtime | `tabby-runtime` | `RuntimeApplicationService`, `RuntimeCoordinator` | `features/runtime/` |
| Settings | `tabby-settings` | `SettingsApplicationService` | `features/settings/` |
| Shell/Transport | — (infrastructure) | `AppShell`, `BootstrapService` | `app-shell/` |

**Port traits and implementations:**

| Port | Defined in | Implemented by |
|------|-----------|---------------|
| `PreferencesRepository` | `application/ports.rs` | `TauriStorePreferencesRepository` |
| `ProjectionPublisherPort` | `application/ports.rs` | `TauriProjectionPublisher` |
| `TerminalProcessPort` | `application/ports.rs` | `PtyManager` (in `shell/`) |
| `BrowserSurfacePort` | `application/ports.rs` | `TauriBrowserSurfaceAdapter` |
| `RuntimeObservationReceiver` | `application/runtime_observation_receiver.rs` | `RuntimeApplicationService` |

**Allowed dependency directions:**
- Presentation → Application → Domain (never the reverse)
- `commands/` → `application/` → domain crates (thin handlers only)
- `mapping/` sits at the transport boundary; maps between `tabby-contracts` DTOs and domain types
- Application services depend on port traits, not on concrete infrastructure
- `infrastructure/` implements port traits; depends on Tauri plugins and external crates
- `shell/` provides `AppShell` facade and `PtyManager`; wires ports to services
- Domain crates must not depend on each other (except through `tabby-contracts` for shared DTOs)
- Frontend features import domain models from their own `domain/` directory, not from other features
- Generated DTOs (`tauri-bindings.ts`) appear only in transport clients and snapshot-mappers, never in stores or domain models

## Invariants

- Terminal runtimes must survive tab switches, focus changes, and layout updates.
- Each pane owns one independent runtime and one working directory.
- Single-instance routing must apply launch overrides to the existing app window instead of creating duplicate instances.
- Keep terminal and browser runtime behavior explicit rather than collapsing them into loosely typed state.
- Keep Tauri commands thin and move behavior into application services or domain code.
- Application services depend on port traits (`ports.rs`), never on concrete Tauri or plugin imports.
- Infrastructure adapters (`infrastructure/`) implement port traits; all Tauri-specific code lives here or in `shell/`.
- Runtime lifecycle has a single owner: `RuntimeApplicationService`. All start/stop/replace/restart/exit flows go through it.
- Workspace aggregate owns only structural concerns (tabs, panes, layout). Runtime-observed state (cwd, status) belongs to the runtime context.
- Frontend stores use internal read models; generated DTOs (`tauri-bindings.ts`) exist only at the transport boundary (clients and snapshot-mappers).
- Frontend cross-context orchestration is handled by `AppBootstrapCoordinator`, not by stores reaching into each other.
- Never cite or mention private reference material under `workbench/` in user-facing docs, specs, PR text, or release notes.

## Commands

```bash
bun install
bun run tauri dev
bun run dev
bun run build
bun run tauri build
bun run lint
bun run typecheck
bun run test
bun run test:e2e

cd src-tauri
cargo check --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

## Working Agreement

- Inspect code first; do not rely on outdated assumptions from earlier architecture sketches.
- Prefer local project commands, agents, and skills over user-global wrappers.
- Prefer TDD for bugs and new behavior.
- After frontend edits, run the smallest relevant subset of lint, typecheck, unit tests, and E2E checks.
- After Rust edits, run fmt, clippy, and relevant cargo tests.
- Use `tracing` for backend diagnostics.
- Do not introduce `.unwrap()` or `.expect()` in production Rust.
- Use `tmux` for long-running dev servers or watch tasks.

## Local Claude Assets

- `.claude/commands/` contains project-local slash commands such as `/plan`, `/tdd`, `/verify`, and `/code-review`.
- `.claude/agents/` contains reusable subagent prompts such as planner, architect, reviewer, and security reviewer.
- `.claude/skills/` contains task-specific workflows that should stay out of always-loaded memory.
- `.claude/hooks/hooks.json` records hook intent; if the runtime does not support Claude hooks, enforce the same checks manually.
- `CLAUDE.local.md` is the right place for uncommitted machine- or user-specific notes.

## Ralph — Autonomous Agent Loop

Ralph is a file-based agent loop that autonomously picks stories from a PRD, runs an AI agent to implement them, commits results, and moves to the next story. Config and state are gitignored.

- `.agents/ralph/config.sh` — main config (agent, iterations, stale timeout)
- `.agents/ralph/agents.sh` — agent runner commands (Claude default)
- `.agents/ralph/PROMPT_build.md` — prompt template injected each iteration
- `.agents/tasks/*.json` — PRD files (one per task/project)
- `.ralph/` — runtime state (progress, guardrails, activity log, run logs)
- `workbench/cmd/ralph-*.sh` — convenience launch scripts

Usage:
```bash
./workbench/cmd/ralph-arch-refactor.sh 50        # run architecture refactor
./workbench/cmd/ralph-run.sh 5 prd-my-task       # run any PRD
ralph build 1 --no-commit --prd .agents/tasks/prd-arch-refactor.json  # dry run
ralph ping --agent=claude                        # health check
```

Ralph reads `AGENTS.md` and `.ralph/guardrails.md` before each iteration. It logs progress to `.ralph/progress.md` and learns from failures via guardrails ("signs").

## Related Docs

- [`README.md`](README.md) for the human-facing overview and setup
- [`AGENTS.md`](AGENTS.md) for the agent-agnostic operating guide
- [`spec.md`](spec.md) for product direction and scope
