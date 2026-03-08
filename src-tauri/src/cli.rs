use clap::Parser;

#[derive(Debug, Clone, Default, Parser)]
#[command(author, version, about = "Terminal workspace shell for Tabby")]
pub struct CliArgs {
    #[arg(long)]
    pub new_tab: bool,
    #[arg(long)]
    pub layout: Option<String>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long)]
    pub cwd: Option<String>,
    #[arg(long)]
    pub command: Option<String>,
}

impl CliArgs {
    pub fn from_argv(argv: &[String]) -> Result<Self, clap::Error> {
        Self::try_parse_from(std::iter::once(String::from("tabby")).chain(argv.iter().cloned()))
            .or_else(|error| {
                if argv.first().is_some_and(|arg| !arg.starts_with('-')) {
                    Self::try_parse_from(argv.iter().cloned())
                } else {
                    Err(error)
                }
            })
    }

    pub fn has_launch_overrides(&self) -> bool {
        self.new_tab
            || self.layout.is_some()
            || self.profile.is_some()
            || self.cwd.is_some()
            || self.command.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::CliArgs;

    #[test]
    fn default_cli_args_do_not_count_as_launch_overrides() {
        assert!(!CliArgs::default().has_launch_overrides());
    }

    #[test]
    fn explicit_overrides_are_detected() {
        assert!(CliArgs {
            new_tab: true,
            ..CliArgs::default()
        }
        .has_launch_overrides());
        assert!(CliArgs {
            profile: Some(String::from("codex")),
            ..CliArgs::default()
        }
        .has_launch_overrides());
        assert!(CliArgs {
            cwd: Some(String::from("/tmp")),
            ..CliArgs::default()
        }
        .has_launch_overrides());
    }

    #[test]
    fn parses_single_instance_arguments_with_or_without_binary_name() {
        let without_binary = vec![
            String::from("--new-tab"),
            String::from("--profile"),
            String::from("codex"),
        ];
        let parsed = CliArgs::from_argv(&without_binary).expect("args should parse");
        assert!(parsed.new_tab);
        assert_eq!(parsed.profile.as_deref(), Some("codex"));

        let with_binary = vec![
            String::from("tabby"),
            String::from("--cwd"),
            String::from("/tmp"),
        ];
        let parsed = CliArgs::from_argv(&with_binary).expect("args should parse");
        assert_eq!(parsed.cwd.as_deref(), Some("/tmp"));
    }
}
