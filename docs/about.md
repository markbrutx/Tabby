# About Tabby

## Status

Tabby is an active local-first prototype. The core workspace shell, split layouts, settings, runtime tracking, terminal and browser pane support, Git integration, and theme system are functional. The architecture continues to evolve.

## What's Done

- Platform foundation (Tauri v2, Rust workspace, React frontend)
- Terminal session core with PTY lifecycle management
- Workspace UI with tabs, split-pane layouts, and presets
- Settings persistence and terminal profiles
- Git integration (status, branches, commits, diffs, blame, stash)
- Theme system with light/dark modes and custom themes
- CLI launch overrides

## What's Next

- CI/CD pipeline with GitHub Actions
- Signed DMG distribution
- Performance tuning and hardening
- Expanded test coverage

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Desktop shell | Tauri v2 |
| Backend | Rust |
| Frontend | React 18, TypeScript |
| Build | Bun, Vite 6 |
| Terminal | xterm.js, portable-pty |
| State | Zustand |
| Styling | Tailwind CSS v4 |
| IPC | specta, tauri-specta |
| Testing | Vitest, Playwright |

## Links

- [GitHub Repository](https://github.com/markbrutx/Tabby)
- [Architecture](/architecture/)
- [Contributing](/contributing/)

## License

MIT
