use chrono::Utc;
use octocrab::Octocrab;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct StaleItem {
    pub kind: String,
    pub number: u64,
    pub title: String,
    pub url: String,
    pub last_activity_days: i64,
    pub stale_severity: f64,
    pub has_assignee: bool,
    pub labels: Vec<String>,
}

pub fn severity(last_activity_days: i64, stale_threshold: u32) -> f64 {
    if last_activity_days <= stale_threshold as i64 {
        return 0.0;
    }
    let excess = (last_activity_days - stale_threshold as i64) as f64;
    (excess / 90.0).min(1.0)
}

pub async fn find_stale_issues(
    client: &Octocrab,
    owner: &str,
    repo: &str,
    stale_days: u32,
    limit: usize,
) -> anyhow::Result<Vec<StaleItem>> {
    let page = client
        .issues(owner, repo)
        .list()
        .state(octocrab::params::State::Open)
        .per_page(100)
        .send()
        .await?;

    let now = Utc::now();
    let mut items: Vec<StaleItem> = Vec::new();

    for issue in page.items {
        if issue.pull_request.is_some() {
            continue;
        }
        let days = (now - issue.updated_at).num_days();
        let sev = severity(days, stale_days);
        if sev > 0.0 {
            items.push(StaleItem {
                kind: "issue".to_string(),
                number: issue.number,
                title: issue.title,
                url: issue.html_url.to_string(),
                last_activity_days: days,
                stale_severity: sev,
                has_assignee: !issue.assignees.is_empty(),
                labels: issue.labels.iter().map(|l| l.name.clone()).collect(),
            });
        }
    }

    items.sort_by(|a, b| {
        b.stale_severity
            .partial_cmp(&a.stale_severity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    items.truncate(limit);
    Ok(items)
}

pub async fn find_stale_prs(
    client: &Octocrab,
    owner: &str,
    repo: &str,
    stale_days: u32,
    limit: usize,
) -> anyhow::Result<Vec<StaleItem>> {
    let page = client
        .pulls(owner, repo)
        .list()
        .state(octocrab::params::State::Open)
        .per_page(100)
        .send()
        .await?;

    let now = Utc::now();
    let mut items: Vec<StaleItem> = Vec::new();

    for pr in page.items {
        let updated = pr
            .updated_at
            .or(pr.created_at)
            .unwrap_or(chrono::Utc::now());
        let days = (now - updated).num_days();
        let sev = severity(days, stale_days);
        if sev > 0.0 {
            items.push(StaleItem {
                kind: "pr".to_string(),
                number: pr.number,
                title: pr.title.unwrap_or_default(),
                url: pr.html_url.map(|u| u.to_string()).unwrap_or_default(),
                last_activity_days: days,
                stale_severity: sev,
                has_assignee: pr.assignee.is_some(),
                labels: pr
                    .labels
                    .unwrap_or_default()
                    .iter()
                    .map(|l| l.name.clone())
                    .collect(),
            });
        }
    }

    items.sort_by(|a, b| {
        b.stale_severity
            .partial_cmp(&a.stale_severity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    items.truncate(limit);
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_below_threshold() {
        assert!((severity(10, 30) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_severity_at_threshold() {
        assert!((severity(30, 30) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_severity_above_threshold() {
        let sev = severity(60, 30);
        assert!(sev > 0.0);
        assert!(sev < 1.0);
    }

    #[test]
    fn test_severity_cap_at_one() {
        let sev = severity(500, 30);
        assert!((sev - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_severity_linear_decay() {
        let sev30 = severity(60, 30);
        let sev60 = severity(120, 30);
        assert!(sev60 > sev30);
    }
}
