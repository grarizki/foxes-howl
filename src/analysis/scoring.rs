use serde::Serialize;

use super::code_quality::CodeQualityReport;
use super::readme::ReadmeReport;
use crate::config::ScoringConfig;
use crate::github::issues::ScoredIssue;

#[derive(Debug, Clone, Serialize)]
pub struct RepoScore {
    pub repo: String,
    pub good_first_score: f64,
    pub stale_score: f64,
    pub readme_score: f64,
    pub code_quality_score: f64,
    pub composite_score: f64,
    pub opportunity_count: usize,
    pub stale_count: usize,
}

pub fn composite_score(
    config: &ScoringConfig,
    good_first_avg: f64,
    stale_avg: f64,
    readme: &ReadmeReport,
    code_quality: &CodeQualityReport,
) -> f64 {
    let readme_inv = 1.0 - readme.score; // higher = worse = more opportunity
    let cq_inv = 1.0 - code_quality.score;

    (good_first_avg * config.good_first_weight)
        + (stale_avg * config.stale_weight)
        + (readme_inv * config.readme_weight)
        + (cq_inv * config.code_quality_weight)
}

pub fn average_good_first_score(issues: &[ScoredIssue]) -> f64 {
    if issues.is_empty() {
        return 0.0;
    }
    let sum: f64 = issues.iter().map(|i| i.score).sum();
    sum / issues.len() as f64
}

pub fn build_repo_score(
    config: &ScoringConfig,
    repo: &str,
    issues: &[ScoredIssue],
    stale_count: usize,
    readme: &ReadmeReport,
    code_quality: &CodeQualityReport,
) -> RepoScore {
    let good_first_avg = average_good_first_score(issues);
    let stale_avg = if stale_count > 0 {
        (stale_count as f64 / 20.0).min(1.0)
    } else {
        0.0
    };
    let composite = composite_score(config, good_first_avg, stale_avg, readme, code_quality);

    RepoScore {
        repo: repo.to_string(),
        good_first_score: good_first_avg,
        stale_score: stale_avg,
        readme_score: readme.score,
        code_quality_score: code_quality.score,
        composite_score: composite,
        opportunity_count: issues.len(),
        stale_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_readme() -> ReadmeReport {
        ReadmeReport {
            has_readme: true,
            has_contributing: true,
            has_code_of_conduct: true,
            has_license: true,
            has_issue_template: true,
            has_pr_template: true,
            has_build_instructions: true,
            broken_links: vec![],
            score: 1.0,
        }
    }

    fn default_cq() -> CodeQualityReport {
        CodeQualityReport {
            todo_count: 0,
            fixme_count: 0,
            hack_count: 0,
            has_ci: true,
            has_lint_config: true,
            has_test_dir: true,
            score: 1.0,
        }
    }

    #[test]
    fn test_composite_perfect_repo() {
        let config = ScoringConfig::default();
        let readme = default_readme();
        let cq = default_cq();
        // no opportunities, no stale => composite ~ 0
        let score = composite_score(&config, 0.0, 0.0, &readme, &cq);
        assert!((score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_composite_bad_repo() {
        let config = ScoringConfig::default();
        let readme = ReadmeReport {
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
        let cq = CodeQualityReport {
            todo_count: 0,
            fixme_count: 0,
            hack_count: 0,
            has_ci: false,
            has_lint_config: false,
            has_test_dir: false,
            score: 0.0,
        };
        let score = composite_score(&config, 0.5, 0.5, &readme, &cq);
        // 0.5*0.3 + 0.5*0.2 + 1.0*0.2 + 1.0*0.3 = 0.15+0.1+0.2+0.3 = 0.75
        assert!((score - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_average_good_first_empty() {
        assert!((average_good_first_score(&[]) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_build_repo_score() {
        let config = ScoringConfig::default();
        let readme = default_readme();
        let cq = default_cq();
        let score = build_repo_score(&config, "test/repo", &[], 0, &readme, &cq);
        assert_eq!(score.repo, "test/repo");
        assert!((score.composite_score - 0.0).abs() < f64::EPSILON);
    }
}
