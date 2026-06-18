use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "ccsm",
    about = "Claude Code Skill Marketplace — discover and install skills from Git marketplaces",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Manage skill marketplaces
    #[command(subcommand)]
    Marketplace(MarketplaceCommand),
    /// Search for skills across all marketplaces
    Search(SearchArgs),
    /// Install a skill from a marketplace
    Install(InstallArgs),
    /// List installed skills in the local store
    List,
    /// Remove an installed skill
    Remove(RemoveArgs),
    /// Update installed skills
    Update(UpdateArgs),
    /// Show details about an installed skill
    Info(InfoArgs),
}

#[derive(Subcommand)]
pub enum MarketplaceCommand {
    /// Add a new skill marketplace
    Add(MarketplaceAddArgs),
    /// Remove a marketplace
    Remove(MarketplaceRemoveArgs),
    /// List configured marketplaces
    List,
    /// Update marketplace catalogues
    Update(MarketplaceUpdateArgs),
}

#[derive(Args)]
pub struct MarketplaceAddArgs {
    /// Git repository URL of the marketplace
    pub url: String,
    /// Custom name for this marketplace (defaults to repo name)
    #[arg(short, long)]
    pub name: Option<String>,
}

#[derive(Args)]
pub struct MarketplaceRemoveArgs {
    /// Name of the marketplace to remove
    pub name: String,
}

#[derive(Args)]
pub struct MarketplaceUpdateArgs {
    /// Name of the marketplace to update (updates all if omitted)
    pub name: Option<String>,
}

#[derive(Args)]
pub struct SearchArgs {
    /// Filter skills by keyword
    pub query: Option<String>,
}

#[derive(Args)]
pub struct InstallArgs {
    /// Name of the skill to install
    pub name: String,
    /// Specify which marketplace to use (required when skill exists in multiple)
    #[arg(short, long)]
    pub marketplace: Option<String>,
    /// Copy skill files into Claude skills directory (default)
    #[arg(long)]
    pub copy: bool,
    /// Create a symlink into Claude skills directory instead of copying
    #[arg(long)]
    pub link: bool,
}

#[derive(Args)]
pub struct RemoveArgs {
    /// Name of the skill to remove
    pub name: String,
}

#[derive(Args)]
pub struct UpdateArgs {
    /// Name of the skill to update (updates all if omitted)
    pub name: Option<String>,
}

#[derive(Args)]
pub struct InfoArgs {
    /// Name of the skill
    pub name: String,
}
