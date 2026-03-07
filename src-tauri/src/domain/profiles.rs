use serde::{Deserialize, Serialize};
use specta::Type;

use crate::domain::error::TabbyError;

pub const CUSTOM_PROFILE_ID: &str = "custom";
pub const TERMINAL_PROFILE_ID: &str = "terminal";
pub const BROWSER_PROFILE_ID: &str = "browser";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "camelCase")]
pub struct PaneProfile {
    pub id: String,
    pub label: String,
    pub description: String,
    pub startup_command: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedProfile {
    pub id: String,
    pub label: String,
    pub startup_command: Option<String>,
}

pub fn built_in_profiles() -> Vec<PaneProfile> {
    vec![
        PaneProfile {
            id: String::from(TERMINAL_PROFILE_ID),
            label: String::from("Terminal"),
            description: String::from(
                "Standard shell session \u{2014} your system shell (zsh, bash)",
            ),
            startup_command: None,
        },
        PaneProfile {
            id: String::from("claude"),
            label: String::from("Claude Code"),
            description: String::from(
                "Anthropic AI coding assistant \u{2014} launches \u{2018}claude\u{2019} CLI",
            ),
            startup_command: Some(String::from("claude")),
        },
        PaneProfile {
            id: String::from("codex"),
            label: String::from("Codex"),
            description: String::from(
                "OpenAI Codex agent \u{2014} launches \u{2018}codex\u{2019} CLI",
            ),
            startup_command: Some(String::from("codex")),
        },
        PaneProfile {
            id: String::from(CUSTOM_PROFILE_ID),
            label: String::from("Custom"),
            description: String::from("Run any command of your choice"),
            startup_command: None,
        },
        PaneProfile {
            id: String::from(BROWSER_PROFILE_ID),
            label: String::from("Browser"),
            description: String::from("Launch Google Chrome with a specific profile and URL"),
            startup_command: None,
        },
    ]
}

pub fn resolve_profile(
    profile_id: &str,
    startup_command: Option<String>,
) -> Result<ResolvedProfile, TabbyError> {
    let profile = built_in_profiles()
        .into_iter()
        .find(|candidate| candidate.id == profile_id)
        .ok_or_else(|| TabbyError::Validation(format!("Unknown profile: {profile_id}")))?;

    if profile.id == BROWSER_PROFILE_ID {
        return Ok(ResolvedProfile {
            id: profile.id,
            label: profile.label,
            startup_command: None,
        });
    }

    if profile.id == CUSTOM_PROFILE_ID {
        let command = startup_command
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                TabbyError::Validation(String::from("Custom profile requires a startup command"))
            })?;

        return Ok(ResolvedProfile {
            id: profile.id,
            label: profile.label,
            startup_command: Some(command),
        });
    }

    Ok(ResolvedProfile {
        id: profile.id,
        label: profile.label,
        startup_command: profile.startup_command,
    })
}
