mod ai;
mod analysis;
mod cli;
mod config;
mod db;
mod github;
mod hooks;
mod security;
mod serve;
mod tui;

use anyhow::Context;
use clap::Parser;
use cli::{Cli, Commands};
use comfy_table::{presets::UTF8_FULL, Table};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    let cfg = config::Config::load();

    match cli.command {
        Commands::Scan {
            repo,
            json,
            limit,
            no_cache,
        } => {
            let (owner, name) = cli::parse_repo(&repo)?;
            let client = github::build_client().context("Failed to build GitHub client")?;

            let issues;
            let cache = if !no_cache {
                db::Cache::open().ok()
            } else {
                None
            };

            // try cache first
            if let Some(ref cache) = cache {
                if let Ok(cached) = cache.load(&repo, 3600) {
                    if !cached.is_empty() {
                        issues = cached;
                        tracing::info!("Loaded {} issues from cache", issues.len());
                        if json {
                            println!("{}", serde_json::to_string_pretty(&issues)?);
                        } else {
                            print_issues_table(&owner, &name, &issues);
                        }
                        return Ok(());
                    }
                }
            }

            issues = github::issues::fetch_and_score(&client, &owner, &name, limit)
                .await
                .context("Failed to fetch issues")?;

            // store in cache
            if let Some(ref cache) = cache {
                cache.store(&repo, &issues).ok();
            }

            if json {
                println!("{}", serde_json::to_string_pretty(&issues)?);
            } else {
                print_issues_table(&owner, &name, &issues);
            }
        }

        Commands::Stale {
            repo,
            json,
            limit,
            days,
        } => {
            let (owner, name) = cli::parse_repo(&repo)?;
            let client = github::build_client().context("Failed to build GitHub client")?;

            let stale_issues = analysis::stale::find_stale_issues(&client, &owner, &name, days, limit)
                .await
                .context("Failed to fetch stale issues")?;
            let stale_prs = analysis::stale::find_stale_prs(&client, &owner, &name, days, limit)
                .await
                .context("Failed to fetch stale PRs")?;

            if json {
                let output = serde_json::json!({
                    "issues": stale_issues,
                    "pull_requests": stale_prs,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                print_stale_table(&owner, &name, &stale_issues, &stale_prs, days);
            }
        }

        Commands::Readme { repo, json } => {
            let (owner, name) = cli::parse_repo(&repo)?;
            let client = github::build_client().context("Failed to build GitHub client")?;

            let report = analysis::readme::analyze_repo(&client, &owner, &name)
                .await
                .context("Failed to analyze README")?;

            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_readme_report(&owner, &name, &report);
            }
        }

        Commands::Quality { repo, json } => {
            let (owner, name) = cli::parse_repo(&repo)?;
            let client = github::build_client().context("Failed to build GitHub client")?;

            let report = analysis::code_quality::analyze_repo(&client, &owner, &name)
                .await
                .context("Failed to analyze code quality")?;

            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_quality_report(&owner, &name, &report);
            }
        }

        Commands::Discover {
            lang,
            topic,
            min_stars,
            limit,
            json,
            no_cache: _,
        } => {
            let client = github::build_client().context("Failed to build GitHub client")?;

            let repos =
                github::discover::discover_repos(&client, lang.as_deref(), topic.as_deref(), min_stars, limit)
                    .await
                    .context("Failed to discover repos")?;

            if json {
                println!("{}", serde_json::to_string_pretty(&repos)?);
            } else {
                print_discovered_repos(&repos);
            }
        }

        Commands::Tui { repos } => {
            let client = github::build_client().context("Failed to build GitHub client")?;

            let mut all_issues = Vec::new();
            let mut all_stale = Vec::new();
            let mut all_repo_scores = Vec::new();

            let repo_list: Vec<String> = if repos.is_empty() {
                vec!["rust-lang/rust".to_string()]
            } else {
                repos
            };

            for repo_str in &repo_list {
                let (owner, name) = cli::parse_repo(repo_str)?;

                let issues = github::issues::fetch_and_score(&client, &owner, &name, 50)
                    .await
                    .unwrap_or_default();
                let stale =
                    analysis::stale::find_stale_issues(&client, &owner, &name, cfg.scoring.stale_days, 50)
                        .await
                        .unwrap_or_default();
                let readme = analysis::readme::analyze_repo(&client, &owner, &name)
                    .await
                    .unwrap_or_else(|_| analysis::readme::ReadmeReport {
                        has_readme: false,
                        has_contributing: false,
                        has_code_of_conduct: false,
                        has_license: false,
                        has_issue_template: false,
                        has_pr_template: false,
                        has_build_instructions: false,
                        broken_links: vec![],
                        score: 0.0,
                    });
                let quality = analysis::code_quality::analyze_repo(&client, &owner, &name)
                    .await
                    .unwrap_or_else(|_| analysis::code_quality::CodeQualityReport {
                        todo_count: 0,
                        fixme_count: 0,
                        hack_count: 0,
                        has_ci: false,
                        has_lint_config: false,
                        has_test_dir: false,
                        score: 0.0,
                    });

                let score = analysis::scoring::build_repo_score(
                    &cfg.scoring,
                    repo_str,
                    &issues,
                    stale.len(),
                    &readme,
                    &quality,
                );

                all_issues.extend(issues);
                all_stale.extend(stale);
                all_repo_scores.push(score);
            }

            tui::run_tui(all_issues, all_stale, all_repo_scores)?;
        }

        Commands::Init => {
            let path = config::Config::init()?;
            println!("Config created at: {}", path.display());
        }

        Commands::Ai { action } => {
            match action {
                cli::AiAction::Analyze { repo, yes } => {
                    let provider = ai::build_provider(&cfg.ai)?;
                    let (owner, name) = cli::parse_repo(&repo)?;
                    let client = github::build_client().context("Failed to build GitHub client")?;

                    let issues = github::issues::fetch_and_score(&client, &owner, &name, 25)
                        .await
                        .unwrap_or_default();
                    let stale_count = analysis::stale::find_stale_issues(
                        &client, &owner, &name, cfg.scoring.stale_days, 50,
                    )
                    .await
                    .map(|s| s.len())
                    .unwrap_or(0);
                    let readme = analysis::readme::analyze_repo(&client, &owner, &name)
                        .await
                        .map(|r| r.score)
                        .unwrap_or(0.0);
                    let quality = analysis::code_quality::analyze_repo(&client, &owner, &name)
                        .await
                        .map(|r| r.score)
                        .unwrap_or(0.0);

                    let issues_json = serde_json::to_string(&issues)?;
                    let (system, user) = ai::prompts::build_analyze_prompt(
                        &repo, &issues_json, stale_count, readme, quality,
                    );

                    let est = ai::estimate::estimate(&system, &user, &cfg.ai.model);
                    if !yes {
                        println!("{}", ai::estimate::format_estimate(&est));
                        print!("Proceed? [Y/n] ");
                        use std::io::Write;
                        std::io::stdout().flush()?;
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;
                        if input.trim().to_lowercase() == "n" {
                            println!("Aborted.");
                            return Ok(());
                        }
                    }

                    let response = provider.complete(&system, &user).await?;
                    println!("{}", response);
                }

                cli::AiAction::Recommend { repo, skills, hours, yes } => {
                    let provider = ai::build_provider(&cfg.ai)?;
                    let (owner, name) = cli::parse_repo(&repo)?;
                    let client = github::build_client().context("Failed to build GitHub client")?;

                    let issues = github::issues::fetch_and_score(&client, &owner, &name, 25)
                        .await
                        .unwrap_or_default();
                    let stale = analysis::stale::find_stale_issues(
                        &client, &owner, &name, cfg.scoring.stale_days, 50,
                    )
                    .await
                    .unwrap_or_default();

                    let profile = config::UserProfile {
                        name: cfg.ai.profile.name.clone(),
                        skills: skills
                            .map(|s| s.split(',').map(|x| x.trim().to_string()).collect())
                            .unwrap_or_else(|| cfg.ai.profile.skills.clone()),
                        experience: cfg.ai.profile.experience.clone(),
                        hours_per_week: hours,
                        interests: cfg.ai.profile.interests.clone(),
                    };

                    let issues_json = serde_json::to_string(&issues)?;
                    let stale_json = serde_json::to_string(&stale)?;
                    let (system, user) = ai::prompts::build_recommend_prompt(
                        &repo, &profile, &issues_json, &stale_json,
                    );

                    let est = ai::estimate::estimate(&system, &user, &cfg.ai.model);
                    if !yes {
                        println!("{}", ai::estimate::format_estimate(&est));
                        print!("Proceed? [Y/n] ");
                        use std::io::Write;
                        std::io::stdout().flush()?;
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;
                        if input.trim().to_lowercase() == "n" {
                            println!("Aborted.");
                            return Ok(());
                        }
                    }

                    let response = provider.complete(&system, &user).await?;
                    println!("{}", response);
                }

                cli::AiAction::Difficulty { repo, yes } => {
                    let provider = ai::build_provider(&cfg.ai)?;
                    let (owner, name) = cli::parse_repo(&repo)?;
                    let client = github::build_client().context("Failed to build GitHub client")?;

                    let issues = github::issues::fetch_and_score(&client, &owner, &name, 25)
                        .await
                        .unwrap_or_default();

                    let issues_json = serde_json::to_string(&issues)?;
                    let (system, user) = ai::prompts::build_difficulty_prompt(&repo, &issues_json);

                    let est = ai::estimate::estimate(&system, &user, &cfg.ai.model);
                    if !yes {
                        println!("{}", ai::estimate::format_estimate(&est));
                        print!("Proceed? [Y/n] ");
                        use std::io::Write;
                        std::io::stdout().flush()?;
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;
                        if input.trim().to_lowercase() == "n" {
                            println!("Aborted.");
                            return Ok(());
                        }
                    }

                    let response = provider.complete(&system, &user).await?;
                    println!("{}", response);
                }
            }
        }

        Commands::Tools => {
            let tools = ai::tools::definitions();
            println!("{}", serde_json::to_string_pretty(&tools)?);
        }

        Commands::Call { tool, args } => {
            let _args: serde_json::Value = serde_json::from_str(&args)?;
            match tool.as_str() {
                "discover_repos" | "scan_issues" | "analyze_repo" | "ai_recommend" => {
                    println!("{}", serde_json::json!({
                        "error": format!("Tool '{}' dispatch via 'call' not yet implemented. Use the specific CLI command instead.", tool),
                        "hint": format!("Try: gh-opportunities {} --help", match tool.as_str() {
                            "discover_repos" => "discover",
                            "scan_issues" => "scan",
                            "analyze_repo" => "readme",
                            "ai_recommend" => "ai recommend",
                            _ => "help",
                        })
                    }));
                }
                _ => {
                    println!("{}", serde_json::json!({
                        "error": format!("Unknown tool '{}'", tool),
                        "available_tools": ["discover_repos", "scan_issues", "analyze_repo", "ai_recommend"]
                    }));
                }
            }
        }

        Commands::Serve { port } => {
            let token_env = cfg.serve.token_env.clone();
            serve::run_server(port, &token_env, cfg).await?;
        }

        Commands::Security { json, check, fix } => {
            if fix {
                // Auto-fix: only cargo fmt for now
                println!("Running cargo fmt...");
                let output = tokio::process::Command::new("cargo")
                    .args(["fmt"])
                    .output()
                    .await;
                match output {
                    Ok(o) if o.status.success() => println!("Formatted."),
                    Ok(o) => {
                        let stderr = String::from_utf8_lossy(&o.stderr);
                        println!("cargo fmt failed: {}", stderr);
                    }
                    Err(e) => println!("cargo fmt not available: {}", e),
                }
                return Ok(());
            }

            let report = match check.as_deref() {
                Some("audit") => {
                    let check = security::audit::run_audit().await;
                    security::SecurityReport::from_checks(vec![check])
                }
                Some("secrets") => {
                    let check = security::secrets::scan_secrets(&cfg.security.secret_patterns);
                    security::SecurityReport::from_checks(vec![check])
                }
                Some("quality") => {
                    let check = security::quality::run_quality_gate().await;
                    security::SecurityReport::from_checks(vec![check])
                }
                Some("license") => {
                    let check = security::license::run_license_check(
                        cfg.security.deny_config_path.as_deref(),
                    )
                    .await;
                    security::SecurityReport::from_checks(vec![check])
                }
                Some(other) => {
                    eprintln!("Unknown check '{}'. Available: audit, secrets, quality, license", other);
                    return Ok(());
                }
                None => {
                    security::run_all(
                        cfg.security.deny_config_path.as_deref(),
                        &cfg.security.secret_patterns,
                    )
                    .await
                }
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("{}", report.summary);
                for check in &report.checks {
                    let status = if !check.tool_available {
                        "UNAVAILABLE"
                    } else if check.passed {
                        "PASS"
                    } else {
                        "FAIL"
                    };
                    println!("  {}: {} ({} finding(s))", check.name, status, check.findings.len());
                    for finding in &check.findings {
                        println!("    [{}] {}", finding.severity, finding.message);
                        if let Some(fix) = &finding.fix {
                            println!("      Fix: {}", fix);
                        }
                    }
                }
            }

            if !report.passed {
                std::process::exit(1);
            }
        }

        Commands::Hooks { action } => {
            match action {
                cli::HooksAction::Install => {
                    let path = hooks::install_hook()?;
                    println!("Pre-push hook installed at {}", path.display());
                }
                cli::HooksAction::Remove => {
                    let path = hooks::remove_hook()?;
                    println!("Pre-push hook removed from {}", path.display());
                }
            }
        }
    }

    Ok(())
}

