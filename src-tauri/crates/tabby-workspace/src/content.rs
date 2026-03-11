use tabby_kernel::CommandTemplate;

use crate::ids::{BrowserUrl, PaneContentId};

/// Describes what runs inside a pane slot — separated from the workspace structural Pane type.
///
/// Each `PaneContentDefinition` has 1:1 ownership with a `PaneSlot`:
/// - Each instance belongs to exactly one pane
/// - Never shared between panes
/// - Never reused after destruction
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaneContentDefinition {
    Terminal {
        id: PaneContentId,
        profile_id: String,
        working_directory: String,
        command_override: Option<CommandTemplate>,
    },
    Browser {
        id: PaneContentId,
        initial_url: BrowserUrl,
    },
    Git {
        id: PaneContentId,
        working_directory: String,
    },
}

impl PaneContentDefinition {
    /// Creates a new terminal content definition with a unique ID.
    pub fn terminal(
        id: PaneContentId,
        profile_id: impl Into<String>,
        working_directory: impl Into<String>,
        command_override: Option<CommandTemplate>,
    ) -> Self {
        Self::Terminal {
            id,
            profile_id: profile_id.into(),
            working_directory: working_directory.into(),
            command_override,
        }
    }

    /// Creates a new browser content definition with a unique ID.
    pub fn browser(id: PaneContentId, initial_url: BrowserUrl) -> Self {
        Self::Browser { id, initial_url }
    }

    /// Creates a new git content definition with a unique ID.
    pub fn git(id: PaneContentId, working_directory: impl Into<String>) -> Self {
        Self::Git {
            id,
            working_directory: working_directory.into(),
        }
    }

    /// Returns the content ID for this definition.
    pub fn content_id(&self) -> &PaneContentId {
        match self {
            Self::Terminal { id, .. } | Self::Browser { id, .. } | Self::Git { id, .. } => id,
        }
    }

    /// Returns the profile ID if this is a terminal content definition.
    pub fn terminal_profile_id(&self) -> Option<&str> {
        match self {
            Self::Terminal { profile_id, .. } => Some(profile_id.as_str()),
            Self::Browser { .. } | Self::Git { .. } => None,
        }
    }

    /// Returns the working directory if this is a terminal content definition.
    pub fn working_directory(&self) -> Option<&str> {
        match self {
            Self::Terminal {
                working_directory, ..
            } => Some(working_directory.as_str()),
            Self::Git {
                working_directory, ..
            } => Some(working_directory.as_str()),
            Self::Browser { .. } => None,
        }
    }

