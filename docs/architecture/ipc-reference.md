# IPC Reference

Tabby uses **specta** and **tauri-specta** to generate type-safe TypeScript bindings from Rust command definitions. This ensures the frontend and backend share a single source of truth for all IPC contracts.

## How It Works

1. **Rust commands** are defined in `src-tauri/src/commands/` as thin `#[tauri::command]` handlers
2. **specta** introspects the Rust types and generates TypeScript definitions
3. **tauri-specta** wires the generated bindings to Tauri's invoke system
4. The generated file lands at `src/contracts/tauri-bindings.ts`

::: warning
`tauri-bindings.ts` is auto-generated. Do not edit it manually.
:::

## Transport Boundary

The transport boundary sits between the frontend and backend:

```
Frontend Store  ->  Transport Client  ->  tauri-bindings.ts (invoke)
                                              |
                                        Tauri IPC bridge
                                              |
                                     commands/ (Rust handler)
                                              |
                                     mapping/ (DTO <-> domain)
                                              |
                                     application/ (service)
```

### Frontend Side

- `src/app-shell/clients/` -- `createTauriShellClients` factory produces typed clients (`WorkspaceClient`, `SettingsClient`, `RuntimeClient`)
- Clients call generated functions from `tauri-bindings.ts`
- Snapshot mappers in each feature's `application/` folder convert DTOs into internal read models

### Backend Side

- `src-tauri/src/commands/` -- thin handlers that deserialize DTOs, delegate to application services, and return response DTOs
- `src-tauri/src/mapping/dto_mappers.rs` -- maps between `tabby-contracts` DTOs and domain types
- Application services work only with domain types, never with DTOs

## DTO Crate

`tabby-contracts` (at `src-tauri/crates/tabby-contracts/`) defines all transport types:

- **Command DTOs** -- `WorkspaceCommandDto`, `SettingsCommandDto`, `RuntimeCommandDto`, `GitCommandDto`
- **View DTOs** -- `WorkspaceViewDto`, `SettingsViewDto`, `RuntimeViewDto`
- **Event structs** -- emitted via Tauri events for push-based updates
- **Value object re-exports** -- re-exports types from `tabby-kernel` that cross the IPC boundary

## Events

The backend pushes state updates to the frontend via Tauri events:

| Event | Payload | Trigger |
|-------|---------|---------|
| Workspace snapshot | `WorkspaceViewDto` | Tab/pane structural changes |
| Runtime snapshot | `RuntimeViewDto` | Runtime status or CWD changes |
| Settings snapshot | `SettingsViewDto` | Preferences updates |
| Terminal output | Raw bytes + pane ID | PTY read loop (high-frequency) |

Frontend stores subscribe to these events and update their local read models.

## DTO Boundary Rule

Generated DTOs from `tauri-bindings.ts` must only appear in:
- Transport clients (`app-shell/clients/`)
- Snapshot mappers (each feature's `application/snapshot-mappers.ts`)

They must **never** appear in:
- Zustand stores
- Domain models
- UI components

This keeps the frontend decoupled from the wire format.
