use crate::config::AiConfig;
use std::future::Future;
use std::pin::Pin;

pub trait LlmProvider: Send + Sync {
    fn complete(
        &self,
        system: &str,
        user: &str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send + '_>>;

    fn name(&self) -> &str;
    fn model(&self) -> &str;
}

pub fn build_provider(cfg: &AiConfig) -> anyhow::Result<Box<dyn LlmProvider>> {
    let api_key = std::env::var(&cfg.api_key_env).map_err(|_| {
        anyhow::anyhow!(
            "Set {} in your environment. Config: api_key_env = \"{}\"",
            cfg.api_key_env,
            cfg.api_key_env
        )
    })?;

    match cfg.provider.as_str() {
        "openai" => Ok(Box::new(super::openai::OpenAiProvider::new(
            api_key,
            cfg.model.clone(),
            cfg.max_tokens,
        ))),
        "anthropic" => Ok(Box::new(super::anthropic::AnthropicProvider::new(
            api_key,
            cfg.model.clone(),
            cfg.max_tokens,
        ))),
        _ => anyhow::bail!(
            "Unknown AI provider '{}'. Supported: openai, anthropic",
            cfg.provider
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_provider_unknown() {
        let cfg = AiConfig {
            provider: "unknown".to_string(),
            model: "test".to_string(),
            api_key_env: "TEST_KEY".to_string(),
            max_tokens: 100,
            profile: Default::default(),
        };
        std::env::set_var("TEST_KEY", "test-key");
        match build_provider(&cfg) {
            Ok(_) => panic!("Expected error for unknown provider"),
            Err(e) => assert!(e.to_string().contains("Unknown AI provider")),
        }
        std::env::remove_var("TEST_KEY");
    }

    #[test]
    fn test_build_provider_missing_key() {
        let cfg = AiConfig {
            provider: "openai".to_string(),
            model: "test".to_string(),
            api_key_env: "NONEXISTENT_KEY_VAR".to_string(),
            max_tokens: 100,
            profile: Default::default(),
        };
        match build_provider(&cfg) {
            Ok(_) => panic!("Expected error for missing key"),
            Err(e) => assert!(e.to_string().contains("Set NONEXISTENT_KEY_VAR")),
        }
    }

    #[test]
    fn test_build_provider_openai() {
        let cfg = AiConfig {
            provider: "openai".to_string(),
            model: "gpt-4o".to_string(),
            api_key_env: "TEST_OPENAI_KEY".to_string(),
            max_tokens: 100,
            profile: Default::default(),
        };
        std::env::set_var("TEST_OPENAI_KEY", "sk-test");
        let provider = build_provider(&cfg).unwrap();
        assert_eq!(provider.name(), "openai");
        assert_eq!(provider.model(), "gpt-4o");
        std::env::remove_var("TEST_OPENAI_KEY");
    }

    #[test]
    fn test_build_provider_anthropic() {
        let cfg = AiConfig {
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            api_key_env: "TEST_ANTHROPIC_KEY".to_string(),
            max_tokens: 100,
            profile: Default::default(),
        };
        std::env::set_var("TEST_ANTHROPIC_KEY", "sk-ant-test");
        let provider = build_provider(&cfg).unwrap();
        assert_eq!(provider.name(), "anthropic");
        assert_eq!(provider.model(), "claude-sonnet-4-20250514");
        std::env::remove_var("TEST_ANTHROPIC_KEY");
    }
}
