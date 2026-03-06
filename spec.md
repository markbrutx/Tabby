# Tabby — Terminal Workspace

Free, open-source macOS terminal workspace. One app, browser-style tabs, grid of terminals.

## Stack
- **Tauri v2** (Rust + React/TypeScript)
- **xterm.js** with WebGL renderer
- **portable-pty** crate for PTY sessions

## Features

**Tabs** — like Chrome. Each tab is an independent workspace. `Cmd+T` new, `Cmd+W` close, `Cmd+1-9` switch.

**Grid** — each workspace is a grid of terminal panes. Pick on creation: 1x1, 1x2, 2x2, 2x3, 3x3. Panes are resizable by dragging.

**CLI per pane** — choose what runs in each pane: plain Terminal, Claude Code, Codex, or custom command. Changeable at any time.

**Working directory per pane** — each pane can point to a different folder.

**Fullscreen by default** — app launches fullscreen. DMG install, zero config.

**Performance is critical** — input latency < 5ms, tab switch < 50ms, terminals stay alive when switching tabs (no restart).

## Implementation Phases

1. **Core** — Tauri scaffold, single xterm.js pane, PTY via Rust, basic tabs
2. **Grid** — CSS grid layout, resizable panes, multiple PTY sessions
3. **CLI presets** — picker per pane, working directory, pane header badge
4. **Polish** — keyboard shortcuts, settings (font/theme/default CLI), DMG build

## Distribution
MIT license. macOS DMG via Tauri bundler. GitHub Releases + Actions CI.