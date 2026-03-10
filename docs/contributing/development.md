# Development Setup

## Prerequisites

| Requirement | Purpose |
|-------------|---------|
| macOS | Primary target platform |
| [Bun](https://bun.sh/) | JavaScript runtime and package manager |
| [Rust](https://rustup.rs/) | Tauri backend compilation |
| Xcode Command Line Tools | macOS system libraries |

## Getting Started

```bash
git clone https://github.com/markbrutx/Tabby.git
cd Tabby
bun install
```

## Development Modes

### Full App (Tauri + Frontend)

```bash
bun run tauri dev
```

This starts both the Vite dev server and the Tauri Rust backend. Terminal panes use real PTY sessions.

### Frontend Only

```bash
bun run dev
```

Runs only the web frontend. A mock transport simulates terminal behavior in the browser, useful for UI work without compiling Rust.

## Available Scripts

| Command | Description |
|---------|-------------|
| `bun run dev` | Frontend dev server (mock transport) |
| `bun run tauri dev` | Full desktop app with hot reload |
| `bun run build` | Frontend production bundle |
| `bun run tauri build` | Desktop app bundle (.app / .dmg) |
| `bun run lint` | ESLint + DTO boundary check |
| `bun run typecheck` | TypeScript strict mode check |
| `bun run test` | Vitest unit/integration tests |
| `bun run test:e2e` | Playwright E2E tests |
| `bun run test:rust` | Rust fmt + clippy + tests |
| `bun run verify` | Frontend lint + typecheck + tests |
| `bun run verify:all` | Full verification (frontend + Rust) |
| `bun run docs:dev` | Documentation site dev server |
| `bun run docs:build` | Build documentation site |

## Rust Development

The Rust workspace lives under `src-tauri/`:

```bash
cd src-tauri

# Check compilation
cargo check --workspace

# Format code
cargo fmt --all

# Lint
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run tests
cargo test --workspace
```

## Testing

### Unit & Integration Tests

```bash
bun run test           # Run once
bun run test:watch     # Watch mode
```

### E2E Tests

```bash
bun run test:e2e           # Headless
bun run test:e2e:headed    # With browser visible
```

### Rust Tests

```bash
cd src-tauri && cargo test --workspace
```

## Tips

- Use `tmux` for long-running dev sessions to avoid orphaned processes
- Never run multiple Tabby instances simultaneously
- The `bun run dev` mode is the fastest iteration loop for UI changes
- Rust changes require `bun run tauri dev` to take effect
