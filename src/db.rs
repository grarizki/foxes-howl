use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::PathBuf;

use crate::github::issues::ScoredIssue;

pub struct Cache {
    conn: Connection,
}

impl Cache {
    pub fn open() -> anyhow::Result<Self> {
        let db_path = Self::db_path()?;
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&db_path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS cached_issues (
                repo TEXT NOT NULL,
                number INTEGER NOT NULL,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                labels TEXT NOT NULL,
                assignee TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                body_preview TEXT,
                score REAL NOT NULL,
                matched_labels TEXT NOT NULL,
                cached_at TEXT NOT NULL,
                PRIMARY KEY (repo, number)
            );
            CREATE INDEX IF NOT EXISTS idx_repo ON cached_issues(repo);
            CREATE INDEX IF NOT EXISTS idx_score ON cached_issues(score DESC);
            ",
        )?;
        Ok(Self { conn })
    }

    #[allow(dead_code)]
    pub fn open_at(path: &std::path::Path) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS cached_issues (
                repo TEXT NOT NULL,
                number INTEGER NOT NULL,
                title TEXT NOT NULL,
                url TEXT NOT NULL,
                labels TEXT NOT NULL,
                assignee TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                body_preview TEXT,
                score REAL NOT NULL,
                matched_labels TEXT NOT NULL,
                cached_at TEXT NOT NULL,
                PRIMARY KEY (repo, number)
            );
            CREATE INDEX IF NOT EXISTS idx_repo ON cached_issues(repo);
            CREATE INDEX IF NOT EXISTS idx_score ON cached_issues(score DESC);
            ",
        )?;
        Ok(Self { conn })
    }

    fn db_path() -> anyhow::Result<PathBuf> {
        let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        Ok(base.join("gh-opp").join("cache.db"))
    }

    pub fn store(&self, repo: &str, issues: &[ScoredIssue]) -> anyhow::Result<()> {
        let now = Utc::now().to_rfc3339();
        let mut stmt = self.conn.prepare(
            "INSERT OR REPLACE INTO cached_issues
             (repo, number, title, url, labels, assignee, created_at, updated_at, body_preview, score, matched_labels, cached_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        )?;
        for issue in issues {
            stmt.execute(params![
                repo,
                issue.number as i64,
                issue.title,
                issue.url,
                serde_json::to_string(&issue.labels)?,
                issue.assignee,
                issue.created_at.to_rfc3339(),
                issue.updated_at.to_rfc3339(),
                issue.body_preview,
                issue.score,
                serde_json::to_string(&issue.matched_labels)?,
                now,
            ])?;
        }
        Ok(())
    }

    pub fn load(&self, repo: &str, max_age_secs: i64) -> anyhow::Result<Vec<ScoredIssue>> {
        let cutoff = Utc::now() - chrono::Duration::seconds(max_age_secs);
        let cutoff_str = cutoff.to_rfc3339();
        let mut stmt = self.conn.prepare(
            "SELECT number, title, url, labels, assignee, created_at, updated_at, body_preview, score, matched_labels
             FROM cached_issues
             WHERE repo = ?1 AND cached_at > ?2
             ORDER BY score DESC, updated_at DESC",
        )?;
        let rows = stmt
            .query_map(params![repo, cutoff_str], |row| {
                let number: i64 = row.get(0)?;
                let title: String = row.get(1)?;
                let url: String = row.get(2)?;
                let labels_str: String = row.get(3)?;
                let assignee: Option<String> = row.get(4)?;
                let created_str: String = row.get(5)?;
                let updated_str: String = row.get(6)?;
                let body_preview: Option<String> = row.get(7)?;
                let score: f64 = row.get(8)?;
                let matched_str: String = row.get(9)?;
                Ok(ScoredIssue {
                    number: number as u64,
                    title,
                    url,
                    labels: serde_json::from_str(&labels_str).unwrap_or_default(),
                    assignee,
                    created_at: DateTime::parse_from_rfc3339(&created_str)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: DateTime::parse_from_rfc3339(&updated_str)
                        .map(|d| d.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    body_preview,
                    score,
                    matched_labels: serde_json::from_str(&matched_str).unwrap_or_default(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    #[allow(dead_code)]
    pub fn clear(&self, repo: &str) -> anyhow::Result<()> {
        self.conn
            .execute("DELETE FROM cached_issues WHERE repo = ?1", params![repo])?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn clear_all(&self) -> anyhow::Result<()> {
        self.conn.execute("DELETE FROM cached_issues", [])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_cache() -> Cache {
        let dir = std::env::temp_dir().join(format!("gh-opp-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("test.db");
        Cache::open_at(&path).unwrap()
    }

    fn sample_issue(num: u64, title: &str) -> ScoredIssue {
        ScoredIssue {
            number: num,
            title: title.to_string(),
            url: format!("https://example.com/{}", num),
            labels: vec!["good first issue".to_string()],
            assignee: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            body_preview: Some("test body".to_string()),
            score: 0.5,
            matched_labels: vec!["good first issue".to_string()],
        }
    }

    #[test]
    fn test_open_and_schema() {
        let cache = temp_cache();
        // table exists
        let count: i64 = cache
            .conn
            .query_row(
                "SELECT COUNT(*) FROM cached_issues WHERE repo = 'test'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_store_and_load() {
        let cache = temp_cache();
        let issues = vec![sample_issue(1, "First issue"), {
            let mut i = sample_issue(2, "Second issue");
            i.score = 0.3; // lower score so it sorts after
            i
        }];
        cache.store("owner/repo", &issues).unwrap();

        let loaded = cache.load("owner/repo", 3600).unwrap();
        assert_eq!(loaded.len(), 2);
        // Higher score first
        assert_eq!(loaded[0].title, "First issue");
    }

    #[test]
    fn test_load_empty_repo() {
        let cache = temp_cache();
        let loaded = cache.load("nonexistent/repo", 3600).unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_clear() {
        let cache = temp_cache();
        let issues = vec![sample_issue(1, "Test")];
        cache.store("owner/repo", &issues).unwrap();
        cache.clear("owner/repo").unwrap();
        let loaded = cache.load("owner/repo", 3600).unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_clear_all() {
        let cache = temp_cache();
        cache.store("a/b", &[sample_issue(1, "A")]).unwrap();
        cache.store("c/d", &[sample_issue(2, "B")]).unwrap();
        cache.clear_all().unwrap();
        assert!(cache.load("a/b", 3600).unwrap().is_empty());
        assert!(cache.load("c/d", 3600).unwrap().is_empty());
    }

    #[test]
    fn test_store_replaces_duplicates() {
        let cache = temp_cache();
        let issue1 = sample_issue(1, "Original");
        let mut issue2 = sample_issue(1, "Updated");
        issue2.score = 0.9;

        cache.store("r/r", &[issue1]).unwrap();
        cache.store("r/r", &[issue2]).unwrap();

        let loaded = cache.load("r/r", 3600).unwrap();
        assert_eq!(loaded.len(), 1);
        // The replace happens, we get one row
    }
}
