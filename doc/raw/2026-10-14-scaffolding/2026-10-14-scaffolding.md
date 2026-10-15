# TASK: Context Setup, Agent Skills Discovery & Documentation Architecture for RustyDeps

## Context
RustyDeps is a high-performance CLI tool written in **Rust** that scans local repositories for dependency manifests (`Cargo.toml`, `package.json`, `go.mod`, etc.), evaluates version upgrades (SemVer) by querying official registries in parallel, and generates an interactive HTML report with AI-driven changelog summaries.

We need to configure the project's agent directives, discover existing skills in `.claude/`, setup the `doc/` directory structure (`doc/raw/` and `doc/wiki/`), and initialize the OKF-compliant LLM-Wiki using the project's automated tooling.

---

## Instructions

### Step 0: Discovery of Skills & Tooling (`.claude/`)
1. **Scan `.claude/` Directory:** Inspect the contents of `.claude/` to discover all existing skills, custom prompt templates, and Python scripts designed for:
   - Wiki generation and structure enforcement.
   - OKF (Open Knowledge Foundation) compliance validation.
   - Prompt analysis and agent task validation.
2. **Acknowledge and Utilize:** Register these tools into your execution context. Whenever you build or validate the wiki in `doc/wiki/`, you MUST execute and rely on the Python scripts provided in `.claude/`.

---

### Step 1: Update Root Configuration Files (`AGENTS.md` & `CLAUDE.md`)
Update the contents of `AGENTS.md` and `CLAUDE.md` in the project root (removing any legacy references to React, TypeScript, Vite, etc.) with the following project rules:

1. **Project Stack & Principles:**
   - **Language:** Rust (2021/2024 edition). Idiomatic, safe, and high-performance code.
   - **Architecture:** Trait-based (`PackageParser` trait to decouple ecosystem parsers). Asynchronous concurrency with `tokio` and HTTP client `reqwest`. File system traversal using `ignore` (automatically respecting `.gitignore`).
   - **CLI Framework:** `clap` (derive macro pattern).
   - **Documentation:** LLM-Wiki in `doc/wiki/` (atomic Markdown, human & LLM friendly, compliant with Open Knowledge Foundation - OKF standards).
   - **Raw Knowledge & Specifications:** Source documents, initial specs, and prompt files in `doc/raw/`.
   - **Skills & Validation Tools:** Pre-existing skills and Python validation scripts located in `.claude/`.

2. **Code Guidelines & Quality:**
   - Error handling via `thiserror` for internal modules/libraries and `anyhow` for CLI binary handlers.
   - No loose unwrap statements (`unwrap()` forbidden in production; use `?` or explicit `match`/`if let`).
   - Standard formatting with `cargo fmt` and zero-warning policy on `cargo clippy`.
   - Unit tests required for every Rust module.

3. **Development Workflow & Report:**
   - Scan `.claude/` on startup to load pre-configured skills and Python scripts.
   - Use the `.claude/` Python tools whenever modifying or validating `doc/wiki/`.
   - At the end of every development step, generate a `development-report-[TIMESTAMP].md` in `doc/raw/` detailing: changes made, tests executed, technical debt, and next recommended actions.

---

### Step 2: Initialize Directory Structure
Ensure the following directory structure exists:
- `doc/raw/`
- `doc/wiki/`

Inside `doc/raw/`, create/verify the file `rustydeps-spec.md` containing the core architectural requirements for RustyDeps (Rust CLI, manifest scanning, classification into Safe/Risky/Breaking updates, HTML report engine, and AI module).

---

### Step 3: Initialize & Validate LLM-Wiki (`doc/wiki/`)
Use the Python scripts discovered in `.claude/` to generate and validate the initial Markdown files in `doc/wiki/`. Ensure every wiki file includes an **OKF-compliant YAML frontmatter** (e.g., `title`, `license`, `version`, `status`, `metadata`).

Required initial files:
1. **`doc/wiki/index.md`**
   - Main entrypoint and knowledge graph map.
   - Relative Markdown links to all other wiki pages.
2. **`doc/wiki/architecture.md`**
   - Rust architecture breakdown (CLI, File System Walk, Parser Trait, Registry Clients, Report Generator, AI Module).
   - Core DTO definitions (`Dependency`, `UpdateType`).
3. **`doc/wiki/roadmap.md`**
   - Tracking the 6 project development phases:
     - Phase 1: Rust Setup & CLI (`clap` + `ignore`)
     - Phase 2: Parsers (Cargo & npm) & Registry HTTP Clients
     - Phase 3: Unified DTO & SemVer Logic
     - Phase 4: Interactive HTML Report Engine
     - Phase 5: AI Changelog Summarizer Module
     - Phase 6: Multi-ecosystem Expansion (PyPI, Go, Composer)
4. **`doc/wiki/okf-compliance.md`**
   - Guidelines on open data formats (Markdown, JSON Schema), open licensing (MIT/Apache-2.0 for code, CC-BY-4.0 for wiki), and metadata traceability.

Run the `.claude/` validation script on `doc/wiki/` to confirm 100% OKF compliance.

---

## Execution & Verification
1. Scan `.claude/` and list all detected skills and scripts.
2. Update `AGENTS.md`, `CLAUDE.md`, and create directory structures.
3. Run `.claude/` scripts to generate/validate `doc/wiki/`.
4. Generate `doc/raw/development-report-[TIMESTAMP].md` summarizing execution results.
