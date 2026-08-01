use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub scoring: ScoringConfig,
    #[serde(default)]
    #[allow(dead_code)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub ai: AiConfig,
    #[serde(default)]
    pub serve: ServeConfig,
    #[serde(default)]
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiConfig {
    #[serde(default = "default_ai_provider")]
    pub provider: String,
    #[serde(default = "default_ai_model")]
    pub model: String,
    #[serde(default = "default_api_key_env")]
    pub api_key_env: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default)]
    pub profile: UserProfile,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserProfile {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default = "default_experience")]
    pub experience: String,
    #[serde(default = "default_hours_per_week")]
    pub hours_per_week: u32,
    #[serde(default)]
    pub interests: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServeConfig {
    #[serde(default = "default_token_env")]
    pub token_env: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    #[serde(default)]
    pub deny_config_path: Option<String>,
    #[serde(default)]
    pub secret_patterns: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScoringConfig {
    #[serde(default = "default_stale_days")]
    pub stale_days: u32,
    #[serde(default = "default_good_first_weight")]
    pub good_first_weight: f64,
    #[serde(default = "default_stale_weight")]
    pub stale_weight: f64,
    #[serde(default = "default_readme_weight")]
    pub readme_weight: f64,
    #[serde(default = "default_code_quality_weight")]
    pub code_quality_weight: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct DisplayConfig {
    #[serde(default = "default_max_results")]
    pub max_results: usize,
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            stale_days: default_stale_days(),
            good_first_weight: default_good_first_weight(),
            stale_weight: default_stale_weight(),
            readme_weight: default_readme_weight(),
            code_quality_weight: default_code_quality_weight(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            max_results: default_max_results(),
        }
    }
}

fn default_stale_days() -> u32 {
    30
}
fn default_good_first_weight() -> f64 {
    0.3
}
fn default_stale_weight() -> f64 {
    0.2
}
fn default_readme_weight() -> f64 {
    0.2
}
fn default_code_quality_weight() -> f64 {
    0.3
}
fn default_max_results() -> usize {
    25
}
fn default_ai_provider() -> String {
    "openai".to_string()
}
fn default_ai_model() -> String {
    "gpt-4o".to_string()
}
fn default_api_key_env() -> String {
    "OPENAI_API_KEY".to_string()
}
fn default_max_tokens() -> u32 {
    2048
}
fn default_experience() -> String {
    "intermediate".to_string()
}
fn default_hours_per_week() -> u32 {
    4
}
fn default_token_env() -> String {
    "GH_OPP_TOKEN".to_string()
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: default_ai_provider(),
            model: default_ai_model(),
            api_key_env: default_api_key_env(),
            max_tokens: default_max_tokens(),
            profile: UserProfile::default(),
        }
    }
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            name: None,
            skills: Vec::new(),
            experience: default_experience(),
            hours_per_week: default_hours_per_week(),
            interests: Vec::new(),
        }
    }
}

