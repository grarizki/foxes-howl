use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

use super::provider::LlmProvider;

pub struct OpenAiProvider {
    api_key: String,
    model: String,
    max_tokens: u32,
    client: reqwest::Client,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageResponse,
}

#[derive(Deserialize)]
struct MessageResponse {
    content: Option<String>,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: Option<ErrorDetail>,
}

#[derive(Deserialize)]
struct ErrorDetail {
    message: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String, model: String, max_tokens: u32) -> Self {
        Self {
            api_key,
            model,
            max_tokens,
            client: reqwest::Client::new(),
        }
    }
}

impl LlmProvider for OpenAiProvider {
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
            let request = ChatRequest {
                model,
                messages: vec![
                    Message {
                        role: "system".to_string(),
                        content: system,
                    },
                    Message {
                        role: "user".to_string(),
                        content: user,
                    },
                ],
                max_tokens,
            };

            let response = client
                .post("https://api.openai.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await?;

            let status = response.status();
            if status == 429 {
                // Rate limited — retry once after 2s
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                let response = client
                    .post("https://api.openai.com/v1/chat/completions")
                    .header("Authorization", format!("Bearer {}", api_key))
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
                    anyhow::bail!("OpenAI API error after retry: {}", msg);
                }

                let chat: ChatResponse = response.json().await?;
                return chat
                    .choices
                    .first()
                    .and_then(|c| c.message.content.clone())
                    .ok_or_else(|| anyhow::anyhow!("Empty response from OpenAI"));
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
                anyhow::bail!("OpenAI API error: {}", msg);
            }

            let chat: ChatResponse = response.json().await?;
            chat.choices
                .first()
                .and_then(|c| c.message.content.clone())
                .ok_or_else(|| anyhow::anyhow!("Empty response from OpenAI"))
        })
    }

    fn name(&self) -> &str {
        "openai"
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
        let provider = OpenAiProvider::new("sk-test".to_string(), "gpt-4o".to_string(), 1024);
        assert_eq!(provider.name(), "openai");
        assert_eq!(provider.model(), "gpt-4o");
    }

    #[test]
    fn test_request_serialization() {
        let request = ChatRequest {
            model: "gpt-4o".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are helpful".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                },
            ],
            max_tokens: 100,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"model\":\"gpt-4o\""));
        assert!(json.contains("\"role\":\"system\""));
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"max_tokens\":100"));
    }

    #[test]
    fn test_response_deserialization() {
        let json = r#"{"choices":[{"message":{"content":"Hello!"}}]}"#;
        let resp: ChatResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.choices[0].message.content.as_deref(), Some("Hello!"));
    }

    #[test]
    fn test_error_response_deserialization() {
        let json = r#"{"error":{"message":"Invalid API key"}}"#;
        let resp: ErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.error.unwrap().message, "Invalid API key");
    }
}
