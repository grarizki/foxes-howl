use crate::analysis::scoring::RepoScore;
use crate::analysis::stale::StaleItem;
use crate::github::issues::ScoredIssue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    Issues,
    Repos,
    Detail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Filtering,
}

pub struct App {
    pub screen: Screen,
    pub input_mode: InputMode,
    pub input: String,
    pub filter: String,
    pub issues: Vec<ScoredIssue>,
    pub stale_items: Vec<StaleItem>,
    pub repo_scores: Vec<RepoScore>,
    pub selected: usize,
    pub scroll: u16,
    pub status: String,
    #[allow(dead_code)]
    pub loading: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::Dashboard,
            input_mode: InputMode::Normal,
            input: String::new(),
            filter: String::new(),
            issues: Vec::new(),
            stale_items: Vec::new(),
            repo_scores: Vec::new(),
            selected: 0,
            scroll: 0,
            status: "Ready".to_string(),
            loading: false,
        }
    }

    pub fn next_screen(&mut self) {
        self.screen = match self.screen {
            Screen::Dashboard => Screen::Issues,
            Screen::Issues => Screen::Repos,
            Screen::Repos => Screen::Detail,
            Screen::Detail => Screen::Dashboard,
        };
        self.selected = 0;
        self.scroll = 0;
    }

    pub fn prev_screen(&mut self) {
        self.screen = match self.screen {
            Screen::Dashboard => Screen::Detail,
            Screen::Issues => Screen::Dashboard,
            Screen::Repos => Screen::Issues,
            Screen::Detail => Screen::Repos,
        };
        self.selected = 0;
        self.scroll = 0;
    }

    pub fn next_item(&mut self) {
        let max = self.current_list_len();
        if max > 0 {
            self.selected = (self.selected + 1) % max;
        }
    }

    pub fn prev_item(&mut self) {
        let max = self.current_list_len();
        if max > 0 {
            self.selected = if self.selected == 0 {
                max - 1
            } else {
                self.selected - 1
            };
        }
    }

    fn current_list_len(&self) -> usize {
        match self.screen {
            Screen::Dashboard => 0,
            Screen::Issues => {
                if self.filter.is_empty() {
                    self.issues.len()
                } else {
                    self.filtered_issues().len()
                }
            }
            Screen::Repos => self.repo_scores.len(),
            Screen::Detail => 0,
        }
    }

    pub fn filtered_issues(&self) -> Vec<&ScoredIssue> {
        if self.filter.is_empty() {
            return self.issues.iter().collect();
        }
        let lower_filter = self.filter.to_lowercase();
        self.issues
            .iter()
            .filter(|i| {
                i.title.to_lowercase().contains(&lower_filter)
                    || i.labels
                        .iter()
                        .any(|l| l.to_lowercase().contains(&lower_filter))
            })
            .collect()
    }

    pub fn selected_issue(&self) -> Option<&ScoredIssue> {
        if self.screen == Screen::Issues || self.screen == Screen::Detail {
            let filtered = self.filtered_issues();
            filtered.get(self.selected).copied()
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn selected_repo(&self) -> Option<&RepoScore> {
        self.repo_scores.get(self.selected)
    }

    pub fn set_issues(&mut self, issues: Vec<ScoredIssue>) {
        self.issues = issues;
        self.selected = 0;
    }

    pub fn set_stale(&mut self, items: Vec<StaleItem>) {
        self.stale_items = items;
    }

    pub fn set_repo_scores(&mut self, scores: Vec<RepoScore>) {
        self.repo_scores = scores;
        self.selected = 0;
    }

    pub fn start_filter(&mut self) {
        self.input_mode = InputMode::Filtering;
        self.input = self.filter.clone();
    }

    pub fn apply_filter(&mut self) {
        self.filter = self.input.clone();
        self.input_mode = InputMode::Normal;
        self.selected = 0;
    }

    pub fn cancel_filter(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input.clear();
    }

    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.input.clear();
        self.input_mode = InputMode::Normal;
        self.selected = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn sample_issue(num: u64, title: &str) -> ScoredIssue {
        ScoredIssue {
            number: num,
            title: title.to_string(),
            url: format!("https://example.com/{}", num),
            labels: vec!["good first issue".to_string()],
            assignee: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            body_preview: Some("test".to_string()),
            score: 0.5,
            matched_labels: vec!["good first issue".to_string()],
        }
    }

    #[test]
    fn test_new_app() {
        let app = App::new();
        assert_eq!(app.screen, Screen::Dashboard);
        assert_eq!(app.selected, 0);
        assert!(app.issues.is_empty());
    }

    #[test]
    fn test_screen_navigation() {
        let mut app = App::new();
        assert_eq!(app.screen, Screen::Dashboard);

        app.next_screen();
        assert_eq!(app.screen, Screen::Issues);

        app.next_screen();
        assert_eq!(app.screen, Screen::Repos);

        app.next_screen();
        assert_eq!(app.screen, Screen::Detail);

        app.next_screen();
        assert_eq!(app.screen, Screen::Dashboard); // wraps

        app.prev_screen();
        assert_eq!(app.screen, Screen::Detail);
    }

    #[test]
    fn test_item_navigation() {
        let mut app = App::new();
        app.set_issues(vec![
            sample_issue(1, "First"),
            sample_issue(2, "Second"),
            sample_issue(3, "Third"),
        ]);
        app.screen = Screen::Issues;

        assert_eq!(app.selected, 0);
        app.next_item();
        assert_eq!(app.selected, 1);
        app.next_item();
        assert_eq!(app.selected, 2);
        app.next_item();
        assert_eq!(app.selected, 0); // wraps

        app.prev_item();
        assert_eq!(app.selected, 2); // wraps back
    }

    #[test]
    fn test_filter_issues() {
        let mut app = App::new();
        app.set_issues(vec![
            sample_issue(1, "Fix docs"),
            sample_issue(2, "Add tests"),
            sample_issue(3, "Fix typo"),
        ]);
        app.screen = Screen::Issues;

        app.filter = "fix".to_string();
        let filtered = app.filtered_issues();
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_by_label() {
        let mut app = App::new();
        let mut issue = sample_issue(1, "Test");
        issue.labels = vec!["bug".to_string()];
        app.set_issues(vec![issue, sample_issue(2, "Other")]);

        app.filter = "bug".to_string();
        let filtered = app.filtered_issues();
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_selected_issue() {
        let mut app = App::new();
        app.set_issues(vec![sample_issue(1, "Test"), sample_issue(2, "Other")]);
        app.screen = Screen::Issues;

        let issue = app.selected_issue().unwrap();
        assert_eq!(issue.number, 1);

        app.next_item();
        let issue = app.selected_issue().unwrap();
        assert_eq!(issue.number, 2);
    }

    #[test]
    fn test_input_mode_toggle() {
        let mut app = App::new();
        assert_eq!(app.input_mode, InputMode::Normal);

        app.start_filter();
        assert_eq!(app.input_mode, InputMode::Filtering);

        app.input = "test".to_string();
        app.apply_filter();
        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.filter, "test");
    }

    #[test]
    fn test_clear_filter() {
        let mut app = App::new();
        app.filter = "test".to_string();
        app.clear_filter();
        assert!(app.filter.is_empty());
        assert_eq!(app.selected, 0);
    }
}
