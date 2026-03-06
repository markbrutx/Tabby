# Tabby — Terminal Workspace

Free, open-source macOS terminal workspace. One app, browser-style tabs, grid of
live terminals.

Tabby uses a macOS-first Tauri desktop architecture with a manager-based Rust
backend, React/TypeScript frontend, store-backed settings, typed IPC bindings,
and a terminal-first UI built around PTY-backed workspaces.

## Stack
- **Platform baseline** — **Tauri v2** desktop shell with a **Rust** backend
  and **React 18 + TypeScript** frontend, built with **Vite 6** and **Bun**.
- **Frontend architecture** — **Zustand** for app state, **Immer** for complex
  immutable updates, **Tailwind CSS v4** for styling, **i18next /
  react-i18next** for localization, and lightweight UI utilities such as
  **sonner** and **lucide-react**.
- **Type-safe IPC** — **specta + tauri-specta** for generated TypeScript
  bindings and thin Tauri command handlers.
- **Terminal runtime** — **xterm.js** for rendering and interaction in the UI;
  **portable-pty** in Rust for PTY/session lifecycle, shell spawning,
  stdin/stdout streaming, resize, and teardown.
- **Tauri plugin baseline** — use plugins such as `log`, `store`, `os`, `fs`,
  `process`, `dialog`, `opener`, `clipboard-manager`, `global-shortcut`,
  `autostart`, `updater`, and `single-instance` where they make sense for
  Tabby.
- **Persistence and settings** — `tauri-plugin-store` for JSON-backed app
  settings, mirrored by frontend stores/hooks.
- **Quality and tooling** — strict TypeScript, ESLint, Prettier, Playwright for
  smoke/E2E coverage, Rust fmt/clippy/tests, and GitHub Actions for build and
  release automation.
- **Shipping target** — macOS-first signed Tauri bundle with DMG distribution;
  architecture should remain portable even if the initial product scope is
  macOS only.

## Features

**Workspace tabs** — browser-style tabs where each tab is an independent
terminal workspace. `Cmd+T` creates a tab, `Cmd+W` closes it, `Cmd+1-9`
switches directly.

**Grid layouts** — each workspace starts from a layout preset (`1x1`, `1x2`,
`2x2`, `2x3`, `3x3`) and panes remain resizable after creation.

**Persistent live terminals** — every pane owns an independent PTY session;
switching tabs, changing focus, or resizing layout must not restart the running
process.

**CLI profile per pane** — a pane can launch plain Terminal, Claude Code,
Codex, or a custom command/profile. The profile is editable without recreating
the whole workspace.

**Working directory per pane** — each pane can start in and stay associated
with its own project folder.

**Pane chrome and status** — each pane shows enough metadata to understand what
is running, where it is running, and which pane is active.

**Keyboard-first workflow** — fast shortcuts for tab creation/switching, pane
focus, pane closing, and core workspace actions.

**Settings-driven defaults** — configurable defaults for shell/profile, startup
layout, font/theme, shortcut mapping, launch behavior, and other workspace
preferences.

**CLI automation and single-instance behavior** — CLI entry points should allow
opening/focusing Tabby, creating tabs, selecting panes, and launching presets
from scripts or hotkey daemons.

**Fullscreen-first launch** — app opens fullscreen by default on macOS while
remaining configurable later.

**Performance constraints** — input latency target `< 5ms`, tab switch target
`< 50ms`, and no PTY/session loss during normal UI navigation.

**Simple shipping model** — zero required cloud services, DMG install, and a
low-friction local-first setup.

## Implementation Phases

1. **Platform Foundation** — establish Bun/Vite/Tauri setup, Rust app
   bootstrap, specta bindings, logging, settings persistence, single-instance
   wiring, and macOS bundle/distribution scaffolding.
2. **Terminal Session Core** — implement PTY/session managers, terminal
   coordinator/event flow, xterm integration, shell spawn/write/read/resize/kill
   operations, and the invariant that sessions stay alive independently from the
   visible UI.
3. **Workspace Model and Layout UI** — add tab state, pane identity, layout
   presets, resizing, active-pane management, pane headers, and the
   React/Zustand model that maps UI state to backend PTY state.
4. **Profiles, Settings, and Automation** — add per-pane CLI profiles,
   working-directory selection, default settings, shortcut configuration,
   store-backed settings screens, and CLI actions for automating Tabby from
   outside the app.
5. **Hardening and Release** — cover critical paths with targeted Rust tests and
   Playwright smoke tests, tune performance, remove lifecycle bugs, polish UX,
   and ship signed DMG builds through CI/CD.

## Distribution
MIT license. macOS DMG via Tauri bundler. GitHub Releases + Actions CI.
