# Installation

## Download

The easiest way to install Tabby is to download the pre-built `.dmg` from GitHub Releases.

**[Download Latest Release](https://github.com/markbrutx/Tabby/releases/latest)**

### Choose Your Architecture

| Build | For | File |
|-------|-----|------|
| **Apple Silicon** | M1, M2, M3, M4 Macs | `Tabby_x.x.x_aarch64.dmg` |
| **Intel** | Intel-based Macs | `Tabby_x.x.x_x64.dmg` |

Not sure which one? Click the Apple menu → **About This Mac**. If the chip says "Apple M1" (or M2, M3, M4), download the Apple Silicon build. If it says "Intel", download the Intel build.

### Install Steps

1. Open the downloaded `.dmg` file
2. Drag **Tabby** into the **Applications** folder
3. Open Tabby from your Applications folder or Spotlight

### Bypass Gatekeeper (First Launch)

Tabby is currently unsigned. macOS will block the first launch with a warning like *"Tabby can't be opened because Apple cannot check it for malicious software."*

To bypass this:

1. **Right-click** (or Control-click) the Tabby app in Finder
2. Choose **Open** from the context menu
3. Click **Open** in the confirmation dialog

You only need to do this once. After the first launch, macOS remembers your choice.

Alternatively, run this in Terminal:

```bash
xattr -cr /Applications/Tabby.app
```

## System Requirements

- **macOS 13 (Ventura)** or later
- Apple Silicon or Intel x64 processor
- ~100 MB disk space

## Build from Source

For developers who want to contribute or run the latest unreleased code.

### Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| **macOS** | 13+ | — |
| **Xcode CLI Tools** | Latest | `xcode-select --install` |
| **Rust** | Latest stable | [rustup.rs](https://rustup.rs/) |
| **Bun** | 1.x+ | [bun.sh](https://bun.sh/) |

### Clone and Install

```bash
git clone https://github.com/markbrutx/Tabby.git
cd Tabby
bun install
```

### Run the Desktop App

Launch the full Tauri app with real PTY integration and hot reload:

```bash
bun run tauri dev
```

This compiles the Rust backend, starts the Vite dev server, and opens the Tabby window. First build takes a few minutes; subsequent runs are fast.

### Run Frontend Only

If you're working on UI and don't need real terminal sessions, run the frontend in the browser with a mock transport layer:

```bash
bun run dev
```

This simulates terminal behavior so you can iterate on the interface without compiling Rust. Useful for styling, layout work, and component development.

### Build a Release Binary

```bash
# Frontend bundle only
bun run build

# Full desktop app (.app + .dmg)
bun run tauri build
```

The built `.dmg` appears in `src-tauri/target/release/bundle/dmg/`.

## Single-Instance Safety

Tabby is designed to run as a single instance. Running multiple copies from the same checkout can cause port conflicts and PTY issues. For long-running dev sessions, use tmux:

```bash
tmux new-session -d -s tabby-dev 'cd ~/path/to/Tabby && bun run tauri dev'
tmux attach -t tabby-dev
```

## Verification

Run all checks to confirm a healthy development setup:

```bash
# Everything at once
bun run verify:all
```

Or run checks individually:

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

## Updating

### From .dmg

Download the latest `.dmg` from the [releases page](https://github.com/markbrutx/Tabby/releases/latest) and drag the new Tabby app into Applications, replacing the old one. Your settings are preserved.

### From source

```bash
cd Tabby
git pull
bun install
bun run tauri dev
```

## Uninstalling

1. Quit Tabby if it's running
2. Drag **Tabby** from Applications to the Trash
3. Optionally, remove stored preferences:

```bash
rm -rf ~/Library/Application\ Support/com.tabby.app
```
