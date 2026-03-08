# Tabby

Free, open-source macOS terminal workspace. Browser-style tabs, each containing a grid of terminal panes. Run Terminal, Claude Code, Codex, or custom commands in each pane.

## Prerequisites

- [Bun](https://bun.sh/) — frontend toolchain
- [Rust](https://rustup.rs/) — backend (Tauri v2)

## Quick Start

```bash
# Install dependencies
bun install

# Run full app (Tauri + frontend)
bun run tauri dev

# Run frontend only (mock transport, no real PTY)
bun run dev
```

### Using tmux (optional)

Tabby uses single-instance guards — never run multiple instances simultaneously. tmux lets you run the dev server in the background:

```bash
# Start dev server in a detached tmux session
tmux new-session -d -s tabby 'bun run tauri dev'

# Attach to see logs
tmux attach -t tabby

# Stop
tmux kill-session -t tabby
```

## Build

```bash
# Production build (.dmg)
bun run tauri build
```

## Rust

```bash
cd src-tauri
cargo check
cargo test
cargo clippy
```

## Tests

```bash
# Unit / integration tests
bun run test

# E2E tests (Playwright)
bun run test:e2e
```

## License

MIT
