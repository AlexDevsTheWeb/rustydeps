//! Minimal async client for OpenAI-compatible chat-completions APIs.
//!
//! OpenAI, Gemini (through its OpenAI-compatible endpoint), and a local Ollama
//! server all accept the same request and response shape, so one client covers
//! every provider. Requests are deliberately non-streaming: the report needs a
//! finished Markdown summary, not a token stream.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config::AiSettings;

/// Per-request timeout. One slow library must not stall the whole scan.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Failures raised while talking to a provider.
#[derive(Debug, Error)]
pub enum AiClientError {
    /// The HTTP client could not be constructed.
    #[error("could not build the AI HTTP client: {0}")]
    Build(#[source] reqwest::Error),
    /// The request failed before a usable response was received.
    #[error("request to the AI provider failed: {0}")]
    Transport(#[source] reqwest::Error),
    /// The provider answered with a non-success status.
    #[error("AI provider returned HTTP {status}: {body}")]
    Status {
        /// HTTP status code.
        status: u16,
        /// Short excerpt of the response body.
        body: String,
    },
    /// The response was successful but not a usable completion.
    #[error("AI provider returned an unusable response: {0}")]
    Response(String),
}

/// An async OpenAI-compatible client.
#[derive(Clone)]
pub struct AiClient {
    http: reqwest::Client,
    settings: AiSettings,
}

impl AiClient {
    /// Build a client for the resolved settings.
    pub fn new(settings: AiSettings) -> Result<Self, AiClientError> {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .user_agent(concat!("rustydeps/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(AiClientError::Build)?;
        Ok(Self { http, settings })
    }

    /// The fully resolved chat-completions URL.
    pub fn endpoint(&self) -> String {
        chat_endpoint(&self.settings.endpoint)
    }

    /// Send a system/user prompt pair and return the completion text.
    pub async fn complete(&self, system: &str, user: &str) -> Result<String, AiClientError> {
        let request = ChatRequest {
            model: &self.settings.model,
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: system,
                },
                ChatMessage {
                    role: "user",
                    content: user,
                },
            ],
            max_tokens: self.settings.max_tokens,
            temperature: 0.2,
            stream: false,
        };

        let mut builder = self.http.post(self.endpoint()).json(&request);
        if let Some(api_key) = &self.settings.api_key {
            builder = builder.bearer_auth(api_key);
        }

        let response = builder.send().await.map_err(AiClientError::Transport)?;
        let status = response.status();
        let body = response.text().await.map_err(AiClientError::Transport)?;

        if !status.is_success() {
            return Err(AiClientError::Status {
                status: status.as_u16(),
                body: truncate(&body, 300),
            });
        }

        parse_chat_response(&body)
    }
}

/// Resolve a base URL to the chat-completions endpoint.
fn chat_endpoint(base: &str) -> String {
    let base = base.trim_end_matches('/');
    if base.ends_with("/chat/completions") {
        base.to_string()
    } else {
        format!("{base}/chat/completions")
    }
}

/// Parse a standard chat-completions response body.
fn parse_chat_response(body: &str) -> Result<String, AiClientError> {
    let response: ChatResponse = serde_json::from_str(body)
        .map_err(|error| AiClientError::Response(format!("invalid JSON: {error}")))?;

    let content = response
        .choices
        .first()
        .and_then(|choice| choice.message.content.as_deref())
        .map(str::trim)
        .filter(|content| !content.is_empty())
        .ok_or_else(|| AiClientError::Response("completion has no text".to_string()))?;

    Ok(content.to_string())
}

/// Keep an error excerpt short and character-safe.
fn truncate(text: &str, limit: usize) -> String {
    text.chars().take(limit).collect()
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    max_tokens: u32,
    temperature: f32,
    stream: bool,
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ChatResponse {
    #[serde(default)]
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageContent,
}

#[derive(Deserialize)]
struct ChatMessageContent {
    content: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_base_urls_and_full_endpoints() {
        assert_eq!(
            chat_endpoint("https://api.openai.com/v1"),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            chat_endpoint("http://localhost:11434/v1/"),
            "http://localhost:11434/v1/chat/completions"
        );
        assert_eq!(
            chat_endpoint("https://example.test/v1/chat/completions"),
            "https://example.test/v1/chat/completions"
        );
    }

    #[test]
    fn parses_the_first_choice() {
        let body = r####"{
            "choices": [
                { "message": { "role": "assistant", "content": "### 🚨 Breaking Changes\nUse `foo`." } }
            ]
        }"####;

        assert_eq!(
            parse_chat_response(body).expect("parse"),
            "### 🚨 Breaking Changes\nUse `foo`."
        );
    }

    #[test]
    fn rejects_empty_or_malformed_responses() {
        assert!(matches!(
            parse_chat_response(r#"{"choices": []}"#),
            Err(AiClientError::Response(_))
        ));
        assert!(matches!(
            parse_chat_response(r#"{"choices": [{"message": {"content": "   "}}]}"#),
            Err(AiClientError::Response(_))
        ));
        assert!(matches!(
            parse_chat_response("not json"),
            Err(AiClientError::Response(_))
        ));
    }

    #[test]
    fn truncation_is_character_safe() {
        assert_eq!(truncate("aébc", 3), "aéb");
    }
}
