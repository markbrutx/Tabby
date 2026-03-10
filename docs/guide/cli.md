# CLI Usage

When the `tabby` binary is installed or built, it supports launch overrides that control the running app instance via single-instance routing.

## Basic Usage

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

## Flags

| Flag | Description |
|------|-------------|
| `--new-tab` | Opens a new tab in the running Tabby instance |
| `--layout <preset>` | Sets the layout for the new tab (`1x1`, `1x2`, `2x2`, `2x3`, `3x3`) |
| `--profile <name>` | Sets the pane profile (`terminal`, `claude-code`, `codex`, `custom`) |
| `--cwd <path>` | Sets the working directory for panes in the new tab |
| `--command <cmd>` | Sets a custom command (used with `--profile custom`) |

## Combining Flags

Flags compose naturally:

```bash
# Full workspace setup in one command
tabby --new-tab --layout 2x2 --profile terminal --cwd ~/projects/my-app

# Quick custom command in a new tab
tabby --new-tab --profile custom --command "docker compose up"
```

## Single-Instance Behavior

Tabby uses single-instance routing. When a Tabby instance is already running, CLI commands are forwarded to the existing window instead of launching a duplicate. This makes Tabby scriptable from hotkey daemons, shell aliases, or automation tools.

## Automation Examples

Add to your shell profile for quick workspace launchers:

```bash
# ~/.zshrc or ~/.bashrc

# Launch a dev workspace
alias tabby-dev='tabby --new-tab --layout 1x2 --cwd ~/projects/my-app'

# Launch a monitoring workspace
alias tabby-mon='tabby --new-tab --profile custom --command "htop"'
```

Use with macOS hotkey tools (like Hammerspoon or skhd):

```
# skhd example
cmd + shift - t : tabby --new-tab --layout 2x2 --cwd ~/projects
```
