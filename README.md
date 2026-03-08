# Tabby

Tabby is a macOS-first terminal workspace app built with Tauri, Rust, React, and xterm.js. It gives you browser-style tabs, split-pane layouts, live terminal sessions, and per-pane launch profiles for Terminal, Claude Code, Codex, or custom commands.

## Status

Tabby is an active local-first prototype. The repository already contains the core workspace shell, split layouts, settings, runtime tracking, and browser/terminal pane support, but the architecture is still evolving.

## What It Does

- Browser-style workspace tabs
- Split layouts from `1x1` up to `3x3`
- Independent pane runtime and working directory per pane
- Built-in pane profiles: Terminal, Claude Code, Codex, Custom
- Browser panes alongside terminal panes
- Settings for layout, theme, font size, fullscreen, and startup defaults
- Typed Tauri IPC contracts shared between Rust and TypeScript
- Single-instance CLI routing for opening/focusing the existing app

## Stack

- Tauri v2 desktop shell
- Rust workspace under [`src-tauri/`](src-tauri)
- React 18 + TypeScript frontend under [`src/`](src)
- Bun + Vite toolchain
- xterm.js for terminal rendering
- Zustand for frontend state
- specta / tauri-specta for typed IPC bindings
- Tailwind CSS v4 for styling
- Vitest + Playwright for verification

## Prerequisites

- macOS
- [Bun](https://bun.sh/)
- [Rust](https://rustup.rs/)
- Xcode Command Line Tools

## Quick Start

```bash
bun install

# Full desktop app (Tauri + frontend)
bun run tauri dev
```

Frontend-only mode is also available and uses the web app without real PTY integration:

```bash
bun run dev
```

## Build

```bash
# Frontend bundle only
bun run build

# Desktop bundle via Tauri
bun run tauri build
```

## Single-Instance Safety

Tabby is designed around a single running app instance. Do not start multiple copies of `bun run tauri dev` or `bun run dev` from the same checkout at the same time.

For long-running local sessions, prefer `tmux` so logs stay attached and cleanup is explicit:

```bash
tmux new-session -d -s tabby-dev 'cd /Users/markbrutx/pet/Tabby && bun run tauri dev'
tmux attach -t tabby-dev
tmux kill-session -t tabby-dev
```

## Verification

Frontend:

```bash
bun run lint
bun run typecheck
bun run test
bun run test:e2e
```

Rust:

```bash
cd src-tauri
cargo check --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

## CLI Launch Overrides

When the `tabby` binary is installed or built, it supports launch overrides for the running app:

```bash
tabby --new-tab --layout 2x2 --profile codex --cwd /path/to/project
tabby --new-tab --profile custom --command "npm run dev"
```

Supported flags today are `--new-tab`, `--layout`, `--profile`, `--cwd`, and `--command`.

## Project Layout

- [`src/`](src) - React app shell, stores, and feature UI
- [`src-tauri/src/`](src-tauri/src) - Tauri bootstrap, shell integration, CLI, menu
- [`src-tauri/crates/tabby-workspace/`](src-tauri/crates/tabby-workspace) - workspace and split-layout domain
- [`src-tauri/crates/tabby-runtime/`](src-tauri/crates/tabby-runtime) - pane runtime registry and status
- [`src-tauri/crates/tabby-settings/`](src-tauri/crates/tabby-settings) - preferences and launch profiles
- [`src-tauri/crates/tabby-contracts/`](src-tauri/crates/tabby-contracts) - shared DTOs and IPC contracts
- [`tests/e2e/`](tests/e2e) - Playwright smoke coverage
- [`workbench/`](workbench) - scratch research and reference material, not production source

## Contributing

Before opening a PR or asking an agent to make changes:

- keep diffs minimal and targeted
- run the relevant verification commands for the files you touched
- avoid adding references to private material under `workbench/` in user-facing docs or specs
- preserve the invariant that pane runtimes survive tab switches and layout changes

## Agent Docs

- [`AGENTS.md`](AGENTS.md) - operational instructions for coding agents
- [`CLAUDE.md`](CLAUDE.md) - Claude Code memory and project-specific guidance
- [`spec.md`](spec.md) - product scope and intent

## License

MIT