impl Default for ServeConfig {
    fn default() -> Self {
        Self {
            token_env: default_token_env(),
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            deny_config_path: None,
            secret_patterns: Vec::new(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn config_path() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("gh-opportunities").join("config.toml")
    }

    pub fn default_toml() -> &'static str {
        r#"# gh-opportunities configuration

[scoring]
stale_days = 30
good_first_weight = 0.3
stale_weight = 0.2
readme_weight = 0.2
code_quality_weight = 0.3

[display]
max_results = 25

[ai]
provider = "openai"
model = "gpt-4o"
api_key_env = "OPENAI_API_KEY"
max_tokens = 2048

[ai.profile]
skills = []
experience = "intermediate"
hours_per_week = 4
interests = []

[serve]
token_env = "GH_OPP_TOKEN"

[security]
deny_config_path = ""
secret_patterns = []
"#
    }

    pub fn init() -> anyhow::Result<PathBuf> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if !path.exists() {
            std::fs::write(&path, Self::default_toml())?;
        }
        Ok(path)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scoring: ScoringConfig::default(),
            display: DisplayConfig::default(),
            ai: AiConfig::default(),
            serve: ServeConfig::default(),
            security: SecurityConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = Config::default();
        assert_eq!(cfg.scoring.stale_days, 30);
        assert_eq!(cfg.display.max_results, 25);
        assert!((cfg.scoring.good_first_weight - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_toml() {
        let toml_str = r#"
[scoring]
stale_days = 14
good_first_weight = 0.5

[display]
max_results = 50
"#;
        let cfg: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.scoring.stale_days, 14);
        assert!((cfg.scoring.good_first_weight - 0.5).abs() < f64::EPSILON);
        assert_eq!(cfg.display.max_results, 50);
        // defaults for unmentioned fields
        assert!((cfg.scoring.stale_weight - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_empty_toml() {
        let cfg: Config = toml::from_str("").unwrap();
        assert_eq!(cfg.scoring.stale_days, 30);
    }

    #[test]
    fn test_default_toml_is_valid() {
        let cfg: Config = toml::from_str(Config::default_toml()).unwrap();
        assert_eq!(cfg.scoring.stale_days, 30);
    }

    #[test]
    fn test_ai_config_defaults() {
        let cfg = Config::default();
        assert_eq!(cfg.ai.provider, "openai");
        assert_eq!(cfg.ai.model, "gpt-4o");
        assert_eq!(cfg.ai.api_key_env, "OPENAI_API_KEY");
        assert_eq!(cfg.ai.max_tokens, 2048);
    }

    #[test]
    fn test_user_profile_defaults() {
        let cfg = Config::default();
        assert!(cfg.ai.profile.name.is_none());
        assert!(cfg.ai.profile.skills.is_empty());
        assert_eq!(cfg.ai.profile.experience, "intermediate");
        assert_eq!(cfg.ai.profile.hours_per_week, 4);
        assert!(cfg.ai.profile.interests.is_empty());
    }

    #[test]
    fn test_serve_config_defaults() {
        let cfg = Config::default();
        assert_eq!(cfg.serve.token_env, "GH_OPP_TOKEN");
    }

    #[test]
    fn test_security_config_defaults() {
        let cfg = Config::default();
        assert!(cfg.security.deny_config_path.is_none());
        assert!(cfg.security.secret_patterns.is_empty());
    }

    #[test]
    fn test_parse_ai_config() {
        let toml_str = r#"
[ai]
provider = "anthropic"
model = "claude-sonnet-4-20250514"
api_key_env = "ANTHROPIC_API_KEY"
max_tokens = 4096

[ai.profile]
name = "Alice"
skills = ["rust", "web"]
experience = "senior"
hours_per_week = 10
interests = ["tooling", "cli"]
"#;
        let cfg: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.ai.provider, "anthropic");
        assert_eq!(cfg.ai.model, "claude-sonnet-4-20250514");
        assert_eq!(cfg.ai.api_key_env, "ANTHROPIC_API_KEY");
        assert_eq!(cfg.ai.max_tokens, 4096);
        assert_eq!(cfg.ai.profile.name.as_deref(), Some("Alice"));
        assert_eq!(cfg.ai.profile.skills, vec!["rust", "web"]);
        assert_eq!(cfg.ai.profile.experience, "senior");
        assert_eq!(cfg.ai.profile.hours_per_week, 10);
        assert_eq!(cfg.ai.profile.interests, vec!["tooling", "cli"]);
    }

    #[test]
    fn test_parse_serve_config() {
        let toml_str = r#"
[serve]
token_env = "MY_CUSTOM_TOKEN"
"#;
        let cfg: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.serve.token_env, "MY_CUSTOM_TOKEN");
    }

    #[test]
    fn test_parse_security_config() {
        let toml_str = r#"
[security]
deny_config_path = "/custom/deny.toml"
secret_patterns = ["MY_SECRET_[0-9]+"]
"#;
        let cfg: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(
            cfg.security.deny_config_path.as_deref(),
            Some("/custom/deny.toml")
        );
        assert_eq!(cfg.security.secret_patterns, vec!["MY_SECRET_[0-9]+"]);
    }
}
