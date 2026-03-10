# Features

## Workspace Tabs

Tabby uses browser-style tabs where each tab is a fully independent workspace. Every tab has its own layout, its own set of panes, and its own running terminal sessions. Nothing is shared between tabs — switching from Tab 1 to Tab 2 and back will never interrupt a running process.

**Typical setup:**

- **Tab 1** — Frontend dev: `npm run dev` in one pane, editor in another, browser preview in a third
- **Tab 2** — Backend: API server running, database shell open, logs tailing
- **Tab 3** — Git & monitoring: `git log`, htop, docker stats

**Shortcuts:**

| Action | Shortcut |
|--------|----------|
| New tab | `Cmd+T` |
| Close tab | `Cmd+W` |
| Switch to tab 1-9 | `Cmd+1` through `Cmd+9` |

Tabs display the active pane's working directory name in the tab title, so you can quickly identify which project each tab belongs to.

## Grid Layouts

Every tab starts from a layout preset. After choosing a preset, you can drag pane borders to resize them freely.

### Available Presets

**1x1** — Single full-screen pane. The default. Good for focused single-task work.

```
┌─────────────────────┐
│                     │
│      Terminal       │
│                     │
└─────────────────────┘
```

**1x2** — Two panes side by side. Great for code + output, or editor + tests.

```
┌──────────┬──────────┐
│          │          │
│  Pane 1  │  Pane 2  │
│          │          │
└──────────┴──────────┘
```

**2x2** — Four-pane grid. The workhorse layout for multi-task workflows.

```
┌──────────┬──────────┐
│  Pane 1  │  Pane 2  │
├──────────┼──────────┤
│  Pane 3  │  Pane 4  │
└──────────┴──────────┘
```

**2x3** — Six panes. Useful for monitoring dashboards or projects with many services.

```
┌───────┬───────┬───────┐
│   1   │   2   │   3   │
├───────┼───────┼───────┤
│   4   │   5   │   6   │
└───────┴───────┴───────┘
```

**3x3** — Nine panes. Maximum density for comprehensive monitoring or microservice architectures.

```
┌─────┬─────┬─────┐
│  1  │  2  │  3  │
├─────┼─────┼─────┤
│  4  │  5  │  6  │
├─────┼─────┼─────┤
│  7  │  8  │  9  │
└─────┴─────┴─────┘
```

**Tips:**

- You can change the layout preset after creation — existing panes rearrange into the new grid
- Drag any border between panes to resize them to your preferred proportions
- The default layout preset is configurable in Settings

## Persistent Live Terminals

Every pane runs an independent PTY (pseudo-terminal) session. This is a core design principle, not just a feature — Tabby guarantees that your running processes are never killed by normal UI interactions.

**What survives:**

- Switching between tabs
- Resizing panes or changing layouts
- Changing focus between panes
- Opening or closing other tabs
- Opening the settings modal

**Practical example:**

1. Start `npm run dev` in Pane 1 — your dev server is running
2. Switch to Tab 2 to check logs
3. Open Settings to adjust font size
4. Switch back to Tab 1 — `npm run dev` is still running with full scroll history intact

This means you can treat Tabby like a persistent workspace. Start your processes once, then navigate freely without worrying about restarts.

## CLI Profiles

Each pane can run a different profile. Profiles determine what command launches when the pane starts.

### Terminal

Launches your default login shell (whatever `$SHELL` is set to, usually `zsh` on macOS). This is the default profile for new panes.

### Claude Code

