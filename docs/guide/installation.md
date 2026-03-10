# Installation

## Prerequisites

- **macOS** (macOS-first, architecture remains portable)
- [Bun](https://bun.sh/) -- JavaScript runtime and package manager
- [Rust](https://rustup.rs/) -- for the Tauri backend
- Xcode Command Line Tools (`xcode-select --install`)

## Quick Start

Clone the repository and install dependencies:

```bash
git clone https://github.com/markbrutx/Tabby.git
cd Tabby
bun install
```

### Full Desktop App

Launch the complete Tauri desktop app with real PTY integration:

```bash
bun run tauri dev
```

### Frontend-Only Mode

Run the web frontend without the native shell. Useful for UI development -- a mock transport simulates terminal behavior in the browser:

```bash
bun run dev
```

## Build

```bash
# Frontend bundle only
bun run build

# Desktop bundle via Tauri (produces .app / .dmg)
bun run tauri build
```

## Single-Instance Safety

Tabby is designed around a single running app instance. Do not start multiple copies simultaneously from the same checkout.

For long-running development sessions, use `tmux`:

```bash
tmux new-session -d -s tabby-dev 'cd /path/to/Tabby && bun run tauri dev'
tmux attach -t tabby-dev
```

## Verification

Run all checks to confirm a healthy setup:

**Frontend:**

```bash
bun run lint       # ESLint + DTO boundary check
bun run typecheck  # TypeScript strict mode
bun run test       # Vitest unit/integration tests
bun run test:e2e   # Playwright E2E tests
```

**Rust backend:**

```bash
cd src-tauri
cargo check --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

Or run everything at once:

```bash
bun run verify:all
```
