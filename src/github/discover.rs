use octocrab::Octocrab;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DiscoveredRepo {
    pub full_name: String,
    pub url: String,
    pub description: Option<String>,
    pub stars: u64,
    pub language: Option<String>,
    pub good_first_issues: u64,
    pub score: f64,
}

pub async fn discover_repos(
    client: &Octocrab,
    lang: Option<&str>,
    _topic: Option<&str>, // ponytail: topic needs repo-metadata field we don't fetch; dropped
    min_stars: u64,
    limit: usize,
) -> anyhow::Result<Vec<DiscoveredRepo>> {
    // Search for issues with good-first-issue labels.
    // Note: GitHub issue search does not support `language:`/`stars:`/`topic:`
    // qualifiers (repo-search only). They are applied post-fetch in `applies`.
    let query = "label:\"good first issue\" is:issue is:open";

    let results: octocrab::Page<octocrab::models::issues::Issue> = client
        .search()
        .issues_and_pull_requests(&query)
        .sort("updated")
        .order("desc")
        .send()
        .await?;

    // Group issues by repo
    let mut repos: std::collections::HashMap<String, DiscoveredRepo> =
        std::collections::HashMap::new();

    for issue in results.items {
        let repo_url = issue.repository_url.to_string();
        let full_name = repo_url
            .rsplit('/')
            .take(2)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("/");

        let entry = repos
            .entry(full_name.clone())
            .or_insert_with(|| DiscoveredRepo {
                full_name: full_name.clone(),
                url: format!("https://github.com/{}", full_name),
                description: None,
                stars: 0,
                language: None,
                good_first_issues: 0,
                score: 0.0,
            });

        entry.good_first_issues += 1;
    }

    // Fetch repo metadata for each discovered repo
    let mut result: Vec<DiscoveredRepo> = Vec::new();
    for (full_name, mut repo) in repos {
        let parts: Vec<&str> = full_name.split('/').collect();
        if parts.len() != 2 {
            continue;
        }

        match client.repos(parts[0], parts[1]).get().await {
            Ok(repo_data) => {
                repo.stars = repo_data.stargazers_count.unwrap_or(0) as u64;
                repo.language = repo_data.language.as_ref().map(|l| l.to_string());
                repo.description = repo_data.description.clone();

                // Score: good_first_issues * (1 + log10(stars))
                let star_bonus = if repo.stars > 0 {
                    (repo.stars as f64).log10()
                } else {
                    0.0
                };
                repo.score = repo.good_first_issues as f64 * (1.0 + star_bonus);

                if !applies(lang, min_stars, &repo) {
                    continue;
                }
                result.push(repo);
            }
            Err(e) => {
                tracing::warn!("Failed to fetch repo {}: {}", full_name, e);
                // Still include it with partial data
                repo.score = repo.good_first_issues as f64;
                if !applies(lang, min_stars, &repo) {
                    continue;
                }
                result.push(repo);
            }
        }
    }

    // Sort by score descending
    result.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    result.truncate(limit);

    Ok(result)
}

/// Post-fetch filter. `lang` is exact-match (case-insensitive, same as GitHub).
/// octocrab's `language` carries surrounding quotes (e.g. `"TypeScript"`); trim.
fn applies(lang: Option<&str>, min_stars: u64, repo: &DiscoveredRepo) -> bool {
    if repo.stars < min_stars {
        return false;
    }
    if let Some(lang) = lang {
        let lang = lang.trim_matches('"');
        let matches = repo
            .language
            .as_deref()
            .map(|l| l.trim_matches('"').eq_ignore_ascii_case(lang))
            .unwrap_or(false);
        if !matches {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_calculation() {
        // 5 issues, 1000 stars => 5 * (1 + 3) = 20
        let issues = 5u64;
        let stars = 1000u64;
        let score = issues as f64 * (1.0 + (stars as f64).log10());
        assert!((score - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_score_no_stars() {
        let issues = 3u64;
        let stars = 0u64;
        let score = issues as f64
            * (1.0
                + if stars > 0 {
                    (stars as f64).log10()
                } else {
                    0.0
                });
        assert!((score - 3.0).abs() < 0.01);
    }

    fn repo(stars: u64, language: Option<&str>) -> DiscoveredRepo {
        DiscoveredRepo {
            full_name: "o/r".to_string(),
            url: "https://github.com/o/r".to_string(),
            description: None,
            stars,
            language: language.map(|s| s.to_string()),
            good_first_issues: 1,
            score: 0.0,
        }
    }

    #[test]
    fn test_applies_lang_exact() {
        let r = repo(5000, Some("TypeScript"));
        assert!(applies(Some("typescript"), 0, &r)); // exact, case-insensitive
        assert!(!applies(Some("rust"), 0, &r)); // mismatch
        assert!(!applies(Some("typescript"), 0, &repo(5000, None))); // no lang on repo
    }

    #[test]
    fn test_applies_lang_octocrab_quotes() {
        // octocrab's language carries surrounding quotes
        let r = repo(5000, Some("\"TypeScript\""));
        assert!(applies(Some("typescript"), 0, &r));
        assert!(applies(Some("\"typescript\""), 0, &r));
        assert!(!applies(Some("rust"), 0, &r));
    }

    #[test]
    fn test_applies_min_stars_bounds() {
        let r = repo(1000, None);
        assert!(applies(None, 1000, &r)); // == passes, not strict >
        assert!(!applies(None, 1001, &r)); // below
    }
}
