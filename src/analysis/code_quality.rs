use octocrab::Octocrab;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CodeQualityReport {
    pub todo_count: u32,
    pub fixme_count: u32,
    pub hack_count: u32,
    pub has_ci: bool,
    pub has_lint_config: bool,
    pub has_test_dir: bool,
    pub score: f64,
}

pub fn score_code_quality(report: &CodeQualityReport) -> f64 {
    let mut score = 1.0;

    // penalize for TODO/FIXME/HACK (more = lower score)
    let issue_count = report.todo_count + report.fixme_count + report.hack_count;
    let penalty = (issue_count as f64 * 0.02).min(0.4);
    score -= penalty;

    // bonus for CI, lint, tests
    if !report.has_ci {
        score -= 0.2;
    }
    if !report.has_lint_config {
        score -= 0.1;
    }
    if !report.has_test_dir {
        score -= 0.2;
    }

    score.max(0.0)
}

pub async fn analyze_repo(
    client: &Octocrab,
    owner: &str,
    repo: &str,
) -> anyhow::Result<CodeQualityReport> {
    // search for TODO/FIXME/HACK in code
    let todo_query = format!("TODO repo:{}/{}", owner, repo);
    let fixme_query = format!("FIXME repo:{}/{}", owner, repo);
    let hack_query = format!("HACK repo:{}/{}", owner, repo);

    let todo_count = search_code_count(client, &todo_query).await.unwrap_or(0);
    let fixme_count = search_code_count(client, &fixme_query).await.unwrap_or(0);
    let hack_count = search_code_count(client, &hack_query).await.unwrap_or(0);

    // check for CI config
    let has_ci = check_file_exists(owner, repo, ".github/workflows/").await
        || check_file_exists(owner, repo, ".circleci/config.yml").await
        || check_file_exists(owner, repo, ".travis.yml").await
        || check_file_exists(owner, repo, "Jenkinsfile").await;

    // check for lint config
    let has_lint_config = check_file_exists(owner, repo, ".eslintrc").await
        || check_file_exists(owner, repo, ".eslintrc.json").await
        || check_file_exists(owner, repo, "clippy.toml").await
        || check_file_exists(owner, repo, ".clippy.toml").await
        || check_file_exists(owner, repo, "rustfmt.toml").await
        || check_file_exists(owner, repo, ".rustfmt.toml").await
        || check_file_exists(owner, repo, ".flake8").await
        || check_file_exists(owner, repo, "pyproject.toml").await;

    // check for test directory
    let has_test_dir = check_file_exists(owner, repo, "tests/").await
        || check_file_exists(owner, repo, "test/").await
        || check_file_exists(owner, repo, "spec/").await
        || check_file_exists(owner, repo, "__tests__/").await;

    let mut report = CodeQualityReport {
        todo_count,
        fixme_count,
        hack_count,
        has_ci,
        has_lint_config,
        has_test_dir,
        score: 0.0,
    };
    report.score = score_code_quality(&report);

    Ok(report)
}

async fn search_code_count(client: &Octocrab, query: &str) -> anyhow::Result<u32> {
    let result = client.search().code(query).send().await?;
    Ok(result.total_count.unwrap_or(0) as u32)
}

async fn check_file_exists(owner: &str, repo: &str, path: &str) -> bool {
    let url = format!(
        "https://raw.githubusercontent.com/{}/{}/HEAD/{}",
        owner, repo, path
    );
    reqwest::get(&url)
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_perfect() {
        let report = CodeQualityReport {
            todo_count: 0,
            fixme_count: 0,
            hack_count: 0,
            has_ci: true,
            has_lint_config: true,
            has_test_dir: true,
            score: 0.0,
        };
        let score = score_code_quality(&report);
        assert!((score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_score_with_todos() {
        let report = CodeQualityReport {
            todo_count: 5,
            fixme_count: 2,
            hack_count: 1,
            has_ci: true,
            has_lint_config: true,
            has_test_dir: true,
            score: 0.0,
        };
        let score = score_code_quality(&report);
        assert!(score < 1.0);
        assert!(score > 0.5);
    }

    #[test]
    fn test_score_no_ci_no_tests() {
        let report = CodeQualityReport {
            todo_count: 0,
            fixme_count: 0,
            hack_count: 0,
            has_ci: false,
            has_lint_config: false,
            has_test_dir: false,
            score: 0.0,
        };
        let score = score_code_quality(&report);
        assert!((score - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_score_floor_at_zero() {
        let report = CodeQualityReport {
            todo_count: 100,
            fixme_count: 100,
            hack_count: 100,
            has_ci: false,
            has_lint_config: false,
            has_test_dir: false,
            score: 0.0,
        };
        let score = score_code_quality(&report);
        // penalty capped at 0.4, CI -0.2, lint -0.1, tests -0.2 = 0.1
        assert!((score - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn test_penalty_capped() {
        let report = CodeQualityReport {
            todo_count: 50,
            fixme_count: 0,
            hack_count: 0,
            has_ci: true,
            has_lint_config: true,
            has_test_dir: true,
            score: 0.0,
        };
        let score = score_code_quality(&report);
        assert!((score - 0.6).abs() < f64::EPSILON);
    }
}
