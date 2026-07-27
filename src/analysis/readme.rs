use octocrab::Octocrab;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ReadmeReport {
    pub has_readme: bool,
    pub has_contributing: bool,
    pub has_code_of_conduct: bool,
    pub has_license: bool,
    pub has_issue_template: bool,
    pub has_pr_template: bool,
    pub has_build_instructions: bool,
    pub broken_links: Vec<String>,
    pub score: f64,
}

pub fn score_readme(report: &ReadmeReport) -> f64 {
    let mut score = 0.0;
    if report.has_readme {
        score += 0.3;
    }
    if report.has_contributing {
        score += 0.2;
    }
    if report.has_code_of_conduct {
        score += 0.1;
    }
    if report.has_license {
        score += 0.15;
    }
    if report.has_issue_template {
        score += 0.1;
    }
    if report.has_pr_template {
        score += 0.05;
    }
    if report.has_build_instructions {
        score += 0.1;
    }
    score
}

pub fn check_readme_content(readme_body: &str) -> ReadmeReport {
    let lower = readme_body.to_lowercase();
    let has_build_instructions = lower.contains("build")
        || lower.contains("install")
        || lower.contains("setup")
        || lower.contains("getting started")
        || lower.contains("prerequisites")
        || lower.contains("compilation");

    let broken_links: Vec<String> = readme_body
        .lines()
        .filter(|line| {
            let lower = line.to_lowercase();
            lower.contains("](") && !lower.contains("](http") && !lower.contains("](#") && !lower.contains("](mailto:")
        })
        .map(|l| l.trim().to_string())
        .collect();

    ReadmeReport {
        has_readme: true,
        has_contributing: false,
        has_code_of_conduct: false,
        has_license: false,
        has_issue_template: false,
        has_pr_template: false,
        has_build_instructions,
        broken_links,
        score: 0.0,
    }
}

pub async fn analyze_repo(
    client: &Octocrab,
    owner: &str,
    repo: &str,
) -> anyhow::Result<ReadmeReport> {
    let repo_info = client.repos(owner, repo).get().await?;

    let readme_result = client
        .repos(owner, repo)
        .get_readme()
        .send()
        .await;
    let readme_body = match readme_result {
        Ok(r) => r.decoded_content().unwrap_or_default(),
        Err(_) => String::new(),
    };

    let mut report = if readme_body.is_empty() {
        ReadmeReport {
            has_readme: false,
            has_contributing: false,
            has_code_of_conduct: false,
            has_license: repo_info.license.is_some(),
            has_issue_template: false,
            has_pr_template: false,
            has_build_instructions: false,
            broken_links: vec![],
            score: 0.0,
        }
    } else {
        let mut r = check_readme_content(&readme_body);
        r.has_license = repo_info.license.is_some();
        r
    };

    let files_to_check = &[
        ("CONTRIBUTING.md", "has_contributing"),
        ("CODE_OF_CONDUCT.md", "has_code_of_conduct"),
        (".github/ISSUE_TEMPLATE.md", "has_issue_template"),
        (".github/ISSUE_TEMPLATE/", "has_issue_template"),
        (".github/pull_request_template.md", "has_pr_template"),
    ];

    for (path, field) in files_to_check {
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/HEAD/{}",
            owner, repo, path
        );
        if let Ok(resp) = reqwest::get(&url).await {
            if resp.status().is_success() {
                match *field {
                    "has_contributing" => report.has_contributing = true,
                    "has_code_of_conduct" => report.has_code_of_conduct = true,
                    "has_issue_template" => report.has_issue_template = true,
                    "has_pr_template" => report.has_pr_template = true,
                    _ => {}
                }
            }
        }
    }

    report.score = score_readme(&report);
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_readme_perfect() {
        let report = ReadmeReport {
            has_readme: true,
            has_contributing: true,
            has_code_of_conduct: true,
            has_license: true,
            has_issue_template: true,
            has_pr_template: true,
            has_build_instructions: true,
            broken_links: vec![],
            score: 0.0,
        };
        let score = score_readme(&report);
        assert!((score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_score_readme_nothing() {
        let report = ReadmeReport {
            has_readme: false,
            has_contributing: false,
            has_code_of_conduct: false,
            has_license: false,
            has_issue_template: false,
            has_pr_template: false,
            has_build_instructions: false,
            broken_links: vec![],
            score: 0.0,
        };
        let score = score_readme(&report);
        assert!((score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_score_readme_partial() {
        let report = ReadmeReport {
            has_readme: true,
            has_contributing: true,
            has_code_of_conduct: false,
            has_license: true,
            has_issue_template: false,
            has_pr_template: false,
            has_build_instructions: false,
            broken_links: vec![],
            score: 0.0,
        };
        let score = score_readme(&report);
        assert!((score - 0.65).abs() < f64::EPSILON);
    }

    #[test]
    fn test_check_readme_content_with_build() {
        let readme = "# My Project\n\n## Getting Started\n\nRun `cargo build` to compile.";
        let report = check_readme_content(readme);
        assert!(report.has_build_instructions);
        assert!(report.has_readme);
    }

    #[test]
    fn test_check_readme_content_without_build() {
        let readme = "# My Project\n\nThis is a cool project.";
        let report = check_readme_content(readme);
        assert!(!report.has_build_instructions);
    }

    #[test]
    fn test_check_relative_links() {
        let readme = "# Project\n[docs](docs/readme.md)\n[ext](https://example.com)";
        let report = check_readme_content(readme);
        assert_eq!(report.broken_links.len(), 1);
    }

    #[test]
    fn test_empty_readme() {
        let report = check_readme_content("");
        assert!(!report.has_build_instructions);
    }
}
