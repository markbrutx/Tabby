use std::path::PathBuf;

use portable_pty::CommandBuilder;

use crate::settings::domain::profiles::ResolvedProfile;

pub fn default_shell() -> String {
    std::env::var("SHELL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| String::from("/bin/zsh"))
}

pub fn build_pty_command(cwd: &str, profile: &ResolvedProfile) -> CommandBuilder {
    let shell = default_shell();

    let startup_command = profile
        .startup_command
        .as_deref()
        .map(|v| v.trim())
        .filter(|v| !v.is_empty());

    let mut command = match startup_command {
        Some(cmd) => {
            let mut cb = CommandBuilder::new(&shell);
            cb.arg("-l");
            cb.arg("-c");
            cb.arg(format!("exec {cmd}"));
            cb
        }
        None => {
            let mut cb = CommandBuilder::new(&shell);
            cb.arg("-l");
            cb
        }
    };

    command.env("TERM", "xterm-256color");
    command.env_remove("CLAUDECODE");
    command.env_remove("CLAUDE_CODE_ENTRYPOINT");

    if !cwd.trim().is_empty() {
        command.cwd(PathBuf::from(cwd));
    }

    command
}

#[cfg(test)]
mod tests {
    use super::build_pty_command;
    use crate::settings::domain::profiles::ResolvedProfile;

    #[test]
    fn terminal_profile_builds_login_shell() {
        let profile = ResolvedProfile {
            id: String::from("terminal"),
            label: String::from("Terminal"),
            startup_command: None,
        };
        let cmd = build_pty_command("/tmp", &profile);
        let argv = cmd.get_argv();
        assert_eq!(argv.len(), 2);
        assert_eq!(argv[1].to_str().unwrap(), "-l");
    }

    #[test]
    fn profile_with_command_builds_shell_c() {
        let profile = ResolvedProfile {
            id: String::from("claude"),
            label: String::from("Claude Code"),
            startup_command: Some(String::from("claude")),
        };
        let cmd = build_pty_command("/tmp", &profile);
        let argv = cmd.get_argv();
        assert_eq!(argv.len(), 4);
        assert_eq!(argv[1].to_str().unwrap(), "-l");
        assert_eq!(argv[2].to_str().unwrap(), "-c");
        assert_eq!(argv[3].to_str().unwrap(), "exec claude");
    }

    #[test]
    fn empty_command_uses_login_shell() {
        let profile = ResolvedProfile {
            id: String::from("terminal"),
            label: String::from("Terminal"),
            startup_command: Some(String::from("  ")),
        };
        let cmd = build_pty_command("/tmp", &profile);
        let argv = cmd.get_argv();
        assert_eq!(argv.len(), 2);
        assert_eq!(argv[1].to_str().unwrap(), "-l");
    }

    #[test]
    fn env_vars_are_set() {
        let profile = ResolvedProfile {
            id: String::from("terminal"),
            label: String::from("Terminal"),
            startup_command: None,
        };
        let cmd = build_pty_command("/tmp", &profile);
        let term = cmd.get_env("TERM");
        assert_eq!(term.map(|v| v.to_str().unwrap()), Some("xterm-256color"));
    }
}
