use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub scoring: ScoringConfig,
    #[serde(default)]
    #[allow(dead_code)]
    pub display: DisplayConfig,
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
}
