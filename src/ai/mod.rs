//! Per-library AI changelog summarizer.
//!
//! Each outdated, registry-resolved dependency gets its own OpenAI-compatible
//! request and a cached Markdown analysis. Configuration is layered (files,
//! `RUSTYDEPS_AI_*` environment variables, CLI flags) and every AI failure is
//! non-fatal: the report simply renders without an analysis.

pub mod cache;
pub mod client;
pub mod prompt;
pub mod summary;

pub use summary::Summarizer;
