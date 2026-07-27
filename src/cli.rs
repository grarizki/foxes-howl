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
    /// Scan a repository for contribution opportunities (good first issues)
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

        /// Skip local cache
        #[arg(long)]
        no_cache: bool,
    },

    /// Find stale issues and PRs (no activity for N days)
    Stale {
        /// Repository in owner/repo format
        #[arg(value_name = "OWNER/REPO")]
        repo: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Max results to show
        #[arg(long, default_value_t = 25)]
        limit: usize,

        /// Stale threshold in days
        #[arg(long, default_value_t = 30)]
        days: u32,
    },

    /// Analyze README and community health files
    Readme {
        /// Repository in owner/repo format
        #[arg(value_name = "OWNER/REPO")]
        repo: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Analyze code quality signals (TODOs, CI, tests, lint)
    Quality {
        /// Repository in owner/repo format
        #[arg(value_name = "OWNER/REPO")]
        repo: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Launch interactive TUI dashboard
    Tui {
        /// Repos to analyze (owner/repo ...)
        #[arg(value_name = "REPO")]
        repos: Vec<String>,
    },

    /// Create default config file
    Init,
}

/// Parse "owner/repo" into (owner, repo)
pub fn parse_repo(input: &str) -> anyhow::Result<(String, String)> {
    let parts: Vec<&str> = input.split('/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        anyhow::bail!("Invalid repo format '{}'. Expected: owner/repo", input);
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_repo_valid() {
        let (owner, repo) = parse_repo("rust-lang/rust").unwrap();
        assert_eq!(owner, "rust-lang");
        assert_eq!(repo, "rust");
    }

    #[test]
    fn test_parse_repo_invalid() {
        assert!(parse_repo("invalid").is_err());
        assert!(parse_repo("a/b/c").is_err());
        assert!(parse_repo("/").is_err());
        assert!(parse_repo("a/").is_err());
        assert!(parse_repo("/b").is_err());
    }

    #[test]
    fn test_parse_repo_with_dashes() {
        let (owner, repo) = parse_repo("my-org/my-repo").unwrap();
        assert_eq!(owner, "my-org");
        assert_eq!(repo, "my-repo");
    }
}
