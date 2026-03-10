# Contributing

Tabby welcomes contributions. Here are the guidelines to keep the codebase healthy.

## Before You Start

1. Read the [Architecture Overview](/architecture/) to understand the system layers
2. Set up your [Development Environment](/contributing/development)
3. Review the [Coding Standards](/contributing/coding-standards)

## Contribution Guidelines

### Keep Diffs Minimal

Focus changes on what's necessary. A bug fix doesn't need surrounding code cleaned up. A feature doesn't need extra configurability.

### Run Verification

Always run the relevant checks for the files you touched:

- **Frontend changes**: `bun run lint && bun run typecheck && bun run test`
- **Rust changes**: `cargo fmt --all --check && cargo clippy --workspace && cargo test --workspace`
- **Full check**: `bun run verify:all`

### Test-Driven Development

For bugs and new behavior, prefer TDD:

1. Write the test first (it should fail)
2. Write minimal implementation to pass
3. Refactor

### Preserve Invariants

These invariants must never be broken:

- Terminal runtimes survive tab switches, focus changes, and layout updates
- Each pane owns one independent runtime and one working directory
- Application services depend on port traits, never on concrete infrastructure
- Frontend stores use internal read models, not raw DTOs

## Project Layout

| Path | What's There |
|------|-------------|
| `src/` | React frontend (features, app-shell, components) |
| `src-tauri/src/` | Tauri bootstrap, shell integration, CLI, menu |
| `src-tauri/crates/` | Domain crates (kernel, workspace, runtime, settings, git, contracts) |
| `tests/e2e/` | Playwright E2E tests |
| `docs/` | This documentation site |

## License

Tabby is released under the [MIT License](https://github.com/markbrutx/Tabby/blob/master/LICENSE).
