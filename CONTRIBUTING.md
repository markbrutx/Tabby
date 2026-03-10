# Contributing to Tabby

Tabby is a **macOS-first** terminal workspace app built with Tauri v2, Rust, and React. The architecture is designed to be portable — if you're interested in bringing Tabby to **Linux or Windows**, we'd love your help. Check the [architecture docs](https://markbrutx.github.io/Tabby/architecture/) to understand the platform abstraction layer.

## Getting Started

### Prerequisites

| Requirement | Install |
|-------------|---------|
| macOS | Apple Silicon or Intel |
| Bun | [bun.sh](https://bun.sh/) |
| Rust | [rustup.rs](https://rustup.rs/) |
| Xcode CLI Tools | `xcode-select --install` |

### Setup

```bash
git clone https://github.com/markbrutx/Tabby.git
cd Tabby
bun install
```

### Development

```bash
# Full desktop app with real PTY sessions
bun run tauri dev

# Frontend only — mock transport, no Rust compilation needed
bun run dev
```

> **Tip:** Use `bun run dev` for fast UI iteration. Switch to `bun run tauri dev` when you need real terminal sessions.

## Before Submitting a PR

### Run Verification

```bash
# Quick check (frontend only)
bun run verify

# Full check (frontend + Rust)
bun run verify:all
```

Or run individual checks:

| Command | What it checks |
|---------|---------------|
| `bun run lint` | ESLint + DTO boundary |
| `bun run typecheck` | TypeScript strict mode |
| `bun run test` | Vitest unit/integration tests |
| `bun run test:e2e` | Playwright E2E tests |
| `cd src-tauri && cargo fmt --all --check` | Rust formatting |
| `cd src-tauri && cargo clippy --workspace` | Rust lints |
| `cd src-tauri && cargo test --workspace` | Rust tests |

### CI

All PRs are automatically checked by GitHub Actions. The CI runs:
- Frontend: lint, typecheck, vitest
- Rust: fmt, clippy, cargo test

## Guidelines

### Code

- **Keep diffs minimal** — a bug fix doesn't need surrounding code cleaned up
- **Prefer TDD** — write the test first, make it pass, then refactor
- **No `.unwrap()` in Rust** — handle errors explicitly
- **No `console.log` in TypeScript** — use proper error handling
- **Conventional commits** — `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`

### Architecture Rules

These invariants must never be broken:

1. Terminal runtimes **survive** tab switches, focus changes, and layout updates
2. Each pane owns **one** independent runtime and **one** working directory
3. Application services depend on **port traits**, never on concrete infrastructure
4. Generated DTOs appear **only** in transport clients and snapshot-mappers
5. Domain crates depend **only** on `tabby-kernel`, never on each other

### Project Structure

```
src/                          React frontend
  features/                   Feature modules (workspace, terminal, browser, git, settings, theme)
  app-shell/                  Transport clients and bootstrap coordinator
src-tauri/                    Rust backend
  src/application/            Application services and port definitions
  src/infrastructure/         Concrete port implementations
  src/commands/               Thin Tauri IPC handlers
  crates/                     Domain crates
tests/e2e/                    Playwright E2E tests
docs/                         VitePress documentation site
```

## Helpful Links

| Resource | Link |
|----------|------|
| Documentation | [markbrutx.github.io/Tabby](https://markbrutx.github.io/Tabby/) |
| Architecture | [Architecture Overview](https://markbrutx.github.io/Tabby/architecture/) |
| Coding Standards | [Standards](https://markbrutx.github.io/Tabby/contributing/coding-standards) |
| Development Setup | [Dev Guide](https://markbrutx.github.io/Tabby/contributing/development) |
| Issue Tracker | [GitHub Issues](https://github.com/markbrutx/Tabby/issues) |

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
