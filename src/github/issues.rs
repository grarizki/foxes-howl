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
            continue; // skip PRs disguised as issues
        }

        let labels: Vec<String> = issue
            .labels
            .iter()
            .map(|l| l.name.to_lowercase())
            .collect();

        let matched: Vec<String> = labels
            .iter()
            .filter(|l| GOOD_FIRST_LABELS.iter().any(|g| l.contains(g) || g.contains(l.as_str())))
            .cloned()
            .collect();

        // score: label match (0.5) + has body (0.3) + unassigned (0.2)
        let label_score = if matched.is_empty() {
            0.0
        } else {
            0.5
        };
        let body_score = if issue.body.as_ref().map_or(false, |b| b.len() > 50) {
            0.3
        } else {
            0.0
        };
        let unassigned_score = if issue.assignees.is_empty() {
            0.2
        } else {
            0.0
        };
        let score = label_score + body_score + unassigned_score;

        let assignee = issue
            .assignees
            .first()
            .map(|u| u.login.clone());

        let body_preview = issue
            .body
            .as_ref()
            .map(|b| {
                let clean: String = b.chars().take(200).collect();
                clean.replace('\n', " ")
            });

        scored.push(ScoredIssue {
            number: issue.number,
            title: issue.title,
            url: issue.html_url.to_string(),
            labels: issue.labels.iter().map(|l| l.name.clone()).collect(),
            assignee,
            created_at: issue.created_at,
            updated_at: issue.updated_at,
            body_preview,
            score,
            matched_labels: matched,
        });
    }

    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);

    // re-sort: high-score first, then by updated_at descending
    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(b.updated_at.cmp(&a.updated_at))
    });

    Ok(scored)
}
