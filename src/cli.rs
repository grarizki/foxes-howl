use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "gh-opp",
    version,
    about = "Find open source contribution opportunities on GitHub"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scan a repository for contribution opportunities
    Scan {
        /// Repository in owner/repo format
        #[arg(value_name = "OWNER/REPO")]
        repo: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Max results to show
        #[arg(long, default_value_t = 25)]
        limit: usize,
    },
}

/// Parse "owner/repo" into (owner, repo)
pub fn parse_repo(input: &str) -> anyhow::Result<(String, String)> {
    let parts: Vec<&str> = input.split('/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        anyhow::bail!("Invalid repo format '{}'. Expected: owner/repo", input);
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}
