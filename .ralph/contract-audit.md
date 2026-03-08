# Contract Audit — tabby-contracts

**Date:** 2026-03-09
**Story:** US-026 — Audit and remove all declared-but-not-real contract abstractions
**Result:** All 24 contract types are fully connected. Zero phantom types found.

## Methodology

Every type in `src-tauri/crates/tabby-contracts/src/lib.rs` was checked for:
1. At least one **producer** (a place that constructs/creates the type)
2. At least one **consumer** (a place that uses/reads the type)
3. Frontend transport clients only reference events that the backend actually emits

## Types Audited

### View DTOs (transport boundary)

| Type | Producer | Consumer | Status |
|------|----------|----------|--------|
| `ThemeModeDto` | `dto_mappers::theme_mode_to_dto()` | `SettingsView` field, `theme_mode_from_dto()` | ✅ |
| `LayoutPresetDto` | `dto_mappers::layout_preset_to_dto()` | `SettingsView` field, `WorkspaceCommandDto::OpenTab`, `layout_preset_from_dto()` | ✅ |
| `SplitDirectionDto` | `dto_mappers::split_node_to_dto()` | `SplitNodeDto` field, `WorkspaceCommandDto::SplitPane`, `split_direction_from_dto()` | ✅ |
| `SplitNodeDto` | `dto_mappers::split_node_to_dto()` | `TabView.layout` field, frontend layout tree renderer | ✅ |
| `PaneSpecDto` | `dto_mappers::pane_content_to_spec_dto()` | `PaneView.spec`, `OpenTab`, `SplitPane`, `ReplacePaneSpec`, `pane_spec_from_dto()` | ✅ |
| `PaneView` | `dto_mappers::workspace_view_from_session()` | `TabView.panes`, frontend workspace snapshot | ✅ |
| `TabView` | `dto_mappers::workspace_view_from_session()` | `WorkspaceView.tabs`, frontend tab rendering | ✅ |
| `WorkspaceView` | `dto_mappers::workspace_view_from_session()` | `WorkspaceBootstrapView`, projection publisher, frontend store | ✅ |
| `ProfileView` | `dto_mappers::profile_catalog_view_from_catalog()` | `ProfileCatalogView.terminal_profiles`, frontend settings | ✅ |
| `ProfileCatalogView` | `dto_mappers::profile_catalog_view_from_catalog()` | `WorkspaceBootstrapView`, `SettingsProjectionUpdatedEvent`, frontend settings store | ✅ |
| `SettingsView` | `dto_mappers::settings_view_from_preferences()` | `WorkspaceBootstrapView`, settings command handler, projection publisher, frontend store | ✅ |
| `RuntimeKindDto` | `dto_mappers::runtime_kind_to_dto()` | `PaneRuntimeView.kind`, frontend runtime display | ✅ |
| `RuntimeStatusDto` | `dto_mappers::runtime_status_to_dto()` | `PaneRuntimeView.status`, frontend runtime status | ✅ |
| `PaneRuntimeView` | `dto_mappers::pane_runtime_to_view()` | `WorkspaceBootstrapView`, `RuntimeStatusChangedEvent`, frontend runtime store | ✅ |
| `WorkspaceBootstrapView` | `dto_mappers::bootstrap_view()` | `bootstrap()` command return, frontend initial load | ✅ |
| `BrowserSurfaceBoundsDto` | Frontend `boundsFromElement()` | `BrowserSurfaceCommandDto::Ensure`, `SetBounds`, browser surface handler | ✅ |

### Command DTOs (frontend → backend)

| Type | Variant | Frontend Producer | Backend Consumer |
|------|---------|-------------------|------------------|
| `WorkspaceCommandDto` | `OpenTab` | workspace store | `dto_mappers` → domain command |
| | `CloseTab` | workspace store | `dto_mappers` → domain command |
| | `SetActiveTab` | workspace store | `dto_mappers` → domain command |
| | `FocusPane` | workspace store | `dto_mappers` → domain command |
| | `SplitPane` | workspace store | `dto_mappers` → domain command |
| | `ClosePane` | workspace store | `dto_mappers` → domain command |
| | `SwapPaneSlots` | workspace store | `dto_mappers` → domain command |
| | `ReplacePaneSpec` | workspace store | `dto_mappers` → domain command |
| | `RestartPaneRuntime` | workspace store | `dto_mappers` → domain command |
| `SettingsCommandDto` | `Update` | settings store | `dto_mappers` → domain command |
| | `Reset` | settings store | `dto_mappers` → domain command |
| `RuntimeCommandDto` | `WriteTerminalInput` | runtime store | `dto_mappers` → domain command |
| | `ResizeTerminal` | runtime store | `dto_mappers` → domain command |
| | `NavigateBrowser` | runtime store | `dto_mappers` → domain command |
| | `ObserveTerminalCwd` | runtime store | `dto_mappers` → domain command |
| | `ObserveBrowserLocation` | runtime store | `dto_mappers` → domain command |
| `BrowserSurfaceCommandDto` | `Ensure` | browser webview hook | `browser_surface.rs` handler |
| | `SetBounds` | runtime store | `browser_surface.rs` handler |
| | `SetVisible` | runtime store | `browser_surface.rs` handler |
| | `Close` | runtime store | `browser_surface.rs` handler |

All command variants: ✅ (9 workspace + 2 settings + 5 runtime + 4 browser surface = 20 variants)

### Event DTOs (backend → frontend)

| Event Type | Backend Emitter | Frontend Listener | Event Name Match |
|------------|----------------|-------------------|------------------|
| `TerminalOutputEvent` | `pty.rs` via `app.emit()` | `RuntimeClient.listenTerminalOutput` | `terminal_output_received` ✅ |
| `RuntimeStatusChangedEvent` | `TauriProjectionPublisher` | `RuntimeClient.listenStatusChanged` | `runtime_status_changed` ✅ |
| `WorkspaceProjectionUpdatedEvent` | `TauriProjectionPublisher` | `WorkspaceClient.listenProjectionUpdated` | `workspace_projection_updated` ✅ |
| `SettingsProjectionUpdatedEvent` | `TauriProjectionPublisher` | `SettingsClient.listenProjectionUpdated` | `settings_projection_updated` ✅ |

All event names match between `src-tauri/src/shell/mod.rs` constants and `src/app-shell/clients/shared.ts` constants.

## Conclusion

The `tabby-contracts` crate contains exactly the types needed by the system. No removals required. The contract surface is clean and every type participates in real producer-consumer paths across the Rust backend and TypeScript frontend.