fn print_issues_table(owner: &str, repo: &str, issues: &[github::issues::ScoredIssue]) {
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

        table.add_row(vec![
            format!("{:.1}", issue.score),
            issue.number.to_string(),
            truncate(&issue.title, 50),
            truncate(&labels_display, 30),
            assigned.to_string(),
            format!("{}d ago", days_ago),
        ]);
    }

    println!("{table}");
    println!("\nTop match: {}", issues[0].url);
}

fn print_stale_table(
    owner: &str,
    repo: &str,
    issues: &[analysis::stale::StaleItem],
    prs: &[analysis::stale::StaleItem],
    threshold: u32,
) {
    println!(
        "\n  Stale items in {}/{} (threshold: {} days)\n",
        owner, repo, threshold
    );

    if !issues.is_empty() {
        println!("  Stale Issues ({}):\n", issues.len());
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec!["Severity", "#", "Title", "Days", "Assigned"]);
        for item in issues {
            table.add_row(vec![
                format!("{:.2}", item.stale_severity),
                item.number.to_string(),
                truncate(&item.title, 45),
                item.last_activity_days.to_string(),
                if item.has_assignee { "Yes" } else { "No" }.to_string(),
            ]);
        }
        println!("{table}");
    }

    if !prs.is_empty() {
        println!("\n  Stale PRs ({}):\n", prs.len());
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec!["Severity", "#", "Title", "Days", "Assigned"]);
        for item in prs {
            table.add_row(vec![
                format!("{:.2}", item.stale_severity),
                item.number.to_string(),
                truncate(&item.title, 45),
                item.last_activity_days.to_string(),
                if item.has_assignee { "Yes" } else { "No" }.to_string(),
            ]);
        }
        println!("{table}");
    }

    if issues.is_empty() && prs.is_empty() {
        println!("No stale items found!");
    }
}

