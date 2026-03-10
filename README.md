# Tabby

[![CI](https://github.com/markbrutx/Tabby/actions/workflows/ci.yml/badge.svg)](https://github.com/markbrutx/Tabby/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![macOS](https://img.shields.io/badge/platform-macOS-lightgrey.svg)]()

**A free, open-source terminal workspace for macOS.**

Tabby gives you browser-style tabs, split-pane layouts, live terminal sessions, and per-pane launch profiles — all in one native app. Press `Cmd+T`, pick a layout, and start working across multiple terminals without losing context.

<div align="center">
  <br />
  <b>Built-in Profiles & Integrations:</b><br /><br />
  <img src="https://img.shields.io/badge/Terminal-2C2C2C?style=for-the-badge&logo=gnu-bash&logoColor=white" alt="Terminal" />
  <img src="https://img.shields.io/badge/Claude_Code-D97757?style=for-the-badge&logo=anthropic&logoColor=white" alt="Claude Code" />
  <img src="https://img.shields.io/badge/Codex-2D7866?style=for-the-badge&logo=openai&logoColor=white" alt="Codex" />
  <img src="https://img.shields.io/badge/Web_Browser-4285F4?style=for-the-badge&logo=safari&logoColor=white" alt="Browser" />
  <img src="https://img.shields.io/badge/Git-F05032?style=for-the-badge&logo=git&logoColor=white" alt="Git" />
  <br /><br />
  <b>Powered by:</b><br /><br />
  <img src="https://img.shields.io/badge/Tauri-FFC131?style=for-the-badge&logo=tauri&logoColor=white" alt="Tauri" />
  <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/React-20232A?style=for-the-badge&logo=react&logoColor=61DAFB" alt="React" />
  <img src="https://img.shields.io/badge/TypeScript-007ACC?style=for-the-badge&logo=typescript&logoColor=white" alt="TypeScript" />
  <img src="https://img.shields.io/badge/Tailwind_CSS-38B2AC?style=for-the-badge&logo=tailwind-css&logoColor=white" alt="Tailwind" />
  <img src="https://img.shields.io/badge/Bun-000000?style=for-the-badge&logo=bun&logoColor=white" alt="Bun" />
  <br /><br />
</div>

## Why Tabby?

- **One window, many terminals** — tabs hold independent workspace layouts, each with its own split configuration
- **No session loss** — switching tabs or resizing panes never kills your running processes
- **Per-pane identity** — each pane has its own working directory, profile, and runtime
- **Built-in profiles** — launch Terminal, Claude Code, Codex, or any custom command per pane
- **Scriptable** — CLI flags let you automate workspace creation from scripts or hotkey daemons

## How It Works

1. **Open a tab** — `Cmd+T` creates a new workspace tab
2. **Pick a layout** — choose from `1x1` up to `3x3` grid presets
3. **Work** — each pane runs an independent PTY session that survives tab switches
4. **Customize** — set profiles, working directories, and themes per pane

## Quick Start

### Prerequisites

- macOS (Apple Silicon or Intel)
- [Bun](https://bun.sh/)
- [Rust](https://rustup.rs/)
- Xcode Command Line Tools (`xcode-select --install`)

### Install & Run

```bash
git clone https://github.com/markbrutx/Tabby.git
cd Tabby
bun install

# Full desktop app with real terminal sessions
bun run tauri dev
```

Frontend-only mode (no Rust, mock transport for UI development):

```bash
bun run dev
```

### Build

```bash
bun run build        # Frontend bundle
bun run tauri build  # macOS .app + .dmg
```

## Features

| Feature | Description |
|---------|-------------|
| **Workspace tabs** | `Cmd+T` / `Cmd+W` / `Cmd+1-9` — each tab is an independent workspace |
| **Split layouts** | Grid presets from 1x1 to 3x3, fully resizable after creation |
| **Persistent sessions** | PTY processes survive tab switches, focus changes, and layout resizing |
| **CLI profiles** | Terminal, Claude Code, Codex, or custom commands per pane |
| **Per-pane cwd** | Each pane tracks its own working directory |
| **Browser panes** | Embed web views alongside terminal panes |
| **Git integration** | Status, branches, commits, diffs, blame, and stash management |
| **Theme system** | Light/dark modes, custom themes, live editor |
| **Keyboard-first** | Full shortcut coverage for tabs, panes, and workspace actions |
| **CLI automation** | `tabby --new-tab --layout 2x2 --profile codex --cwd ~/project` |

## CLI Usage

```bash
# Open a new tab with a 2x2 grid in a specific directory
tabby --new-tab --layout 2x2 --cwd ~/projects/my-app

# Open a tab with Codex profile
tabby --new-tab --profile codex

# Custom command in a new tab
tabby --new-tab --profile custom --command "docker compose up"
```

| Flag | Description |
|------|-------------|
| `--new-tab` | Opens a new tab in the running instance |
| `--layout` | Layout preset: `1x1`, `1x2`, `2x2`, `2x3`, `3x3` |
| `--profile` | Pane profile: `terminal`, `claude-code`, `codex`, `custom` |
| `--cwd` | Working directory for panes |
| `--command` | Custom command (with `--profile custom`) |

## Architecture

Tabby follows a layered architecture with five bounded contexts:

```
Presentation  →  React 18 + TypeScript + Zustand + Tailwind CSS v4
Transport     →  specta + tauri-specta (typed IPC bindings)
Application   →  Rust services with port-adapter pattern
Infrastructure→  Tauri v2 + portable-pty + plugin ecosystem
Domain        →  Pure Rust crates (kernel, workspace, runtime, settings, git)
```

| Crate | Purpose |
|-------|---------|
| `tabby-kernel` | Shared value objects and ID types |
| `tabby-workspace` | Tabs, panes, split layouts, domain events |
| `tabby-runtime` | Pane runtime lifecycle and status tracking |
| `tabby-settings` | User preferences and terminal profiles |
| `tabby-git` | Git operations domain model |
| `tabby-contracts` | Transport DTOs and IPC event structs |

For full architecture docs, see the [documentation site](https://markbrutx.github.io/Tabby/architecture/).

## Verification

```bash
# Frontend
bun run lint          # ESLint + DTO boundary check
bun run typecheck     # TypeScript strict mode
bun run test          # Vitest (500+ tests)
bun run test:e2e      # Playwright E2E

# Rust
cd src-tauri
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace   # 500+ tests

# Everything at once
bun run verify:all
```

## Project Layout

```
src/                          React frontend
  features/                   Feature modules (workspace, terminal, browser, git, settings, theme)
  app-shell/                  Transport clients and bootstrap
  components/                 Shared UI components
src-tauri/                    Rust backend
  src/                        Tauri bootstrap, services, commands, infrastructure
  crates/                     Domain crates (kernel, workspace, runtime, settings, git, contracts)
tests/e2e/                    Playwright E2E tests
docs/                         VitePress documentation site
```

## Contributing

Tabby is **macOS-first**. The architecture is portable — if you're interested in bringing Tabby to Linux or Windows, contributions are welcome.

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines, or visit the [docs site](https://markbrutx.github.io/Tabby/contributing/).

## Acknowledgments

- [Tauri](https://tauri.app/) for the Rust-based desktop framework
- [xterm.js](https://xtermjs.org/) for terminal rendering
- [portable-pty](https://docs.rs/portable-pty) for cross-platform PTY management

## License

[MIT](LICENSE)
