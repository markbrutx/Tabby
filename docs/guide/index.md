# Getting Started

Tabby is a free, open-source macOS terminal workspace app. It gives you browser-style tabs, split-pane layouts, live terminal sessions, and per-pane launch profiles for Terminal, Claude Code, Codex, or custom commands.

## Why Tabby?

Modern development involves juggling multiple terminals across different projects, directories, and tools. Tabby organizes this chaos into a single workspace:

- **One window, many terminals** -- tabs hold independent workspace layouts
- **No session loss** -- switching tabs or resizing panes never kills your running processes
- **Per-pane identity** -- each pane has its own working directory, profile, and runtime
- **Scriptable** -- CLI flags let you automate workspace creation from scripts or hotkey daemons

## Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri v2 |
| Backend | Rust |
| Frontend | React 18 + TypeScript |
| Build tools | Bun + Vite 6 |
| Terminal | xterm.js + portable-pty |
| State | Zustand |
| Styling | Tailwind CSS v4 |
| IPC | specta + tauri-specta |
| Testing | Vitest + Playwright |

## Next Steps

- [Installation](/guide/installation) -- prerequisites and quick start
- [Features](/guide/features) -- full feature walkthrough
- [CLI Usage](/guide/cli) -- automate Tabby from the command line
