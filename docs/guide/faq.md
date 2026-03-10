# FAQ & Troubleshooting

## Installation & Launch

### "macOS says the app is damaged" or "can't be opened"

Tabby is currently unsigned. macOS Gatekeeper blocks unsigned apps by default.

**Fix:** Right-click the app → choose **Open** → click **Open** in the dialog. You only need to do this once.

If that doesn't work, run this in your terminal:

```bash
xattr -cr /Applications/Tabby.app
```

Then open Tabby normally.

### Tabby won't start or shows a blank window

This usually means another Tabby instance is already running in the background.

**Fix:**

```bash
# Kill any existing Tabby processes
pkill -f "tabby"

# Then reopen Tabby
open /Applications/Tabby.app
```

If you're running from source:

```bash
pkill -f "target/debug/tabby"
pkill -f "tauri dev"
bun run tauri dev
```

### Where is Tabby installed?

- **App:** `/Applications/Tabby.app`
- **Settings:** `~/Library/Application Support/com.tabby.app/`

---

## Terminal & Shell

### How do I change the default shell?

Tabby uses your system's default login shell (usually `zsh` on modern macOS). To change it:

```bash
# Check your current shell
echo $SHELL

# Change to bash (example)
chsh -s /bin/bash
```

Alternatively, create a custom profile in Tabby's settings that launches whatever shell or command you want — this doesn't change your system default.

### Terminal output looks garbled or shows weird characters

This is usually a locale or font issue.

**Check your locale:**

```bash
locale
```

If you see warnings or `C` locale, add this to your `~/.zshrc`:

```bash
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8
```

**Font rendering:** Tabby uses the system monospace font stack. If you see missing glyphs (like Nerd Font icons), the terminal may need a font that includes those characters. Adjust the font in Tabby's settings.

### Can I use Tabby with tmux?

Yes. Tabby panes are real terminal sessions, so tmux, screen, and any other terminal multiplexer work normally inside them. Just keep in mind that Tabby already provides tabs and panes — you might not need tmux's multiplexing features on top.

### Do terminal sessions persist when I quit and reopen Tabby?

Not yet. Currently, terminal sessions are alive as long as Tabby is running — switching tabs, resizing, and changing layouts never kills your processes. But quitting the app ends all sessions. Session restore on relaunch is on the roadmap.

---

## Features

### Can I split panes vertically and horizontally?

Yes. Tabby supports grid layout presets (1x1, 1x2, 2x2, 2x3, 3x3) and you can resize panes by dragging their borders after creation. See [Features](/guide/features#grid-layouts) for details.

### What profiles are available?

Four built-in profiles:

| Profile | What it runs |
|---------|-------------|
| **Terminal** | Your default login shell |
| **Claude Code** | `claude` CLI (must be installed separately) |
| **Codex** | `codex` CLI (must be installed separately) |
| **Custom** | Any command you specify |

You can switch a pane's profile at any time from the pane header or settings.

### Can I open a web browser inside Tabby?

Yes. Browser panes render web content alongside your terminals within the same layout. Useful for documentation, localhost previews, or dashboards. See [Features](/guide/features#browser-panes).

### Does the Git UI support push/pull/commit?

The current Git integration is read-only: status, branches, commit history, diffs (unified and split), blame, and stash viewing. Write operations (commit, push, pull) are planned for a future release.

---

## Platform & Compatibility

### Can I use Tabby on Linux or Windows?

Not yet. Tabby is macOS-first. The underlying architecture (Tauri, Rust, React) is cross-platform, so Linux and Windows support is technically feasible and may come in the future, but it's not actively developed right now.

### What macOS versions are supported?

macOS 13 (Ventura) and later. Both Apple Silicon (M1/M2/M3/M4) and Intel processors are supported.

### Multiple monitors?

Tabby currently runs as a single window. You can move and resize it freely, but multi-window or multi-monitor workspace splitting isn't supported yet.

---

## Settings & Configuration

### Where are settings stored?

Settings are persisted via Tauri's plugin-store at:

```
~/Library/Application Support/com.tabby.app/
```

### How do I reset to default settings?

Quit Tabby, then delete the settings store:

```bash
rm -rf ~/Library/Application\ Support/com.tabby.app
```

Relaunch Tabby and it will start with fresh defaults.

### Can I export/import settings?

Not directly through a UI yet. You can back up and restore the settings directory manually. Theme import/export is available through the theme system UI.

---

## Development

### Build takes forever on first run

The first `bun run tauri dev` compiles the entire Rust workspace from scratch. This can take 2-5 minutes depending on your machine. Subsequent runs use incremental compilation and are much faster (usually a few seconds).

### Rust compiler errors after git pull

Dependencies may have changed. Run:

```bash
bun install
cd src-tauri
cargo update
```

### How do I run just the frontend without Rust?

```bash
bun run dev
```

This starts the Vite dev server with a mock transport layer that simulates terminal behavior in the browser. No Rust compilation needed. Great for UI development.
