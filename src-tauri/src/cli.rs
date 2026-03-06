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