fn print_readme_report(owner: &str, repo: &str, report: &analysis::readme::ReadmeReport) {
    println!(
        "\n  README Analysis for {}/{} (score: {:.0}%)\n",
        owner,
        repo,
        report.score * 100.0
    );

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Check", "Status"]);
    table.add_row(vec![
        "README.md".to_string(),
        status_icon(report.has_readme),
    ]);
    table.add_row(vec![
        "CONTRIBUTING.md".to_string(),
        status_icon(report.has_contributing),
    ]);
    table.add_row(vec![
        "CODE_OF_CONDUCT.md".to_string(),
        status_icon(report.has_code_of_conduct),
    ]);
    table.add_row(vec![
        "LICENSE".to_string(),
        status_icon(report.has_license),
    ]);
    table.add_row(vec![
        "Issue Template".to_string(),
        status_icon(report.has_issue_template),
    ]);
    table.add_row(vec![
        "PR Template".to_string(),
        status_icon(report.has_pr_template),
    ]);
    table.add_row(vec![
        "Build Instructions".to_string(),
        status_icon(report.has_build_instructions),
    ]);
    println!("{table}");

    if !report.broken_links.is_empty() {
        println!("\n  Possible broken links:");
        for link in &report.broken_links {
            println!("    {}", link);
        }
    }
}

