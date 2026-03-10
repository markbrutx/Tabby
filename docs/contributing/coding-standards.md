# Coding Standards

## Core Invariants

These rules are non-negotiable and must be preserved across all changes:

1. **Terminal runtimes survive UI changes** -- switching tabs, changing focus, or resizing layout must never restart a running PTY session
2. **Single runtime owner** -- `RuntimeApplicationService` is the sole owner of runtime lifecycle. All start/stop/replace/restart flows go through it
3. **Port-adapter isolation** -- application services depend on port traits (`ports.rs`), never on concrete Tauri or plugin imports
4. **Domain purity** -- domain crates depend on `tabby-kernel` only, never on `tabby-contracts` or each other
5. **DTO boundary** -- generated DTOs (`tauri-bindings.ts`) appear only in transport clients and snapshot-mappers, never in stores or components
6. **Thin commands** -- Tauri IPC handlers only deserialize, delegate, and return. No business logic
7. **Projection-based publishing** -- `ProjectionPublisherPort` accepts domain types, not DTOs. Infrastructure adapters handle mapping

## TypeScript

- Strict mode enabled
- Prefer immutable patterns -- create new objects instead of mutating
- Functions under 50 lines, files under 800 lines
- No `console.log` in committed code
- No hardcoded values -- use constants or configuration

## Rust

- Use `tracing` for diagnostics, not `println!`
- No `.unwrap()` or `.expect()` in production code
- Run `cargo fmt` and `cargo clippy` before committing
- Keep Tauri command handlers thin -- delegate to application services

## Frontend Architecture

- Each feature owns its `domain/`, `application/`, and `components/` directories
- Features do not import from other features' internals
- Zustand stores use internal read models derived from snapshots
- Shared UI components live in `src/components/`

## Rust Architecture

- Domain crates are pure -- no framework dependencies
- Application services define behavior through port traits
- Infrastructure adapters implement ports and live in `infrastructure/`
- Cross-context coordination goes through `AppShell`, not direct service-to-service calls

## Dependency Direction

```
Presentation -> Application -> Domain
                    |
              (port traits)
                    |
              Infrastructure
```

Never invert this direction. Domain code must not know about the presentation layer or infrastructure details.
