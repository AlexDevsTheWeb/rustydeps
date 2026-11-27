//! Minimal async client for OpenAI-compatible chat-completions APIs.
//!
//! OpenAI, Gemini (through its OpenAI-compatible endpoint), OpenRouter, and a
//! local Ollama server all accept the same request and response shape, so one
//! client covers every provider — including a user-supplied OpenAI-compatible
//! proxy. Requests are deliberately non-streaming: the report needs a finished
//! Markdown summary, not a token stream.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::debug;

use crate::config::{AiProvider, AiSettings};

/// Per-request timeout. One slow library must not stall the whole scan, but a
/// reasoning model can take tens of seconds to produce a summary.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(45);

/// Per-request timeout for the Ollama daemon health probe.
pub const OLLAMA_PROBE_TIMEOUT: Duration = Duration::from_millis(1000);

/// Maximum number of response-body characters kept in an error message.
pub const ERROR_BODY_LIMIT: usize = 300;

/// Attribution sent to OpenRouter so usage can be identified in its dashboard.
pub const OPENROUTER_APP_URL: &str = "https://github.com/AlexDevsTheWeb/rustydeps";
/// Human-readable application name sent to OpenRouter.
pub const OPENROUTER_APP_TITLE: &str = "RustyDeps";

/// Failures raised while talking to a provider.
#[derive(Debug, Error)]
pub enum AiClientError {
    /// The HTTP client could not be constructed.
    #[error("could not build the AI HTTP client: {0}")]
    Build(#[source] reqwest::Error),
    /// The resolved settings cannot produce a valid request.
    #[error("invalid AI configuration: {0}")]
    Configuration(String),
    /// The request failed before a usable response was received.
    #[error("request to {url} failed: {source}")]
    Transport {
        /// Resolved chat-completions URL.
        url: String,
        /// Underlying transport error.
        #[source]
        source: reqwest::Error,
    },
    /// The provider answered with a non-success status.
    #[error("request to {url} returned HTTP {status}: {body}")]
    Status {
        /// Resolved chat-completions URL.
        url: String,
        /// HTTP status code.
        status: u16,
        /// Compact excerpt of the response body.
        body: String,
    },
    /// The response was successful but not a usable completion.
    #[error("AI provider returned an unusable response: {0}")]
    Response(String),
}

impl AiClientError {
    /// Whether retrying the same request for other packages cannot succeed.
    ///
    /// Authentication, quota, model-not-found, and connection failures are
    /// configuration problems: one clear warning is enough, and the remaining
    /// packages must not burn time or tokens. Status 429 is included because a
    /// quota-exhausted account stays exhausted for the rest of the scan.
    pub fn is_fatal(&self) -> bool {
        match self {
            Self::Status { status, body, .. } => {
                matches!(*status, 401 | 402 | 403 | 404 | 429)
                    || body.to_ascii_lowercase().contains("quota")
            }
            Self::Transport { source, .. } => source.is_connect(),
            Self::Build(_) | Self::Configuration(_) | Self::Response(_) => true,
        }
    }
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
        if settings.endpoint.trim().is_empty() {
            return Err(AiClientError::Configuration(format!(
                "the {} provider needs an endpoint; set ai.endpoint in the config or RUSTYDEPS_AI_ENDPOINT",
                settings.provider.label()
            )));
        }
        if settings.model.trim().is_empty() {
            return Err(AiClientError::Configuration(format!(
                "the {} provider needs a model; set ai.model in the config or RUSTYDEPS_AI_MODEL",
                settings.provider.label()
            )));
        }

        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .user_agent(concat!("rustydeps/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(AiClientError::Build)?;
        Ok(Self { http, settings })
    }

    /// The fully resolved chat-completions URL.
    pub fn endpoint(&self) -> String {
        chat_endpoint(self.settings.provider, &self.settings.endpoint)
    }

    /// Origin of the Ollama daemon, used in offline warnings.
    pub fn ollama_origin(&self) -> String {
        match reqwest::Url::parse(self.settings.endpoint.trim()) {
            Ok(url) => match (url.host_str(), url.port()) {
                (Some(host), Some(port)) => format!("{}://{}:{}", url.scheme(), host, port),
                (Some(host), None) => format!("{}://{}", url.scheme(), host),
                _ => self.settings.endpoint.clone(),
            },
            Err(_) => self.settings.endpoint.clone(),
        }
    }

    /// Probe the local Ollama daemon's `/api/tags` endpoint.
    ///
    /// The probe has a 1-second budget so an offline daemon cannot delay the
    /// scan; it expects the configured endpoint to be an Ollama base URL (with
    /// or without `/v1`) and always targets the daemon's root.
    pub async fn ollama_ready(&self) -> bool {
        let Some(url) = ollama_tags_url(&self.settings.endpoint) else {
            return false;
        };
        let Ok(probe) = reqwest::Client::builder()
            .timeout(OLLAMA_PROBE_TIMEOUT)
            .build()
        else {
            return false;
        };

        matches!(
            probe.get(url).send().await,
            Ok(response) if response.status().is_success()
        )
    }

    /// Send a system/user prompt pair and return the completion text.
    pub async fn complete(&self, system: &str, user: &str) -> Result<String, AiClientError> {
        let mut request = ChatRequest {
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
            reasoning_effort: reasoning_effort_for(self.settings.provider, &self.settings.model),
        };

        let url = self.endpoint();
        let (mut status, mut body) = self.post(&url, &request).await?;

        // Lite and older Gemini variants reject `reasoning_effort`; retry once
        // without it so an explicit model override keeps working.
        if status == reqwest::StatusCode::BAD_REQUEST
            && request.reasoning_effort.is_some()
            && body.to_ascii_lowercase().contains("invalid argument")
        {
            debug!(
                provider = self.settings.provider.label(),
                "provider rejected reasoning_effort; retrying without it"
            );
            request.reasoning_effort = None;
            (status, body) = self.post(&url, &request).await?;
        }

        if !status.is_success() {
            return Err(AiClientError::Status {
                url,
                status: status.as_u16(),
                body: snippet(&body, ERROR_BODY_LIMIT),
            });
        }

        parse_chat_response(&body)
    }

    /// POST one chat-completions request and return its status and raw body.
    async fn post(
        &self,
        url: &str,
        request: &ChatRequest<'_>,
    ) -> Result<(reqwest::StatusCode, String), AiClientError> {
        let mut builder = self.http.post(url).json(request);
        if let Some(api_key) = &self.settings.api_key {
            builder = builder.bearer_auth(api_key);
        }
        if self.settings.provider == AiProvider::OpenRouter {
            builder = builder
                .header("HTTP-Referer", OPENROUTER_APP_URL)
                .header("X-Title", OPENROUTER_APP_TITLE);
        }

        let response = builder
            .send()
            .await
            .map_err(|source| AiClientError::Transport {
                url: url.to_string(),
                source,
            })?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|source| AiClientError::Transport {
                url: url.to_string(),
                source,
            })?;
        Ok((status, body))
    }
}

/// Disable hidden reasoning for Gemini thinking models.
///
/// Gemini 3 charges its internal reasoning against `max_tokens` before emitting
/// any text, so the 400-token summary budget can be consumed with an empty or
/// truncated answer. `reasoning_effort = "none"` keeps the whole budget for the
/// visible summary. Lite variants do not accept the field and are left alone.
fn reasoning_effort_for(provider: AiProvider, model: &str) -> Option<&'static str> {
    let is_gemini_thinking =
        provider == AiProvider::Gemini && !model.to_ascii_lowercase().contains("lite");
    is_gemini_thinking.then_some("none")
}

