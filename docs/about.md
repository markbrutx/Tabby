# About Tabby

## Vision

Tabby exists to solve a simple problem: developers juggle too many terminal windows. Instead of spreading terminals across monitors, relying on tmux for session persistence, and context-switching between apps, Tabby gives you one native workspace with tabs, split panes, and live sessions that never die.

The long-term goal is a scriptable, profile-driven terminal workspace that adapts to how you work — whether that's a solo developer with three projects, a team lead reviewing PRs, or an AI-assisted workflow with Claude Code running alongside your shell.

## What Makes Tabby Different?

| | Terminal.app | iTerm2 | Warp | **Tabby** |
|---|---|---|---|---|
| Tabs + split panes | Basic | Yes | Yes | **Yes** |
| Sessions survive tab switch | No | Partial | Yes | **Always** |
| Per-pane profiles (Claude, Codex) | No | No | No | **Yes** |
| Browser panes alongside terminals | No | No | No | **Yes** |
| Built-in Git UI | No | No | Partial | **Yes** |
| CLI automation | No | Limited | No | **Yes** |
| Open source | No | Yes | No | **Yes** |
| Native performance (Rust + Tauri) | Native | Native | Rust | **Rust + Tauri** |

Tabby isn't trying to replace your shell or reinvent the terminal. It's a workspace layer on top of real PTY sessions that keeps everything organized, persistent, and scriptable.

## Current Status

**v0.1.0** — First public release. The core workspace is functional and usable for daily development.

### What Works

- **Workspace shell** — browser-style tabs with independent layouts and sessions
- **Terminal sessions** — real PTY processes via portable-pty, persistent across all UI interactions
- **Split layouts** — grid presets (1x1 through 3x3) with resizable panes
- **CLI profiles** — Terminal, Claude Code, Codex, and custom command profiles per pane
- **Browser panes** — embedded web views alongside terminals
- **Settings** — font size, default layout, profile, working directory, preferences persistence
- **Git integration** — repository status, branches, commit history, diffs (unified + split), blame, stash
- **Theme system** — light/dark modes, custom color tokens, import/export, live preview
- **CLI automation** — launch overrides via command-line flags with single-instance routing

### What's Experimental

- Git integration is read-only (no commit, push, pull yet)
- Browser panes have basic navigation (no DevTools, extensions)
- Theme import/export is manual (file-based)

## Roadmap

### v0.2 — Distribution & Reliability

- CI/CD pipeline with GitHub Actions for automated builds and tests
- Signed and notarized `.dmg` distribution (no more Gatekeeper bypass)
- Auto-update mechanism
- Expanded test coverage (unit, integration, E2E)

### v0.3 — Session & Workspace Persistence

- Session restore on relaunch — reopen tabs, panes, and running commands after quitting
- Tab drag-and-drop reordering
- Workspace save/load — export and import full workspace configurations
- Git write operations (commit, push, pull from the UI)

### Future

- Plugin system for extending functionality
- Custom keybinding configuration
- Community theme sharing
- Linux and Windows support
- Search across terminal output history
- Split pane zoom (temporarily maximize a single pane)

## Tech Stack

| Component | Technology | Role |
|-----------|-----------|------|
| Desktop shell | Tauri v2 | Native macOS app container |
| Backend | Rust | PTY management, Git operations, state |
| Frontend | React 18 + TypeScript | UI rendering and interaction |
| Build | Bun + Vite 6 | Fast bundling and dev server |
| Terminal | xterm.js + portable-pty | Terminal rendering + real PTY sessions |
| State | Zustand | Frontend state management |
| Styling | Tailwind CSS v4 | Utility-first CSS |
| IPC | specta + tauri-specta | Type-safe Rust ↔ TypeScript communication |
| Testing | Vitest + Playwright | Unit, integration, and E2E tests |

## Links

- [GitHub Repository](https://github.com/markbrutx/Tabby)
- [Releases & Downloads](https://github.com/markbrutx/Tabby/releases)
- [Architecture Documentation](/architecture/)
- [Contributing Guide](/contributing/)

## License

MIT — see [LICENSE](https://github.com/markbrutx/Tabby/blob/master/LICENSE) for details.
