//! AI summary module (Phase 3 scaffold).
//!
//! Phase 3 detects whether an LLM provider is configured and always produces a
//! deterministic templated summary for at-risk and breaking updates. Live
//! provider calls (Anthropic / OpenAI / a custom endpoint) land in Phase 5, the
//! dedicated AI changelog summarizer.

pub mod summary;

pub use summary::Summarizer;