/// Resolve an Ollama-compatible base to the daemon's `/api/tags` URL.
///
/// Ollama serves the OpenAI-compatible API under `/v1` but its health endpoint
/// at the root, so any path on the configured URL is replaced.
fn ollama_tags_url(endpoint: &str) -> Option<reqwest::Url> {
    let mut url = reqwest::Url::parse(endpoint.trim()).ok()?;
    url.set_path("/api/tags");
    url.set_query(None);
    url.set_fragment(None);
    Some(url)
}

/// Resolve a provider base URL to a chat-completions endpoint.
///
/// The resolver is deliberately forgiving: it accepts a bare origin
/// (`https://api.openai.com`), a provider base path (`/v1`, `/v1beta`,
/// `/v1beta/openai`), a full `/chat/completions` endpoint, trailing slashes, and
/// an optional query string or fragment. Only path segments are appended; an
/// existing query is preserved at the end instead of being corrupted.
fn chat_endpoint(provider: AiProvider, base: &str) -> String {
    let (address, suffix) = split_suffix(base.trim());
    let address = address.trim_end_matches('/');

    if address.is_empty() {
        return format!("{}{suffix}", provider.default_endpoint());
    }

    let endpoint = if address.ends_with("/chat/completions") {
        address.to_string()
    } else {
        format!("{address}{}", completion_path(provider, address))
    };

    format!("{endpoint}{suffix}")
}

