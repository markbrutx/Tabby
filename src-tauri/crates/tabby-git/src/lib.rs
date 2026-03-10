// tabby-git — Git bounded context domain crate.
//
// Module structure:
// - value_objects: Git-specific value objects (branch names, commit hashes, etc.)
// - entities: Git domain entities (repository state, diff, file status, etc.)
// - events: Domain events for the Git context
//
// Dependencies: tabby-kernel only. No serde, specta, or tauri.
