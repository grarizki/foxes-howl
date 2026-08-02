use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

use super::provider::LlmProvider;

pub struct AnthropicProvider {
    api_key: String,
    model: String,
    max_tokens: u32,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct MessageRequest {
    model: String,
    system: String,
    messages: Vec<Message>,
    max_tokens: u32,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct MessageResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: Option<String>,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: Option<ErrorDetail>,
}

#[derive(Deserialize)]
struct ErrorDetail {
    message: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String, max_tokens: u32) -> Self {
        Self {
            api_key,
            model,
            max_tokens,
            client: reqwest::Client::new(),
        }
    }
}

impl LlmProvider for AnthropicProvider {
    fn complete(
        &self,
        system: &str,
        user: &str,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send + '_>> {
        let system = system.to_string();
        let user = user.to_string();
        let api_key = self.api_key.clone();
        let model = self.model.clone();
        let max_tokens = self.max_tokens;
        let client = self.client.clone();

        Box::pin(async move {
            let request = MessageRequest {
                model,
                system,
                messages: vec![Message {
                    role: "user".to_string(),
                    content: user,
                }],
                max_tokens,
            };

            let response = client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await?;

            let status = response.status();
            if status == 429 {
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                let response = client
                    .post("https://api.anthropic.com/v1/messages")
                    .header("x-api-key", &api_key)
                    .header("anthropic-version", "2023-06-01")
                    .header("Content-Type", "application/json")
                    .json(&request)
                    .send()
                    .await?;

                if !response.status().is_success() {
                    let error: ErrorResponse = response
                        .json()
                        .await
                        .unwrap_or(ErrorResponse { error: None });
                    let msg = error
                        .error
                        .map(|e| e.message)
                        .unwrap_or_else(|| "Unknown error".to_string());
                    anyhow::bail!("Anthropic API error after retry: {}", msg);
                }

                let msg_resp: MessageResponse = response.json().await?;
                return msg_resp
                    .content
                    .first()
                    .and_then(|b| b.text.clone())
                    .ok_or_else(|| anyhow::anyhow!("Empty response from Anthropic"));
            }

            if !status.is_success() {
                let error: ErrorResponse = response
                    .json()
                    .await
                    .unwrap_or(ErrorResponse { error: None });
                let msg = error
                    .error
                    .map(|e| e.message)
                    .unwrap_or_else(|| format!("HTTP {}", status));
                anyhow::bail!("Anthropic API error: {}", msg);
            }

            let msg_resp: MessageResponse = response.json().await?;
            msg_resp
                .content
                .first()
                .and_then(|b| b.text.clone())
                .ok_or_else(|| anyhow::anyhow!("Empty response from Anthropic"))
        })
    }

    fn name(&self) -> &str {
        "anthropic"
    }

    fn model(&self) -> &str {
        &self.model
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_metadata() {
        let provider = AnthropicProvider::new(
            "sk-ant-test".to_string(),
            "claude-sonnet-4-20250514".to_string(),
            1024,
        );
        assert_eq!(provider.name(), "anthropic");
        assert_eq!(provider.model(), "claude-sonnet-4-20250514");
    }

    #[test]
    fn test_request_serialization() {
        let request = MessageRequest {
            model: "claude-sonnet-4-20250514".to_string(),
            system: "You are helpful".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            max_tokens: 100,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"model\":\"claude-sonnet-4-20250514\""));
        assert!(json.contains("\"system\":\"You are helpful\""));
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"max_tokens\":100"));
    }

    #[test]
    fn test_response_deserialization() {
        let json = r#"{"content":[{"text":"Hello!"}]}"#;
        let resp: MessageResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.content[0].text.as_deref(), Some("Hello!"));
    }

    #[test]
    fn test_error_response_deserialization() {
        let json = r#"{"error":{"message":"Invalid API key"}}"#;
        let resp: ErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.error.unwrap().message, "Invalid API key");
    }
}
