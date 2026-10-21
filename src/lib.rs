//! RustyDeps library surface.
//!
//! The binary entrypoint lives in `src/main.rs`; every module is exposed here so
//! integration tests can exercise the AI client, cache, and report without
//! spawning the CLI.

pub mod ai;
pub mod classifier;
pub mod cli;
pub mod config;
pub mod model;
pub mod parser;
pub mod progress;
pub mod registry;
pub mod report;
pub mod scanner;
