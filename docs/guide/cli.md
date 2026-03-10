# CLI Usage & Automation

Tabby includes a command-line interface for creating and configuring workspaces from scripts, shell aliases, and hotkey daemons. When Tabby is already running, CLI commands are forwarded to the existing window via single-instance routing — no duplicate app launches.

## How It Works

Tabby enforces a single running instance. When you run a `tabby` command:

1. If Tabby isn't running, it launches and applies your flags
2. If Tabby is already running, the command is forwarded to the existing window

This makes Tabby fully scriptable. You can bind workspace setups to hotkeys, shell aliases, or startup scripts and they'll always target the same Tabby window.

## Command Reference

### Flags

| Flag | Description | Default |
|------|-------------|---------|
| `--new-tab` | Opens a new tab in the running instance | — |
| `--layout <preset>` | Layout preset for the new tab | `1x1` |
| `--profile <name>` | Pane profile | `terminal` |
| `--cwd <path>` | Working directory for panes | Current directory |
| `--command <cmd>` | Custom command (requires `--profile custom`) | — |

### Layout Presets

`1x1` (single pane), `1x2` (two side-by-side), `2x2` (four-pane grid), `2x3` (six panes), `3x3` (nine panes).

### Profile Names

`terminal`, `claude-code`, `codex`, `custom`.

## Basic Examples

```bash
# Open a new tab with default settings
tabby --new-tab

# Open a new tab with a 2x2 grid layout
tabby --new-tab --layout 2x2

# Open a new tab with Codex profile
tabby --new-tab --profile codex

# Open a new tab in a specific directory
tabby --new-tab --cwd /path/to/project

# Open a new tab with a custom command
tabby --new-tab --profile custom --command "npm run dev"
```

### Combining Flags

Flags compose naturally — use as many as you need:

```bash
# Full workspace setup in one command
tabby --new-tab --layout 2x2 --profile terminal --cwd ~/projects/my-app

# Quick custom command in a new tab
tabby --new-tab --profile custom --command "docker compose up"
```

## Workflow Recipes

### Full-Stack Dev Workspace

Set up a frontend + backend + tests workspace with one script:

```bash
#!/bin/bash
# dev-workspace.sh — launch a complete dev environment

# Frontend dev server
tabby --new-tab --layout 2x2 --cwd ~/projects/my-app/frontend

# Backend server in a new tab
tabby --new-tab --cwd ~/projects/my-app/backend

# Tests watcher
tabby --new-tab --profile custom --command "cd ~/projects/my-app && bun run test:watch"
```

### Monitoring Dashboard

Nine-pane monitoring layout:

```bash
#!/bin/bash
# monitoring.sh — system monitoring dashboard

tabby --new-tab --layout 3x3 --profile custom --command "htop"
```

Each pane starts the same command — modify per-pane after launch, or create individual tabs:

```bash
# Or use separate tabs for different monitors
tabby --new-tab --profile custom --command "htop"
tabby --new-tab --profile custom --command "docker stats"
tabby --new-tab --profile custom --command "tail -f /var/log/system.log"
```

### Multi-Project Workspace

Different projects in different tabs, each with the right directory:

```bash
#!/bin/bash
# multi-project.sh

tabby --new-tab --layout 1x2 --cwd ~/projects/frontend-app
tabby --new-tab --layout 1x2 --cwd ~/projects/api-server
tabby --new-tab --cwd ~/projects/infrastructure
```

### AI-Assisted Development

Claude Code alongside your terminal:

```bash
tabby --new-tab --layout 1x2 --profile claude-code --cwd ~/projects/my-app
```

## Shell Integration

### Aliases

Add to your `~/.zshrc` or `~/.bashrc`:

```bash
# Quick workspace launchers
alias td='tabby --new-tab --layout 1x2 --cwd ~/projects/my-app'
alias tmon='tabby --new-tab --profile custom --command "htop"'
alias tai='tabby --new-tab --profile claude-code --cwd .'
alias tgrid='tabby --new-tab --layout 2x2'

# Project-specific aliases
alias tfrontend='tabby --new-tab --cwd ~/projects/my-app/frontend'
alias tbackend='tabby --new-tab --cwd ~/projects/my-app/backend'
```

### Shell Function

A wrapper function for dynamic workspace creation:

```bash
# Add to ~/.zshrc
tdev() {
  local dir="${1:-.}"
  local layout="${2:-1x2}"
  tabby --new-tab --layout "$layout" --cwd "$dir"
}

# Usage:
# tdev                          → new tab in current dir, 1x2 layout
# tdev ~/projects/my-app        → new tab in project dir
# tdev ~/projects/my-app 2x2    → new tab with 2x2 grid
```

## Hotkey Daemon Examples

### skhd

[skhd](https://github.com/koekeishiya/skhd) is a simple hotkey daemon for macOS:

```
# ~/.skhdrc

# Cmd+Shift+T → new Tabby tab in current layout
cmd + shift - t : tabby --new-tab

# Cmd+Shift+G → 2x2 grid in home directory
cmd + shift - g : tabby --new-tab --layout 2x2 --cwd ~

# Cmd+Shift+A → Claude Code in current directory
cmd + shift - a : tabby --new-tab --profile claude-code --cwd ~

# Cmd+Shift+M → monitoring dashboard
cmd + shift - m : tabby --new-tab --layout 3x3 --profile custom --command "htop"
```

### Hammerspoon

[Hammerspoon](https://www.hammerspoon.org/) is a Lua-based macOS automation tool:

```lua
-- ~/.hammerspoon/init.lua

-- Cmd+Shift+T → new Tabby tab
hs.hotkey.bind({"cmd", "shift"}, "t", function()
  hs.execute("tabby --new-tab", true)
end)

-- Cmd+Shift+D → dev workspace
hs.hotkey.bind({"cmd", "shift"}, "d", function()
  hs.execute("tabby --new-tab --layout 2x2 --cwd ~/projects/my-app", true)
end)
```

### Raycast Script Commands

Create a Raycast script command for quick workspace launching:

```bash
#!/bin/bash
# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title New Tabby Workspace
# @raycast.mode silent

tabby --new-tab --layout 2x2 --cwd ~/projects
```

## Scripting

### Complete Workspace Setup Script

A full example that sets up an entire development environment:

```bash
#!/bin/bash
# setup-workspace.sh — Set up a complete development workspace
# Usage: ./setup-workspace.sh [project-dir]

PROJECT_DIR="${1:-$(pwd)}"

echo "Setting up Tabby workspace for: $PROJECT_DIR"

# Tab 1: Development (2 panes — editor + dev server)
tabby --new-tab --layout 1x2 --cwd "$PROJECT_DIR"

# Tab 2: Testing
tabby --new-tab --profile custom --command "cd $PROJECT_DIR && bun run test:watch"

# Tab 3: Claude Code for AI assistance
tabby --new-tab --profile claude-code --cwd "$PROJECT_DIR"

# Tab 4: Git & misc
tabby --new-tab --cwd "$PROJECT_DIR"

echo "Workspace ready."
```

Make it executable and run:

```bash
chmod +x setup-workspace.sh
./setup-workspace.sh ~/projects/my-app
```