/// Split an optional `?query` or `#fragment` from the URL path.
fn split_suffix(value: &str) -> (&str, &str) {
    match value.find(['?', '#']) {
        Some(index) => (&value[..index], &value[index..]),
        None => (value, ""),
    }
}

/// Whether the URL already contains a path after its authority.
fn has_path(address: &str) -> bool {
    match address.split_once("://") {
        Some((_, rest)) => rest.contains('/'),
        None => address.contains('/'),
    }
}

/// Path to append to a base URL that does not yet name the completion endpoint.
fn completion_path(provider: AiProvider, address: &str) -> &'static str {
    if !has_path(address) {
        return match provider {
            AiProvider::Openai | AiProvider::Ollama => "/v1/chat/completions",
            AiProvider::Gemini => "/v1beta/openai/chat/completions",
            AiProvider::OpenRouter => "/api/v1/chat/completions",
            AiProvider::Custom => "/chat/completions",
        };
    }

    // Google documents both the bare v1beta root and its OpenAI-compatible
    // sub-path; accept the former instead of producing a 404.
    if provider == AiProvider::Gemini && address.ends_with("/v1beta") {
        return "/openai/chat/completions";
    }

    "/chat/completions"
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

/// Keep an error excerpt short, single-line, and character-safe.
fn snippet(text: &str, limit: usize) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        return "<empty response body>".to_string();
    }

    let mut chars = collapsed.chars();
    let excerpt: String = chars.by_ref().take(limit).collect();
    if chars.next().is_some() {
        format!("{excerpt}…")
    } else {
        excerpt
    }
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    max_tokens: u32,
    temperature: f32,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<&'a str>,
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

    fn settings(provider: AiProvider, endpoint: &str, model: &str) -> AiSettings {
        AiSettings {
            enabled: true,
            provider,
            api_key: None,
            model: model.to_string(),
            endpoint: endpoint.to_string(),
            max_tokens: 400,
            cache_enabled: false,
            force_refresh: false,
        }
    }

    fn resolve(provider: AiProvider, base: &str) -> String {
        AiClient::new(settings(provider, base, "test-model"))
            .expect("client")
            .endpoint()
    }

    /// Serve exactly one HTTP request and return the server root plus the raw
    /// request text, so tests can assert on the path and headers without a
    /// network dependency.
    async fn serve_once(response: String) -> (String, tokio::sync::oneshot::Receiver<String>) {
        let (root, mut requests) = serve_sequence(vec![response]).await;
        let (sender, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            if let Some(request) = requests.recv().await {
                let _ = sender.send(request);
            }
        });
        (root, receiver)
    }

    /// Serve one HTTP request per response, in order, and stream the raw
    /// requests back to the test.
    async fn serve_sequence(
        responses: Vec<String>,
    ) -> (String, tokio::sync::mpsc::UnboundedReceiver<String>) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let address = listener.local_addr().expect("address");
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            for response in responses {
                let (mut socket, _) = listener.accept().await.expect("accept");
                let mut buffer = vec![0u8; 8192];
                let read = socket.read(&mut buffer).await.expect("read request");
                let _ = sender.send(String::from_utf8_lossy(&buffer[..read]).to_string());
                socket
                    .write_all(response.as_bytes())
                    .await
                    .expect("write response");
                let _ = socket.shutdown().await;
            }
        });

        (format!("http://{address}"), receiver)
    }

    fn http_response(status: &str, body: &str) -> String {
        format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
    }

    #[tokio::test]
    async fn http_errors_name_the_url_status_and_body() {
        let body = r#"{"error":{"message":"bad key"}}"#;
        let (root, _) = serve_once(http_response("401 Unauthorized", body)).await;
        let client = AiClient::new(settings(
            AiProvider::Custom,
            &format!("{root}/v1"),
            "test-model",
        ))
        .expect("client");

        let error = client
            .complete("system", "user")
            .await
            .expect_err("status error");
        let rendered = error.to_string();

        assert!(rendered.contains(&format!("{root}/v1/chat/completions")));
        assert!(rendered.contains("HTTP 401"));
        assert!(rendered.contains("bad key"));
    }

    #[tokio::test]
    async fn openrouter_requests_carry_attribution_headers() {
        let body = r#"{"choices":[{"message":{"role":"assistant","content":"ok"}}]}"#;
        let (root, request) = serve_once(http_response("200 OK", body)).await;
        let client = AiClient::new(settings(
            AiProvider::OpenRouter,
            &format!("{root}/api/v1"),
            "test-model",
        ))
        .expect("client");

        assert_eq!(
            client.complete("system", "user").await.expect("completion"),
            "ok"
        );

        let request = request.await.expect("request");
        let lower = request.to_ascii_lowercase();
        assert!(request.starts_with("POST /api/v1/chat/completions HTTP/1.1"));
        assert!(lower.contains(&format!(
            "http-referer: {}",
            OPENROUTER_APP_URL.to_ascii_lowercase()
        )));
        assert!(lower.contains(&format!(
            "x-title: {}",
            OPENROUTER_APP_TITLE.to_ascii_lowercase()
        )));
    }

    #[tokio::test]
    async fn retries_without_reasoning_effort_when_the_provider_rejects_it() {
        let error = r#"{"error":{"message":"Request contains an invalid argument."}}"#;
        let success = r#"{"choices":[{"message":{"role":"assistant","content":"ok"}}]}"#;
        let (root, mut requests) = serve_sequence(vec![
            http_response("400 Bad Request", error),
            http_response("200 OK", success),
        ])
        .await;
        let client =
            AiClient::new(settings(AiProvider::Gemini, &root, "gemini-3.5-flash")).expect("client");

        assert_eq!(
            client.complete("system", "user").await.expect("completion"),
            "ok"
        );

        let first = requests.recv().await.expect("first request");
        let second = requests.recv().await.expect("retry");
        assert!(first.contains("\"reasoning_effort\":\"none\""));
        assert!(!second.contains("reasoning_effort"));
    }

    #[test]
    fn only_gemini_thinking_models_disable_reasoning() {
        assert_eq!(
            reasoning_effort_for(AiProvider::Gemini, "gemini-3.5-flash"),
            Some("none")
        );
        assert_eq!(
            reasoning_effort_for(AiProvider::Gemini, "gemini-3.5-flash-lite"),
            None
        );
        assert_eq!(
            reasoning_effort_for(AiProvider::Openai, "gpt-4o-mini"),
            None
        );
        assert_eq!(
            reasoning_effort_for(AiProvider::OpenRouter, "google/gemini-3.5-flash"),
            None
        );
    }

    #[test]
    fn builds_the_ollama_probe_url_and_origin() {
        assert_eq!(
            ollama_tags_url("http://localhost:11434/v1")
                .expect("url")
                .as_str(),
            "http://localhost:11434/api/tags"
        );
        assert_eq!(
            ollama_tags_url("http://localhost:11434/v1/chat/completions?x=1")
                .expect("url")
                .as_str(),
            "http://localhost:11434/api/tags"
        );

        let client = AiClient::new(settings(
            AiProvider::Ollama,
            "http://localhost:11434/v1",
            "llama3.2:3b",
        ))
        .expect("client");
        assert_eq!(client.ollama_origin(), "http://localhost:11434");
    }

    #[test]
    fn resolves_every_provider_from_its_base_url() {
        assert_eq!(
            resolve(AiProvider::Openai, "https://api.openai.com/v1"),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            resolve(
                AiProvider::Gemini,
                "https://generativelanguage.googleapis.com/v1beta/openai"
            ),
            "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"
        );
        assert_eq!(
            resolve(AiProvider::OpenRouter, "https://openrouter.ai/api/v1"),
            "https://openrouter.ai/api/v1/chat/completions"
        );
        assert_eq!(
            resolve(AiProvider::Ollama, "http://localhost:11434/v1"),
            "http://localhost:11434/v1/chat/completions"
        );
        assert_eq!(
            resolve(AiProvider::Custom, "https://llm.example/v1"),
            "https://llm.example/v1/chat/completions"
        );
    }

    #[test]
    fn completes_bare_origins_with_the_provider_root() {
        assert_eq!(
            resolve(AiProvider::Openai, "https://api.openai.com"),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            resolve(
                AiProvider::Gemini,
                "https://generativelanguage.googleapis.com"
            ),
            "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"
        );
        assert_eq!(
            resolve(AiProvider::OpenRouter, "https://openrouter.ai"),
            "https://openrouter.ai/api/v1/chat/completions"
        );
        assert_eq!(
            resolve(AiProvider::Ollama, "http://localhost:11434"),
            "http://localhost:11434/v1/chat/completions"
        );
        assert_eq!(
            resolve(AiProvider::Custom, "https://llm.example"),
            "https://llm.example/chat/completions"
        );
    }

    #[test]
    fn accepts_gemini_roots_trailing_slashes_and_full_endpoints() {
        assert_eq!(
            resolve(
                AiProvider::Gemini,
                "https://generativelanguage.googleapis.com/v1beta"
            ),
            "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"
        );
        assert_eq!(
            resolve(
                AiProvider::Gemini,
                "https://generativelanguage.googleapis.com/v1beta/openai/"
            ),
            "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"
        );
        assert_eq!(
            resolve(
                AiProvider::Openai,
                "https://api.openai.com/v1/chat/completions/"
            ),
            "https://api.openai.com/v1/chat/completions"
        );
        assert_eq!(
            resolve(
                AiProvider::Custom,
                "  https://proxy.example/v1/chat/completions/  "
            ),
            "https://proxy.example/v1/chat/completions"
        );
    }

    #[test]
    fn preserves_query_strings_and_fragments() {
        assert_eq!(
            resolve(AiProvider::Openai, "https://api.openai.com/v1?key=abc"),
            "https://api.openai.com/v1/chat/completions?key=abc"
        );
        assert_eq!(
            resolve(
                AiProvider::Custom,
                "https://proxy.example/v1/chat/completions?key=abc"
            ),
            "https://proxy.example/v1/chat/completions?key=abc"
        );
    }

    #[test]
    fn rejects_a_custom_provider_without_endpoint_or_model() {
        assert!(matches!(
            AiClient::new(settings(AiProvider::Custom, "", "my-model")),
            Err(AiClientError::Configuration(_))
        ));
        assert!(matches!(
            AiClient::new(settings(AiProvider::Custom, "https://llm.example/v1", "")),
            Err(AiClientError::Configuration(_))
        ));
    }

    #[test]
    fn error_snippets_are_compact_and_character_safe() {
        assert_eq!(
            snippet("  rate\n\tlimited  ", ERROR_BODY_LIMIT),
            "rate limited"
        );
        assert_eq!(snippet(" \n\t ", ERROR_BODY_LIMIT), "<empty response body>");
        assert_eq!(snippet("aébc", 3), "aéb…");
    }

    #[test]
    fn quota_and_auth_statuses_stop_the_scan() {
        for status in [401, 402, 403, 404, 429] {
            let error = AiClientError::Status {
                url: "https://api.example/v1/chat/completions".to_string(),
                status,
                body: "denied".to_string(),
            };
            assert!(error.is_fatal(), "status {status}");
        }

        let quota_in_body = AiClientError::Status {
            url: "https://api.example/v1/chat/completions".to_string(),
            status: 400,
            body: "You exceeded your current quota".to_string(),
        };
        assert!(quota_in_body.is_fatal());

        let transient = AiClientError::Status {
            url: "https://api.example/v1/chat/completions".to_string(),
            status: 500,
            body: "server error".to_string(),
        };
        assert!(!transient.is_fatal());
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
}
