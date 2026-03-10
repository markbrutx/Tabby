# Features

## Workspace Tabs

Browser-style tabs where each tab is an independent terminal workspace. Create tabs with `Cmd+T`, close with `Cmd+W`, and switch directly with `Cmd+1` through `Cmd+9`.

Each tab remembers its own layout, pane configuration, and active terminal sessions independently.

## Grid Layouts

Every workspace starts from a layout preset and panes remain resizable after creation.

| Preset | Description |
|--------|-------------|
| `1x1` | Single full-screen pane |
| `1x2` | Two panes side by side |
| `2x2` | Four-pane grid |
| `2x3` | Six-pane grid |
| `3x3` | Nine-pane grid |

You can also split panes horizontally or vertically after creation, swap pane positions, and collapse or expand individual panes.

## Persistent Live Terminals

Every pane owns an independent PTY session. Switching tabs, changing focus, or resizing the layout will never restart the running process. This is a core invariant -- terminal sessions are always preserved.

## CLI Profiles

Each pane can launch a different runtime profile:

- **Terminal** -- plain shell session
- **Claude Code** -- launches Claude Code CLI
- **Codex** -- launches Codex CLI
- **Custom** -- any shell command you specify

Profiles are editable without recreating the workspace. Change a pane's profile and it restarts with the new command while other panes remain unaffected.

## Per-Pane Working Directory

Each pane can start in and stay associated with its own project folder. Working directories are tracked independently per pane and survive layout changes.

## Browser Panes

Browser panes render alongside terminal panes within the same workspace layout. Useful for keeping documentation, dashboards, or web apps visible next to your terminals.

## Git Integration

Built-in Git UI accessible from any terminal pane:

- Repository status and file change tracking
- Branch selection and management
- Commit history with blame annotations
- Syntax-highlighted diff viewer (unified and split modes)
- Stash management

## Theme System

Customizable appearance with built-in themes and a theme editor:

- Light and dark modes
- Custom color token editing
- Theme import/export
- Live preview while editing

## Settings

Configurable through the settings modal:

- Font size (with keyboard zoom: `Cmd+=` / `Cmd+-`)
- Default startup layout
- Default profile
- Default working directory
- Fullscreen behavior
- Keyboard shortcuts reference

## Keyboard Shortcuts

Tabby is designed for keyboard-first workflows:

| Action | Shortcut |
|--------|----------|
| New tab | `Cmd+T` |
| Close tab | `Cmd+W` |
| Switch tab | `Cmd+1` - `Cmd+9` |
| Zoom in | `Cmd+=` |
| Zoom out | `Cmd+-` |
| Settings | `Cmd+,` |

## Performance

Performance targets that guide development:

- Input latency: < 5ms
- Tab switch: < 50ms
- Zero PTY/session loss during normal UI navigation
