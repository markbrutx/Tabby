//! Architecture test: domain crates must not depend on tabby-contracts.
//!
//! Domain crates (tabby-workspace, tabby-runtime, tabby-settings) should depend
//! on tabby-kernel for shared value objects, never on tabby-contracts which
//! contains IPC/transport DTOs. This test parses each domain crate's Cargo.toml
//! and asserts tabby-contracts is absent from [dependencies].

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn parse_dependency_names(cargo_toml_content: &str) -> Vec<String> {
    let mut in_dependencies = false;
    let mut deps = Vec::new();

    for line in cargo_toml_content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('[') {
            in_dependencies = trimmed == "[dependencies]";
            continue;
        }

        if in_dependencies {
            if let Some(name) = trimmed.split('=').next() {
                let name = name.trim();
                if !name.is_empty() && !name.starts_with('#') {
                    deps.push(name.to_string());
                }
            }
        }
    }

    deps
}

#[test]
fn domain_crates_must_not_depend_on_tabby_contracts() {
    let domain_crates = ["tabby-workspace", "tabby-runtime", "tabby-settings"];
    let root = workspace_root();

    for crate_name in &domain_crates {
        let cargo_toml_path = root.join("crates").join(crate_name).join("Cargo.toml");

        let content = std::fs::read_to_string(&cargo_toml_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", cargo_toml_path.display()));

        let deps = parse_dependency_names(&content);

        assert!(
            !deps.iter().any(|d| d == "tabby-contracts"),
            "Domain crate '{crate_name}' must not depend on tabby-contracts. \
             Domain crates should depend on tabby-kernel for shared value objects. \
             Found tabby-contracts in [dependencies] of {}",
            cargo_toml_path.display()
        );
    }
}

#[test]
fn parse_dependency_names_extracts_correct_names() {
    let toml = r#"
[package]
name = "test-crate"

[dependencies]
tabby-kernel = { path = "../tabby-kernel" }
serde = { version = "1.0", features = ["derive"] }
thiserror = "2.0"

[dev-dependencies]
tokio = "1"
"#;
    let deps = parse_dependency_names(toml);
    assert_eq!(deps, vec!["tabby-kernel", "serde", "thiserror"]);
}
