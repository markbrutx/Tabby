# Getting Started

## What is Tabby?

Tabby is a free, open-source terminal workspace app built for macOS. Instead of scattering terminals across multiple windows or relying on tmux to keep sessions alive, Tabby gives you a single window with browser-style tabs, split-pane layouts, and persistent terminal sessions that never die when you switch context.

Think of it as a workspace manager for your terminal life. Each tab is an independent workspace. Each pane inside a tab runs its own terminal session with its own working directory and launch profile. Switch between tabs, resize panes, change layouts — your running processes stay exactly where you left them.

Tabby is built with Tauri v2 and Rust on the backend, React and TypeScript on the frontend. It ships as a lightweight native macOS app with real PTY sessions — not a web app pretending to be a terminal.

## Core Concepts

Understanding four concepts unlocks everything Tabby can do:

### Workspace

The top-level container. One Tabby window = one workspace. Everything lives inside it.

### Tabs

Browser-style tabs along the top of the workspace. Each tab is fully isolated — its own layout, its own set of panes, its own terminal sessions. Create with `Cmd+T`, close with `Cmd+W`, jump between them with `Cmd+1` through `Cmd+9`.

### Panes

Each tab contains one or more panes arranged in a grid layout. Every pane is an independent terminal (or browser) surface with its own PTY process, working directory, and runtime profile. Panes can be resized by dragging their borders.

### Profiles

Profiles define what runs inside a pane. Out of the box, Tabby ships with:

- **Terminal** — your default shell (`zsh`, `bash`, etc.)
- **Claude Code** — launches the Claude Code CLI
- **Codex** — launches the OpenAI Codex CLI
- **Custom** — any shell command you specify

You can swap a pane's profile at any time without affecting other panes.

## Quick Start

### Option A: Download the App

The fastest way to get started. Grab the latest `.dmg` from the [releases page](https://github.com/markbrutx/Tabby/releases/latest), drag Tabby to your Applications folder, and launch it.

> **First launch:** macOS will warn you about an unsigned app. Right-click the app icon, choose **Open**, and confirm. You only need to do this once. See [FAQ](/guide/faq) for details.

### Option B: Build from Source

For developers who want to contribute or run the latest code:

```bash
git clone https://github.com/markbrutx/Tabby.git
cd Tabby
bun install
bun run tauri dev
```

See [Installation](/guide/installation) for prerequisites and detailed setup instructions.

## Next Steps

- **[Installation](/guide/installation)** — download the app or set up the development environment from source
- **[Features](/guide/features)** — full walkthrough of tabs, layouts, profiles, Git integration, themes, and keyboard shortcuts
- **[CLI Usage](/guide/cli)** — automate workspace creation with command-line flags and shell scripts
- **[FAQ](/guide/faq)** — answers to common questions and troubleshooting tips
