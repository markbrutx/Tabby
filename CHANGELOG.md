# Changelog

All notable changes to Tabby will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- VitePress documentation site with GitHub Pages deployment
- Git integration: repository status, branches, commits, diffs, blame, stash
- Theme system with light/dark modes and custom theme editor
- Collapse/expand pane management
- Browser pane support alongside terminal panes
- E2E test suite (Playwright)
- CI/CD pipeline with GitHub Actions
- Automated release workflow for macOS (arm64 + x64)

### Changed
- Refactored git pane into modular action and component structure
- Streamlined Tauri application exit process

## [0.1.0] - 2025-03-10

### Added
- Initial release
- Browser-style workspace tabs
- Split-pane layouts (1x1 to 3x3 presets)
- Persistent PTY terminal sessions
- Per-pane runtime profiles (Terminal, Claude Code, Codex, Custom)
- Per-pane working directory
- Settings persistence (font size, layout, profile, fullscreen)
- Keyboard shortcuts
- CLI launch overrides (`--new-tab`, `--layout`, `--profile`, `--cwd`, `--command`)
- Single-instance routing
- macOS native titlebar overlay