Launches the [Claude Code](https://docs.anthropic.com/en/docs/claude-code) CLI directly in the pane. Requires the `claude` CLI to be installed and available on your `$PATH`.

**Use case:** Keep an AI coding assistant open alongside your terminals. One pane for Claude Code, another for your shell — collaborate with AI without leaving your workspace.

### Codex

Launches the [OpenAI Codex](https://github.com/openai/codex) CLI. Requires the `codex` CLI to be installed.

### Custom

Runs any shell command you specify. Examples:

- `htop` — system monitor
- `docker compose up` — start a Docker stack
- `python manage.py runserver` — Django dev server
- `tail -f /var/log/system.log` — log watcher

**Switching profiles:** You can change a pane's profile at any time from the pane header. The pane restarts with the new command while all other panes remain untouched.

## Per-Pane Working Directory

Each pane starts in and tracks its own working directory independently. When you create a tab, each pane can be pointed at a different project folder.

**Example setup for a monorepo:**

- Pane 1: `~/projects/my-app/frontend` — running `npm run dev`
- Pane 2: `~/projects/my-app/backend` — running `cargo run`
- Pane 3: `~/projects/my-app` — running `git status`

Working directories persist through layout changes and tab switches. You can set the default working directory for new panes in Settings.

## Browser Panes

Browser panes embed a web view directly inside your workspace layout, side by side with terminal panes. No need to Alt-Tab to a browser window.

**Common use cases:**

- **Localhost preview** — keep `localhost:3000` visible while editing code
- **Documentation** — pin API docs or framework guides next to your terminal
- **Dashboards** — monitoring UIs, Grafana, CI/CD status pages
- **Web apps** — any URL works

Browser panes include a URL bar for navigation and support standard web interactions (scrolling, clicking, form input).

## Git Integration

Built-in Git UI accessible from any pane's context. Currently read-only — gives you a full view of your repository state without leaving the workspace.

### Repository Status

See which files are modified, staged, untracked, or conflicted at a glance. File status updates automatically as you work in your terminals.

### Branches

Browse local and remote branches. See the current branch, switch branches, and view branch history.

### Commit History

Full commit log with author, date, and message. Navigate through your project's history to understand what changed and when.

### Diffs

Syntax-highlighted diff viewer with two modes:

- **Unified** — traditional patch-style diff with additions and deletions inline
- **Split** — side-by-side comparison of old and new versions

### Blame

Line-by-line blame annotations showing who last modified each line, when, and in which commit. Useful for understanding the context behind code changes.

### Stash

View your stash entries and their contents. See what's stashed without leaving the Tabby interface.

## Theme System

Customize Tabby's appearance with the built-in theme editor.

### Light and Dark Modes

Switch between light and dark base themes. The app follows your macOS system preference by default, or you can set it manually.

### Custom Themes

Create and edit custom themes by modifying color tokens:

- Background colors (primary, soft, muted, alt)
- Text colors (primary, secondary, tertiary)
- Brand/accent colors
- Border and divider colors
- Font families (base and mono)

Changes preview live as you edit, so you can see exactly how your theme looks before saving.

### Import / Export

Share themes with others by exporting them as files. Import themes from the community or your own backups.

## Settings

Access settings with `Cmd+,` or through the app menu.

### Configurable Options

| Setting | Description | Default |
|---------|-------------|---------|
| Font size | Terminal text size, also adjustable with `Cmd+=` / `Cmd+-` | System default |
| Default layout | Layout preset for new tabs | `1x1` |
| Default profile | Profile for new panes | Terminal |
| Working directory | Starting directory for new panes | Home (`~`) |
| Theme | Light, dark, or custom | System preference |

## Keyboard Shortcuts

Tabby is designed for keyboard-first workflows. All major actions have keyboard shortcuts.

### Tab Management

| Action | Shortcut |
|--------|----------|
| New tab | `Cmd+T` |
| Close tab | `Cmd+W` |
| Switch to tab 1 | `Cmd+1` |
| Switch to tab 2 | `Cmd+2` |
| Switch to tab 3 | `Cmd+3` |
| Switch to tab 4 | `Cmd+4` |
| Switch to tab 5 | `Cmd+5` |
| Switch to tab 6 | `Cmd+6` |
| Switch to tab 7 | `Cmd+7` |
| Switch to tab 8 | `Cmd+8` |
| Switch to tab 9 | `Cmd+9` |

### View

| Action | Shortcut |
|--------|----------|
| Zoom in | `Cmd+=` |
| Zoom out | `Cmd+-` |
| Settings | `Cmd+,` |

### Tips

- **Quick tab switching** — `Cmd+1` through `Cmd+9` jumps directly to a tab by position, much faster than cycling through tabs
- **Zoom** — affects font size globally across all panes; great for presentations or pairing sessions
- **Muscle memory** — shortcuts match standard macOS conventions (Safari, Chrome), so they feel natural immediately

## Performance

Tabby is built for responsiveness. Performance targets that guide development:

| Metric | Target |
|--------|--------|
| Input latency | < 5 ms |
| Tab switch | < 50 ms |
| Session loss | Zero during normal UI navigation |

The Rust backend handles PTY I/O on dedicated threads, and terminal output is streamed directly to the xterm.js renderer with minimal overhead. See [ADR-001](/architecture/adr/001-terminal-output-hot-path) for the technical details of the terminal output hot path.
