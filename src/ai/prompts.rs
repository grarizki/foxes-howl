use crate::config::UserProfile;

/// Build system + user prompt for `ai analyze`
pub fn build_analyze_prompt(
    repo: &str,
    issues_json: &str,
    stale_count: usize,
    readme_score: f64,
    quality_score: f64,
) -> (String, String) {
    let system = "You are an expert open source contribution advisor. \
Given repository analysis data, produce a structured contribution landscape summary. \
Return ONLY valid JSON with this exact structure:\n\
{\n\
  \"repo\": \"owner/repo\",\n\
  \"summary\": \"1-2 sentence overview of contribution friendliness\",\n\
  \"top_opportunities\": [\n\
    {\"issue_number\": N, \"title\": \"...\", \"why\": \"why this is good for contributors\", \"estimated_hours\": N}\n\
  ],\n\
  \"gaps\": [\"list of community/quality gaps found\"],\n\
  \"recommended_actions\": [\"concrete next steps for a contributor\"]\n\
}\n\
Keep top_opportunities to max 5 items. Be specific and actionable.";

    let user = format!(
        "Repository: {}\n\nOpen issues (scored by contribution friendliness):\n{}\n\nStale items count: {}\nREADME health score: {:.0}%\nCode quality score: {:.0}%\n\nAnalyze this repo's contribution landscape.",
        repo, issues_json, stale_count, readme_score * 100.0, quality_score * 100.0
    );

    (system.to_string(), user)
}

/// Build system + user prompt for `ai recommend`
pub fn build_recommend_prompt(
    repo: &str,
    profile: &UserProfile,
    issues_json: &str,
    stale_json: &str,
) -> (String, String) {
    let system = "You are an expert open source contribution matchmaker. \
Given a developer profile and repository data, recommend specific issues ranked by fit. \
Return ONLY valid JSON with this exact structure:\n\
{\n\
  \"repo\": \"owner/repo\",\n\
  \"developer_fit\": \"brief assessment of how well this repo matches the developer\",\n\
  \"ranked_issues\": [\n\
    {\n\
      \"issue_number\": N,\n\
      \"title\": \"...\",\n\
      \"fit_score\": 0.N,\n\
      \"difficulty\": \"beginner|intermediate|advanced\",\n\
      \"estimated_hours\": N,\n\
      \"first_step\": \"concrete first action to take\"\n\
    }\n\
  ],\n\
  \"profile_match\": \"overall fit assessment with reasoning\"\n\
}\n\
fit_score: 0.0 (poor fit) to 1.0 (perfect fit). Keep ranked_issues to max 5.";

    let profile_json = serde_json::to_string(profile).unwrap_or_default();

    let user = format!(
        "Repository: {}\n\nDeveloper profile:\n{}\n\nAvailable issues:\n{}\n\nStale items (review/revival opportunities):\n{}\n\nRecommend the best issues for this developer.",
        repo, profile_json, issues_json, stale_json
    );

    (system.to_string(), user)
}

/// Build system + user prompt for `ai difficulty`
pub fn build_difficulty_prompt(repo: &str, issues_json: &str) -> (String, String) {
    let system = "You are a senior software engineer. \
Rate each issue's difficulty level and explain what skills are needed. \
Return ONLY valid JSON with this exact structure:\n\
{\n\
  \"repo\": \"owner/repo\",\n\
  \"ratings\": [\n\
    {\n\
      \"issue_number\": N,\n\
      \"title\": \"...\",\n\
      \"difficulty\": \"beginner|intermediate|advanced\",\n\
      \"reasoning\": \"why this difficulty level\",\n\
      \"skills_needed\": [\"list\", \"of\", \"skills\"]\n\
    }\n\
  ]\n\
}\n\
Difficulty guide:\n\
- beginner: docs, typos, small config changes, good first issues\n\
- intermediate: bug fixes, small features, test additions\n\
- advanced: architecture changes, performance, security, complex features";

    let user = format!(
        "Repository: {}\n\nIssues to rate:\n{}\n\nRate the difficulty of each issue.",
        repo, issues_json
    );

    (system.to_string(), user)
}

/// Simple string template renderer — replaces {key} placeholders
pub fn render(template: &str, vars: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{}}}", key), value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_analyze_prompt() {
        let (system, user) = build_analyze_prompt("test/repo", "[]", 5, 0.8, 0.9);
        assert!(system.contains("valid JSON"));
        assert!(system.contains("top_opportunities"));
        assert!(user.contains("test/repo"));
        assert!(user.contains("Stale items count: 5"));
        assert!(user.contains("80%"));
    }

    #[test]
    fn test_build_recommend_prompt() {
        let profile = UserProfile {
            name: Some("Alice".to_string()),
            skills: vec!["rust".to_string()],
            experience: "intermediate".to_string(),
            hours_per_week: 4,
            interests: vec!["cli".to_string()],
        };
        let (system, user) = build_recommend_prompt("test/repo", &profile, "[]", "[]");
        assert!(system.contains("fit_score"));
        assert!(user.contains("Alice"));
        assert!(user.contains("rust"));
    }

    #[test]
    fn test_build_difficulty_prompt() {
        let (system, user) = build_difficulty_prompt("test/repo", "[]");
        assert!(system.contains("beginner|intermediate|advanced"));
        assert!(system.contains("skills_needed"));
        assert!(user.contains("test/repo"));
    }

    #[test]
    fn test_render() {
        let template = "Hello {name}, repo is {repo}";
        let result = render(template, &[("name", "Alice"), ("repo", "test/repo")]);
        assert_eq!(result, "Hello Alice, repo is test/repo");
    }

    #[test]
    fn test_render_no_vars() {
        let result = render("Hello world", &[]);
        assert_eq!(result, "Hello world");
    }
}
