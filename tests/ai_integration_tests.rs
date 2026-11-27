//! End-to-end tests for the OpenAI-compatible AI client and summarizer.
//!
//! A `wiremock` server stands in for the provider, so the tests exercise the
//! real HTTP path, response parser, fatal-error breaker, and Ollama preflight
//! without credentials or outbound network access.

use std::path::PathBuf;

use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use rustydeps::ai::client::AiClient;
use rustydeps::ai::summary::{Summarizer, MAX_CONCURRENCY};
use rustydeps::classifier::ClassifiedDependency;
use rustydeps::config::{AiProvider, AiSettings};
use rustydeps::model::{Ecosystem, UpdateType};

fn settings(server: &MockServer, provider: AiProvider) -> AiSettings {
    AiSettings {
        enabled: true,
        provider,
        api_key: None,
        model: "test-model".to_string(),
        endpoint: format!("{}/v1", server.uri()),
        max_tokens: 400,
        cache_enabled: false,
        force_refresh: false,
    }
}

fn dependency(index: usize, update_type: UpdateType) -> ClassifiedDependency {
    ClassifiedDependency {
        name: format!("package-{index}"),
        ecosystem: Ecosystem::Cargo,
        current_version: "1.0.0".to_string(),
        latest_version: Some("2.0.0".to_string()),
        update_type,
        manifest_path: PathBuf::from("Cargo.toml"),
        is_dev_dependency: false,
        ai_summary: None,
        ai_model: None,
        ai_cached: false,
    }
}

fn completion_body(content: &str) -> serde_json::Value {
    json!({
        "id": "chatcmpl-test",
        "object": "chat.completion",
        "choices": [
            {
                "index": 0,
                "message": { "role": "assistant", "content": content },
                "finish_reason": "stop"
            }
        ]
    })
}

#[tokio::test]
async fn parses_a_successful_chat_completion() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(completion_body(
            "### 🚨 Breaking Changes\nUse `new_api` instead.",
        )))
        .mount(&server)
        .await;

    let client = AiClient::new(settings(&server, AiProvider::Custom)).expect("client");
    let summary = client
        .complete("system prompt", "user prompt")
        .await
        .expect("completion");

    assert_eq!(summary, "### 🚨 Breaking Changes\nUse `new_api` instead.");

    let requests = server.received_requests().await.expect("requests");
    assert_eq!(requests.len(), 1);
    let body: serde_json::Value = serde_json::from_slice(&requests[0].body).expect("request json");
    assert_eq!(body["model"], "test-model");
    assert_eq!(body["messages"][0]["role"], "system");
    assert_eq!(body["messages"][1]["role"], "user");
}

#[tokio::test]
async fn a_rate_limit_response_trips_the_circuit_breaker() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(429).set_body_json(json!({
            "error": { "message": "You exceeded your current quota" }
        })))
        .mount(&server)
        .await;

    let summarizer = Summarizer::from_settings(settings(&server, AiProvider::Custom), None);
    let mut dependencies: Vec<_> = (0..10)
        .map(|index| dependency(index, UpdateType::Breaking))
        .collect();

    summarizer.summarize_all(&mut dependencies).await;

    assert!(dependencies
        .iter()
        .all(|dependency| dependency.ai_summary.is_none()));

    let requests = server.received_requests().await.expect("requests");
    assert!(!requests.is_empty(), "the first request must be attempted");
    assert!(
        requests.len() <= MAX_CONCURRENCY,
        "the breaker must stop packages beyond the in-flight batch, saw {}",
        requests.len()
    );
    assert!(
        requests.len() < dependencies.len(),
        "the breaker must skip the remaining packages"
    );
}

#[tokio::test]
async fn a_not_found_response_is_fatal() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "error": { "message": "model not found" }
        })))
        .mount(&server)
        .await;

    let client = AiClient::new(settings(&server, AiProvider::Custom)).expect("client");
    let error = client
        .complete("system prompt", "user prompt")
        .await
        .expect_err("404 must fail");

    assert!(error.is_fatal(), "404 must stop the remaining requests");
    let message = error.to_string();
    assert!(message.contains("404"), "{message}");
    assert!(message.contains("/v1/chat/completions"), "{message}");
}

#[tokio::test]
async fn an_offline_ollama_daemon_skips_every_chat_request() {
    let server = MockServer::start().await;

    // The daemon probe fails, while a working chat endpoint is mounted to prove
    // that no package-level request is attempted.
    Mock::given(method("GET"))
        .and(path("/api/tags"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(completion_body("unused")))
        .mount(&server)
        .await;

    let summarizer = Summarizer::from_settings(settings(&server, AiProvider::Ollama), None);
    let mut dependencies: Vec<_> = (0..3)
        .map(|index| dependency(index, UpdateType::Safe))
        .collect();

    summarizer.summarize_all(&mut dependencies).await;

    assert!(dependencies
        .iter()
        .all(|dependency| dependency.ai_summary.is_none()));

    let requests = server.received_requests().await.expect("requests");
    let chat_requests = requests
        .iter()
        .filter(|request| request.url.path() == "/v1/chat/completions")
        .count();
    assert_eq!(chat_requests, 0, "an offline daemon must skip chat calls");
    assert_eq!(requests.len(), 1, "only the /api/tags probe should run");
}
