use chrono::{DateTime, Utc};
use octocrab::Octocrab;
use serde::Serialize;

const GOOD_FIRST_LABELS: &[&str] = &[
    "good first issue",
    "good-first-issue",
    "beginner",
    "easy",
    "starter",
    "help wanted",
    "help-wanted",
    "E-easy",
    "low-hanging-fruit",
];

#[derive(Debug, Clone, Serialize)]
pub struct ScoredIssue {
    pub number: u64,
    pub title: String,
    pub url: String,
    pub labels: Vec<String>,
    pub assignee: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub body_preview: Option<String>,
    pub score: f64,
    pub matched_labels: Vec<String>,
}

pub fn score_issue(labels: &[String], has_body: bool, is_assigned: bool) -> (f64, Vec<String>) {
    let lower_labels: Vec<String> = labels.iter().map(|l| l.to_lowercase()).collect();
    let matched: Vec<String> = lower_labels
        .iter()
        .filter(|l| {
            GOOD_FIRST_LABELS
                .iter()
                .any(|g| l.contains(g) || g.contains(l.as_str()))
        })
        .cloned()
        .collect();

    let label_score = if matched.is_empty() { 0.0 } else { 0.5 };
    let body_score = if has_body { 0.3 } else { 0.0 };
    let unassigned_score = if is_assigned { 0.0 } else { 0.2 };

    (label_score + body_score + unassigned_score, matched)
}

pub async fn fetch_and_score(
    client: &Octocrab,
    owner: &str,
    repo: &str,
    limit: usize,
) -> anyhow::Result<Vec<ScoredIssue>> {
    let page = client
        .issues(owner, repo)
        .list()
        .state(octocrab::params::State::Open)
        .per_page(100)
        .send()
        .await?;

    let mut scored: Vec<ScoredIssue> = Vec::new();

    for issue in page.items {
        if issue.pull_request.is_some() {
            continue;
        }

        let labels: Vec<String> = issue.labels.iter().map(|l| l.name.clone()).collect();
        let has_body = issue.body.as_ref().is_some_and(|b| b.len() > 50);
        let is_assigned = !issue.assignees.is_empty();
        let (score, matched) = score_issue(&labels, has_body, is_assigned);

        let assignee = issue.assignees.first().map(|u| u.login.clone());

        let body_preview = issue.body.as_ref().map(|b| {
            let clean: String = b.chars().take(200).collect();
            clean.replace('\n', " ")
        });

        scored.push(ScoredIssue {
            number: issue.number,
            title: issue.title,
            url: issue.html_url.to_string(),
            labels,
            assignee,
            created_at: issue.created_at,
            updated_at: issue.updated_at,
            body_preview,
            score,
            matched_labels: matched,
        });
    }

    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(b.updated_at.cmp(&a.updated_at))
    });
    scored.truncate(limit);

    Ok(scored)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_good_first_issue() {
        let labels = vec!["good first issue".to_string(), "bug".to_string()];
        let (score, matched) = score_issue(&labels, true, false);
        assert!((score - 1.0).abs() < f64::EPSILON);
        assert_eq!(matched.len(), 1);
    }

    #[test]
    fn test_score_no_matching_labels() {
        let labels = vec!["bug".to_string(), "P-high".to_string()];
        let (score, matched) = score_issue(&labels, true, false);
        assert!((score - 0.5).abs() < f64::EPSILON); // body + unassigned only
        assert!(matched.is_empty());
    }

    #[test]
    fn test_score_assigned_no_body() {
        let labels = vec!["help wanted".to_string()];
        let (score, _) = score_issue(&labels, false, true);
        assert!((score - 0.5).abs() < f64::EPSILON); // label only
    }

    #[test]
    fn test_score_nothing() {
        let labels: Vec<String> = vec![];
        let (score, _) = score_issue(&labels, false, true);
        assert!((score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_score_case_insensitive() {
        let labels = vec!["Good First Issue".to_string()];
        let (score, matched) = score_issue(&labels, false, false);
        assert!((score - 0.7).abs() < f64::EPSILON);
        assert_eq!(matched.len(), 1);
    }

    #[test]
    fn test_score_multiple_matches() {
        let labels = vec!["good first issue".to_string(), "help wanted".to_string()];
        let (score, matched) = score_issue(&labels, true, false);
        assert!((score - 1.0).abs() < f64::EPSILON);
        assert_eq!(matched.len(), 2);
    }
}
