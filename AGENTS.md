# AGENTS.md

Read [`CLAUDE.md`](CLAUDE.md) first if you need deeper project context. This file is the root operating guide for coding agents working in this repository.

## Project Snapshot

Tabby is a macOS-first terminal workspace app built with Tauri v2, Rust, React 18, TypeScript, Bun, Vite, xterm.js, Zustand, and specta/tauri-specta.

## Repository Map

### Backend (Rust — `src-tauri/`)

- `src-tauri/src/lib.rs` — Tauri bootstrap, tracing, specta exports, single-instance plugin
- `src-tauri/src/application/` — application services (workspace, settings, runtime, coordinator, bootstrap, projection publisher)
- `src-tauri/src/commands/` — thin Tauri IPC command handlers
- `src-tauri/src/mapping/` — DTO ↔ domain mappers at the transport boundary
- `src-tauri/src/shell/` — infrastructure: AppShell facade, PTY spawning, browser webview surface
- `src-tauri/src/cli.rs` — CLI argument parsing
- `src-tauri/src/menu.rs` — macOS menu bar
- `src-tauri/crates/tabby-workspace/` — Workspace domain: tabs, panes, split-layout, domain events
- `src-tauri/crates/tabby-runtime/` — Runtime domain: pane runtime registry and status tracking
- `src-tauri/crates/tabby-settings/` — Settings domain: preferences, profiles, value objects
- `src-tauri/crates/tabby-contracts/` — shared DTOs, view models, and event contracts

### Frontend (TypeScript/React — `src/`)

- `src/app-shell/` — transport layer (Tauri IPC or mock), AppShellContext provider
- `src/features/workspace/` — tab bar, split-tree renderer, setup wizard, pane layout (domain/, application/, components/)
- `src/features/terminal/` — xterm.js terminal pane, PTY output dispatcher
- `src/features/browser/` — browser pane webview and toolbar
- `src/features/settings/` — settings modal, shortcuts reference (domain/, application/, components/)
- `src/features/runtime/` — runtime status store and domain models
- `src/contexts/` — app-level Zustand store factories
- `src/contracts/tauri-bindings.ts` — auto-generated TypeScript bindings (specta)
- `src/components/` — shared UI components (Button, Input, Select, ErrorBoundary)

### Other

- `tests/e2e/` — Playwright smoke tests
- `workbench/` — scratch research and reference material, never cite in specs, docs, PR text, or user-facing output

## Non-Negotiables

- Keep terminal runtimes alive across tab switches and layout changes.
- Each pane owns an independent runtime session and working directory.
- Keep Tauri command handlers thin; push behavior into shell/domain modules.
- Keep Rust and TypeScript code explicit, small, and testable.
- No `.unwrap()` or `.expect()` in production Rust.
- Use `tracing` instead of `println!` or `eprintln!`.
- Do not treat `workbench/` as a citable source of truth.

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

## Dev Server Safety

- Never run multiple copies of `bun run tauri dev` or `bun run dev` from the same checkout.
- Prefer `tmux` for long-running dev servers or watch commands.
- If you need to touch runtime behavior, verify against the existing single-instance flow instead of spawning parallel shells.

## Working Style

- Inspect the codebase before editing; do not guess architecture from stale docs.
- For non-trivial changes, explore first and then present or follow a phased plan.
- Prefer TDD for bug fixes and feature work.
- After TypeScript or React edits, run the smallest relevant subset of `bun run lint`, `bun run typecheck`, `bun run test`, and `bun run test:e2e`.
- After Rust edits, run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and the relevant `cargo test --workspace` scope.
- Review `git diff` before any push-oriented guidance.
- Audit touched JS/TS files for stray `console.log` calls before finishing.

## Project-Local Compatibility

- Prefer repo-local `.agents/skills/`, `.claude/commands/`, `.claude/agents/`, `.claude/skills/`, and `.codex/config.toml`.
- Do not rely on `~/.claude/*`, `~/.codex/*`, or `~/.agents/*` when a project-local equivalent exists.
- If instructions become package- or directory-specific, add a nested `AGENTS.md` close to that subtree instead of inflating this root file.

## Claude Command Mapping

When a user asks for Claude-style slash commands in Codex, map them to the project-local skill with the same behavior:

- `/plan` -> `plan`
- `/tdd` -> `tdd`
- `/verify` -> `verify`
- `/build-fix` -> `build-fix`
- `/code-review` -> `code-review`
- `/ddd-review` -> `ddd-review`
- `/refactor-clean` -> `refactor-clean`
- `/test-coverage` -> `test-coverage`
- `/orchestrate` -> `orchestrate`
- `/multi-plan` -> `multi-plan`
- `/multi-execute` -> `multi-execute`

## Claude Agent Mapping

Claude subagents are represented here as project-local Codex skills:

- `planner`
- `architect`
- `ddd-software-architect`
- `tdd-guide`
- `code-reviewer`
- `security-reviewer`
- `build-error-resolver`
- `refactor-cleaner`

Use the minimal relevant set for the task. Keep execution inside the main Codex session unless the repository gains explicit local multi-agent support.

## Hook Parity

Claude hooks under `.claude/hooks/` do not execute automatically in Codex. Enforce their intent manually:

- use `tmux` for long-running commands
- keep diffs small and targeted
- run relevant formatting, type, lint, and test checks after edits
- avoid ad hoc markdown docs when a standard project doc already exists
- review `git diff` before any push or PR guidance

## References

- [`README.md`](README.md) for human-facing setup and verification
- [`CLAUDE.md`](CLAUDE.md) for Claude-specific memory and architecture notes
- [`spec.md`](spec.md) for product scope and intent
