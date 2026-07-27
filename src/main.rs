mod cli;
mod github;

use anyhow::Context;
use clap::Parser;
use cli::{Cli, Commands};
use comfy_table::{presets::UTF8_FULL, Table};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { repo, json, limit } => {
            let (owner, name) = cli::parse_repo(&repo)?;
            let client = github::build_client()
                .context("Failed to build GitHub client")?;

            let issues = github::issues::fetch_and_score(&client, &owner, &name, limit)
                .await
                .context("Failed to fetch issues")?;

            if json {
                println!("{}", serde_json::to_string_pretty(&issues)?);
            } else {
                print_table(&owner, &name, &issues);
            }
        }
    }

    Ok(())
}

fn print_table(owner: &str, repo: &str, issues: &[github::issues::ScoredIssue]) {
    if issues.is_empty() {
        println!("No contribution opportunities found in {}/{}", owner, repo);
        return;
    }

    println!(
        "\n  Contribution opportunities in {}/{} ({} found)\n",
        owner,
        repo,
        issues.len()
    );

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Score", "#", "Title", "Labels", "Assigned", "Updated"]);

    for issue in issues {
        let labels_display = if issue.labels.is_empty() {
            "-".to_string()
        } else {
            issue.labels.join(", ")
        };
        let assigned = issue.assignee.as_deref().unwrap_or("-");
        let days_ago = (chrono::Utc::now() - issue.updated_at).num_days();
        let updated = format!("{}d ago", days_ago);

        table.add_row(vec![
            format!("{:.1}", issue.score),
            issue.number.to_string(),
            truncate(&issue.title, 50),
            truncate(&labels_display, 30),
            assigned.to_string(),
            updated,
        ]);
    }

    println!("{table}");
    println!("\nTop match: {}", issues[0].url);
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
