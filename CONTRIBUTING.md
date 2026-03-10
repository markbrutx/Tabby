# Contributing to Tabby

Tabby is a **macOS-first** terminal workspace app. The architecture is designed to be portable, but the current focus is macOS.

If you're interested in bringing Tabby to **Linux or Windows**, contributions are welcome — check the [architecture docs](https://markbrutx.github.io/Tabby/architecture/) to understand the platform abstraction layer.

## Getting Started

1. [Install prerequisites](https://markbrutx.github.io/Tabby/guide/installation) (macOS, Bun, Rust, Xcode CLI tools)
2. Clone and install:
   ```bash
   git clone https://github.com/markbrutx/Tabby.git
   cd Tabby
   bun install
   ```
3. Run in development mode:
   ```bash
   bun run tauri dev    # Full app with PTY
   bun run dev          # Frontend only (mock transport)
   ```

## Before Submitting

Run verification:

```bash
# Frontend
bun run lint
bun run typecheck
bun run test

# Rust
cd src-tauri
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace

# Or everything at once
bun run verify:all
```

## Guidelines

- Keep diffs minimal and targeted
- Prefer TDD for bugs and new behavior
- Preserve the invariant that terminal sessions survive tab switches and layout changes
- Follow [coding standards](https://markbrutx.github.io/Tabby/contributing/coding-standards)
- Use conventional commits: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`

## Docs

- [Documentation Site](https://markbrutx.github.io/Tabby/)
- [Architecture](https://markbrutx.github.io/Tabby/architecture/)
- [Development Setup](https://markbrutx.github.io/Tabby/contributing/development)

## License

MIT