fn print_quality_report(owner: &str, repo: &str, report: &analysis::code_quality::CodeQualityReport) {
    println!(
        "\n  Code Quality Analysis for {}/{} (score: {:.0}%)\n",
        owner,
        repo,
        report.score * 100.0
    );

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Check", "Value"]);
    table.add_row(vec!["TODO count".to_string(), report.todo_count.to_string()]);
    table.add_row(vec!["FIXME count".to_string(), report.fixme_count.to_string()]);
    table.add_row(vec!["HACK count".to_string(), report.hack_count.to_string()]);
    table.add_row(vec!["CI Config".to_string(), status_icon(report.has_ci)]);
    table.add_row(vec![
        "Lint Config".to_string(),
        status_icon(report.has_lint_config),
    ]);
    table.add_row(vec![
        "Test Directory".to_string(),
        status_icon(report.has_test_dir),
    ]);
    println!("{table}");
}

fn status_icon(ok: bool) -> String {
    if ok {
        "OK".to_string()
    } else {
        "MISSING".to_string()
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

fn print_discovered_repos(repos: &[github::discover::DiscoveredRepo]) {
    if repos.is_empty() {
        println!("No repos found matching criteria");
        return;
    }

    println!("\n  Discovered repos with contribution opportunities\n");

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Score", "Repo", "Language", "Stars", "Good First Issues"]);

    for repo in repos {
        table.add_row(vec![
            format!("{:.1}", repo.score),
            truncate(&repo.full_name, 30),
            repo.language.as_deref().unwrap_or("-").to_string(),
            repo.stars.to_string(),
            repo.good_first_issues.to_string(),
        ]);
    }

    println!("{table}");
    println!("\nTop match: {}", repos[0].url);
}