    /// Returns the browser URL if this is a browser content definition.
    pub fn browser_url(&self) -> Option<&BrowserUrl> {
        match self {
            Self::Browser { initial_url, .. } => Some(initial_url),
            Self::Terminal { .. } | Self::Git { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::PaneContentId;

    fn make_content_id(label: &str) -> PaneContentId {
        PaneContentId::from(String::from(label))
    }

    #[test]
    fn terminal_construction_and_field_access() {
        let id = make_content_id("content-1");
        let def = PaneContentDefinition::terminal(
            id.clone(),
            "zsh-profile",
            "/home/user",
            Some(CommandTemplate::new("vim")),
        );

        assert_eq!(*def.content_id(), id);
        assert_eq!(def.terminal_profile_id(), Some("zsh-profile"));
        assert_eq!(def.working_directory(), Some("/home/user"));
        assert_eq!(def.browser_url(), None);

        match &def {
            PaneContentDefinition::Terminal {
                command_override, ..
            } => {
                assert_eq!(command_override.as_ref().map(|c| c.as_str()), Some("vim"));
            }
            _ => panic!("expected Terminal variant"),
        }
    }

    #[test]
    fn terminal_without_command_override() {
        let id = make_content_id("content-2");
        let def = PaneContentDefinition::terminal(id.clone(), "bash", "/tmp", None);

        assert_eq!(*def.content_id(), id);
        assert_eq!(def.terminal_profile_id(), Some("bash"));
        assert_eq!(def.working_directory(), Some("/tmp"));

        match &def {
            PaneContentDefinition::Terminal {
                command_override, ..
            } => {
                assert!(command_override.is_none());
            }
            _ => panic!("expected Terminal variant"),
        }
    }

    #[test]
    fn browser_construction_and_field_access() {
        let id = make_content_id("content-3");
        let url = BrowserUrl::new("https://example.com");
        let def = PaneContentDefinition::browser(id.clone(), url);

        assert_eq!(*def.content_id(), id);
        assert_eq!(
            def.browser_url().map(|u| u.as_str()),
            Some("https://example.com")
        );
        assert_eq!(def.terminal_profile_id(), None);
        assert_eq!(def.working_directory(), None);
    }

    #[test]
    fn browser_url_display_and_as_ref() {
        let url = BrowserUrl::new("https://tabby.dev");
        assert_eq!(url.to_string(), "https://tabby.dev");
        assert_eq!(url.as_ref(), "https://tabby.dev");
        assert_eq!(url.as_str(), "https://tabby.dev");
    }

    #[test]
    fn two_panes_get_distinct_content_ids() {
        let id_a = make_content_id("content-a");
        let id_b = make_content_id("content-b");

        let def_a = PaneContentDefinition::terminal(id_a.clone(), "zsh", "/home", None);
        let def_b = PaneContentDefinition::terminal(id_b.clone(), "zsh", "/home", None);

        assert_ne!(def_a.content_id(), def_b.content_id());
        assert_ne!(def_a, def_b);
    }

    #[test]
    fn same_content_id_produces_equal_definitions() {
        let id = make_content_id("content-same");

        let def_a = PaneContentDefinition::terminal(id.clone(), "zsh", "/home", None);
        let def_b = PaneContentDefinition::terminal(id, "zsh", "/home", None);

        assert_eq!(def_a, def_b);
    }

    #[test]
    fn content_id_is_never_shared_between_terminal_and_browser() {
        let terminal_id = make_content_id("terminal-content");
        let browser_id = make_content_id("browser-content");

        let terminal = PaneContentDefinition::terminal(terminal_id, "zsh", "/home", None);
        let browser =
            PaneContentDefinition::browser(browser_id, BrowserUrl::new("https://example.com"));

        assert_ne!(terminal.content_id(), browser.content_id());
    }

    #[test]
    fn clone_preserves_all_fields() {
        let id = make_content_id("content-clone");
        let def = PaneContentDefinition::terminal(
            id,
            "fish",
            "/var/log",
            Some(CommandTemplate::new("tail -f")),
        );
        let cloned = def.clone();

        assert_eq!(def, cloned);
    }

    #[test]
    fn debug_format_is_readable() {
        let id = make_content_id("dbg-test");
        let def = PaneContentDefinition::browser(id, BrowserUrl::new("https://rust-lang.org"));
        let debug = format!("{def:?}");

        assert!(debug.contains("Browser"));
        assert!(debug.contains("rust-lang.org"));
    }

    #[test]
    fn content_definition_does_not_import_structural_types() {
        let id = make_content_id("boundary-test");
        let _def = PaneContentDefinition::terminal(id, "sh", "/", None);
    }

    #[test]
    fn git_construction_and_field_access() {
        let id = make_content_id("git-content-1");
        let def = PaneContentDefinition::git(id.clone(), "/my/repo");

        assert_eq!(*def.content_id(), id);
        assert_eq!(def.working_directory(), Some("/my/repo"));
        assert_eq!(def.terminal_profile_id(), None);
        assert_eq!(def.browser_url(), None);

        match &def {
            PaneContentDefinition::Git { working_directory, .. } => {
                assert_eq!(working_directory, "/my/repo");
            }
            _ => panic!("expected Git variant"),
        }
    }

    #[test]
    fn git_clone_preserves_all_fields() {
        let id = make_content_id("git-clone");
        let def = PaneContentDefinition::git(id, "/clone/path");
        let cloned = def.clone();
        assert_eq!(def, cloned);
    }

    #[test]
    fn git_and_terminal_have_distinct_working_directory_semantics() {
        let git_id = make_content_id("git-wd");
        let term_id = make_content_id("term-wd");
        let git_def = PaneContentDefinition::git(git_id, "/shared");
        let term_def = PaneContentDefinition::terminal(term_id, "sh", "/shared", None);

        // Both have working_directory but are different variants
        assert_eq!(git_def.working_directory(), Some("/shared"));
        assert_eq!(term_def.working_directory(), Some("/shared"));
        assert_ne!(git_def, term_def);
    }

    #[test]
    fn git_debug_format_is_readable() {
        let id = make_content_id("git-debug");
        let def = PaneContentDefinition::git(id, "/debug/repo");
        let debug = format!("{def:?}");
        assert!(debug.contains("Git"));
        assert!(debug.contains("debug/repo"));
    }

    #[test]
    fn content_id_returns_same_id_for_all_variants() {
        let term_id = make_content_id("term-id-check");
        let browser_id = make_content_id("browser-id-check");
        let git_id = make_content_id("git-id-check");

        let term = PaneContentDefinition::terminal(term_id.clone(), "zsh", "/", None);
        let browser = PaneContentDefinition::browser(browser_id.clone(), BrowserUrl::new("https://x.com"));
        let git = PaneContentDefinition::git(git_id.clone(), "/");

        assert_eq!(*term.content_id(), term_id);
        assert_eq!(*browser.content_id(), browser_id);
        assert_eq!(*git.content_id(), git_id);
    }

    #[test]
    fn terminal_working_directory_different_from_git_variant() {
        let term_id = make_content_id("t1");
        let git_id = make_content_id("g1");
        let term = PaneContentDefinition::terminal(term_id, "sh", "/home", None);
        let git = PaneContentDefinition::git(git_id, "/repo");
        // terminal has profile_id, git does not
        assert!(term.terminal_profile_id().is_some());
        assert!(git.terminal_profile_id().is_none());
    }

    #[test]
    fn browser_equality_same_url_and_id() {
        let id = make_content_id("browser-eq");
        let url = BrowserUrl::new("https://example.com");
        let def_a = PaneContentDefinition::browser(id.clone(), url.clone());
        let def_b = PaneContentDefinition::browser(id, url);
        assert_eq!(def_a, def_b);
    }

    #[test]
    fn browser_inequality_different_url() {
        let id = make_content_id("browser-ne");
        let def_a = PaneContentDefinition::browser(id.clone(), BrowserUrl::new("https://a.com"));
        let def_b = PaneContentDefinition::browser(id, BrowserUrl::new("https://b.com"));
        assert_ne!(def_a, def_b);
    }
}
