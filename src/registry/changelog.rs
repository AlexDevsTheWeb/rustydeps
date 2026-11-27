//! Official release-note fetching for AI prompt grounding.
//!
//! Registries rarely carry changelogs, but most packages publish a repository
//! link. The fetcher resolves that link from the registry metadata for the
//! package's ecosystem, then asks the GitHub Releases API for the release that
//! matches the upgrade target. A missing repository, an unknown tag, a network
//! failure, or an empty body all degrade to `None`; the AI summarizer then
//! falls back to its ungrounded prompt.
//!
//! `GITHUB_TOKEN` (or `GH_TOKEN`) is optional but recommended: unauthenticated
//! GitHub API requests are limited to 60 per hour, so a large scan can exhaust
//! the budget. Every fetched body is sanitized and truncated to
//! [`MAX_CHANGELOG_CHARS`] before it reaches the prompt.

use std::collections::BTreeMap;
use std::time::Duration;

use serde::Deserialize;
use tracing::debug;

use crate::model::Ecosystem;

/// Maximum changelog characters injected into one prompt.
pub const MAX_CHANGELOG_CHARS: usize = 1500;

/// Overall budget for one package's changelog lookup (metadata + release).
pub const FETCH_TIMEOUT: Duration = Duration::from_secs(8);

/// Per-request timeout inside a lookup.
const HTTP_TIMEOUT: Duration = Duration::from_secs(5);

const GITHUB_API: &str = "https://api.github.com";
const CRATES_API: &str = "https://crates.io/api/v1/crates";
const NPM_REGISTRY: &str = "https://registry.npmjs.org";
const PYPI_API: &str = "https://pypi.org/pypi";
const PACKAGIST_API: &str = "https://repo.packagist.org/p2";

/// Fetches release notes for package upgrades.
#[derive(Clone)]
pub struct ChangelogFetcher {
    http: Option<reqwest::Client>,
    github_token: Option<String>,
    github_api: String,
    crates_api: String,
    npm_registry: String,
    pypi_api: String,
    packagist_api: String,
}

impl Default for ChangelogFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ChangelogFetcher {
    /// Build a fetcher with a descriptive `User-Agent` and an optional token.
    pub fn new() -> Self {
        let user_agent = format!(
            "{name}/{version} ({repository})",
            name = env!("CARGO_PKG_NAME"),
            version = env!("CARGO_PKG_VERSION"),
            repository = env!("CARGO_PKG_REPOSITORY"),
        );
        let http = reqwest::Client::builder()
            .timeout(HTTP_TIMEOUT)
            .user_agent(user_agent)
            .build()
            .ok();

        Self {
            http,
            github_token: github_token(),
            github_api: GITHUB_API.to_string(),
            crates_api: CRATES_API.to_string(),
            npm_registry: NPM_REGISTRY.to_string(),
            pypi_api: PYPI_API.to_string(),
            packagist_api: PACKAGIST_API.to_string(),
        }
    }

    /// A fetcher that never performs a request (tests and `--offline` runs).
    pub fn disabled() -> Self {
        Self {
            http: None,
            github_token: None,
            github_api: GITHUB_API.to_string(),
            crates_api: CRATES_API.to_string(),
            npm_registry: NPM_REGISTRY.to_string(),
            pypi_api: PYPI_API.to_string(),
            packagist_api: PACKAGIST_API.to_string(),
        }
    }

    /// Whether lookups are possible.
    pub fn is_enabled(&self) -> bool {
        self.http.is_some()
    }

    /// Fetch the release notes for `package` at `target_version`.
    ///
    /// Returns sanitized, truncated Markdown, or `None` when no official notes
    /// can be resolved. The whole lookup is bounded by [`FETCH_TIMEOUT`].
    pub async fn fetch(
        &self,
        ecosystem: Ecosystem,
        package: &str,
        target_version: &str,
    ) -> Option<String> {
        self.http.as_ref()?;

        match tokio::time::timeout(
            FETCH_TIMEOUT,
            self.fetch_inner(ecosystem, package, target_version),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => {
                debug!(
                    package,
                    target = %target_version,
                    "changelog lookup timed out; continuing without official notes"
                );
                None
            }
        }
    }

    async fn fetch_inner(
        &self,
        ecosystem: Ecosystem,
        package: &str,
        target_version: &str,
    ) -> Option<String> {
        let http = self.http.as_ref()?;
        let repository = self
            .repository(http, ecosystem, package, target_version)
            .await?;
        let (owner, repo) = github_slug(&repository)?;
        let release = self.release(http, &owner, &repo, target_version).await?;
        let text = release_text(&release);

        if text.is_empty() {
            None
        } else {
            Some(truncate_chars(&text, MAX_CHANGELOG_CHARS))
        }
    }

    /// Resolve the package's repository URL from its registry metadata.
    async fn repository(
        &self,
        http: &reqwest::Client,
        ecosystem: Ecosystem,
        package: &str,
        target_version: &str,
    ) -> Option<String> {
        match ecosystem {
            Ecosystem::Cargo => self.cargo_repository(http, package).await,
            Ecosystem::Npm => self.npm_repository(http, package).await,
            Ecosystem::Pypi => self.pypi_repository(http, package).await,
            Ecosystem::Go => go_repository(package),
            Ecosystem::Composer => {
                self.composer_repository(http, package, target_version)
                    .await
            }
        }
    }

    async fn cargo_repository(&self, http: &reqwest::Client, package: &str) -> Option<String> {
        let url = format!("{}/{package}", self.crates_api);
        let body = get_text(http, &url).await?;
        let meta: CrateMeta = serde_json::from_str(&body).ok()?;
        meta.krate.repository
    }

    async fn npm_repository(&self, http: &reqwest::Client, package: &str) -> Option<String> {
        let encoded = package.replace('/', "%2F");
        let url = format!("{}/{encoded}/latest", self.npm_registry);
        let body = get_text(http, &url).await?;
        let meta: NpmLatest = serde_json::from_str(&body).ok()?;
        match meta.repository? {
            NpmRepository::Url(url) => Some(url),
            NpmRepository::Detailed { url } => url,
        }
    }

    async fn pypi_repository(&self, http: &reqwest::Client, package: &str) -> Option<String> {
        let url = format!("{}/{package}/json", self.pypi_api);
        let body = get_text(http, &url).await?;
        let meta: PypiMeta = serde_json::from_str(&body).ok()?;

        let urls = meta
            .info
            .project_urls
            .values()
            .map(String::as_str)
            .chain(meta.info.home_page.as_deref())
            .chain(meta.info.project_url.as_deref());

        urls.filter(|url| github_slug(url).is_some())
            .map(str::to_string)
            .next()
    }

    async fn composer_repository(
        &self,
        http: &reqwest::Client,
        package: &str,
        target_version: &str,
    ) -> Option<String> {
        let url = format!("{}/{package}.json", self.packagist_api);
        let body = get_text(http, &url).await?;
        let meta: PackagistResponse = serde_json::from_str(&body).ok()?;

        let mut fallback = None;
        for entry in meta.packages.values().flatten() {
            if versions_match(&entry.version, target_version) {
                return entry.source.as_ref().and_then(|source| source.url.clone());
            }
            fallback.get_or_insert(entry);
        }

        fallback
            .and_then(|entry| entry.source.as_ref())
            .and_then(|source| source.url.clone())
    }

    /// Fetch the GitHub release for one tag candidate set.
    async fn release(
        &self,
        http: &reqwest::Client,
        owner: &str,
        repo: &str,
        target_version: &str,
    ) -> Option<GithubRelease> {
        for tag in tag_candidates(target_version) {
            match self.release_by_tag(http, owner, repo, &tag).await {
                TagLookup::Found(release) if !release_text(&release).is_empty() => {
                    return Some(release);
                }
                TagLookup::Found(_) | TagLookup::Missing => {}
                TagLookup::Failed => return None,
            }
        }

        // Workspace tags often prefix the version (`tokio-1.40.0`,
        // `serde-v1.0.200`); one bounded list request covers those.
        self.release_from_list(http, owner, repo, target_version)
            .await
    }

    /// Look up one exact release tag.
    async fn release_by_tag(
        &self,
        http: &reqwest::Client,
        owner: &str,
        repo: &str,
        tag: &str,
    ) -> TagLookup {
        let Some(url) = release_url(&self.github_api, owner, repo, tag) else {
            return TagLookup::Missing;
        };

        let response = match self.github_get(http, url).send().await {
            Ok(response) => response,
            Err(error) => {
                debug!(
                    %error,
                    owner,
                    repo,
                    tag,
                    "GitHub release lookup failed; continuing without official notes"
                );
                return TagLookup::Failed;
            }
        };

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return TagLookup::Missing;
        }
        if !response.status().is_success() {
            debug!(
                status = %response.status(),
                owner,
                repo,
                tag,
                "GitHub release lookup returned an error status"
            );
            return TagLookup::Failed;
        }

        match response.json::<GithubRelease>().await {
            Ok(release) => TagLookup::Found(release),
            Err(_) => TagLookup::Failed,
        }
    }

    /// Find a release whose tag ends with the target version.
    async fn release_from_list(
        &self,
        http: &reqwest::Client,
        owner: &str,
        repo: &str,
        target_version: &str,
    ) -> Option<GithubRelease> {
        let mut url = reqwest::Url::parse(&self.github_api).ok()?;
        {
            let mut segments = url.path_segments_mut().ok()?;
            segments
                .push("repos")
                .push(owner)
                .push(repo)
                .push("releases");
        }
        url.query_pairs_mut().append_pair("per_page", "100");

        let response = self.github_get(http, url).send().await.ok()?;
        if !response.status().is_success() {
            debug!(
                status = %response.status(),
                owner,
                repo,
                "GitHub release list lookup returned an error status"
            );
            return None;
        }

        let releases: Vec<GithubRelease> = response.json().await.ok()?;
        let candidates = tag_candidates(target_version);

        releases.into_iter().find(|release| {
            release
                .tag_name
                .as_deref()
                .is_some_and(|tag| tag_matches(tag, &candidates))
                && !release_text(release).is_empty()
        })
    }

    /// Build an authenticated GitHub API request.
    fn github_get(&self, http: &reqwest::Client, url: reqwest::Url) -> reqwest::RequestBuilder {
        let request = http
            .get(url)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28");
        match &self.github_token {
            Some(token) => request.bearer_auth(token),
            None => request,
        }
    }
}

/// Result of looking up one exact release tag.
enum TagLookup {
    Found(GithubRelease),
    Missing,
    Failed,
}

/// Whether a release tag names the target version, including workspace prefixes
/// such as `tokio-1.40.0` and `serde-v1.0.200`.
fn tag_matches(tag: &str, candidates: &[String]) -> bool {
    candidates.iter().any(|candidate| {
        tag == candidate
            || tag.ends_with(&format!("-{candidate}"))
            || tag.ends_with(&format!("/{candidate}"))
    })
}

/// Read an optional GitHub token from the environment.
fn github_token() -> Option<String> {
    std::env::var("GITHUB_TOKEN")
        .or_else(|_| std::env::var("GH_TOKEN"))
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// GET a URL and return its body only on a 2xx response.
async fn get_text(http: &reqwest::Client, url: &str) -> Option<String> {
    let response = http.get(url).send().await.ok()?;
    if !response.status().is_success() {
        debug!(url, status = %response.status(), "changelog metadata lookup failed");
        return None;
    }
    response.text().await.ok()
}

#[derive(Debug, Deserialize)]
struct CrateMeta {
    #[serde(rename = "crate")]
    krate: CrateMetaInfo,
}

#[derive(Debug, Deserialize)]
struct CrateMetaInfo {
    #[serde(default)]
    repository: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NpmLatest {
    #[serde(default)]
    repository: Option<NpmRepository>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum NpmRepository {
    Url(String),
    Detailed {
        #[serde(default)]
        url: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
struct PypiMeta {
    info: PypiInfo,
}

#[derive(Debug, Default, Deserialize)]
struct PypiInfo {
    #[serde(default)]
    project_urls: BTreeMap<String, String>,
    #[serde(default)]
    home_page: Option<String>,
    #[serde(default)]
    project_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PackagistResponse {
    #[serde(default)]
    packages: BTreeMap<String, Vec<PackagistVersion>>,
}

#[derive(Debug, Deserialize)]
struct PackagistVersion {
    version: String,
    #[serde(default)]
    source: Option<PackagistSource>,
}

#[derive(Debug, Deserialize)]
struct PackagistSource {
    #[serde(default)]
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    tag_name: Option<String>,
    #[serde(default)]
    body: Option<String>,
}

/// Resolve a Go module path that is hosted on GitHub.
fn go_repository(package: &str) -> Option<String> {
    let lower = package.to_ascii_lowercase();
    lower
        .starts_with("github.com/")
        .then(|| format!("https://{package}"))
}

/// Build a GitHub API URL for one release tag.
fn release_url(api: &str, owner: &str, repo: &str, tag: &str) -> Option<reqwest::Url> {
    let mut url = reqwest::Url::parse(api).ok()?;
    {
        let mut segments = url.path_segments_mut().ok()?;
        segments
            .push("repos")
            .push(owner)
            .push(repo)
            .push("releases")
            .push("tags")
            .push(tag);
    }
    Some(url)
}

/// Tags to try for a version: the version as published, then the `v` variant.
fn tag_candidates(target_version: &str) -> Vec<String> {
    let target = target_version.trim();
    if target.is_empty() {
        return Vec::new();
    }

    let mut tags = vec![target.to_string()];
    match target.strip_prefix('v') {
        Some(stripped) if !stripped.is_empty() => tags.push(stripped.to_string()),
        Some(_) => {}
        None => tags.push(format!("v{target}")),
    }
    tags
}

/// Compare a registry version and an upgrade target, tolerating a `v` prefix.
fn versions_match(left: &str, right: &str) -> bool {
    left.trim_start_matches('v') == right.trim_start_matches('v')
}

/// Extract `(owner, repo)` from a GitHub repository URL.
///
/// Accepts `https://github.com/o/r`, `git+https://…`, `git@github.com:o/r.git`,
/// SSH URLs, and a trailing path. Rejects hosts such as `notgithub.com` and
/// `github.com.evil.example`.
fn github_slug(raw: &str) -> Option<(String, String)> {
    let raw = raw.trim();
    let lower = raw.to_ascii_lowercase();
    let start = lower.find("github.com")?;

    if start > 0 && raw.as_bytes()[start - 1].is_ascii_alphanumeric() {
        return None;
    }

    let rest = &raw[start + "github.com".len()..];
    let rest = rest.strip_prefix(':').or_else(|| rest.strip_prefix('/'))?;
    let path = rest.split(['?', '#']).next()?;
    let mut parts = path
        .trim_start_matches('/')
        .split('/')
        .filter(|part| !part.is_empty());

    let owner = parts.next()?;
    let repo = parts.next()?.trim_end_matches(".git");

    if !valid_github_part(owner) || !valid_github_part(repo) {
        return None;
    }

    Some((owner.to_string(), repo.to_string()))
}

fn valid_github_part(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
}

/// Sanitize a release body: normalize newlines, drop control characters, and
/// collapse runs of blank lines.
fn clean_text(raw: &str) -> String {
    let normalized = raw.replace("\r\n", "\n").replace('\r', "\n");
    let mut output = String::with_capacity(normalized.len());
    let mut blank_lines = 0;

    for line in normalized.split('\n') {
        let line = line.trim_end();
        if line.trim().is_empty() {
            blank_lines += 1;
            if blank_lines > 1 {
                continue;
            }
            output.push('\n');
        } else {
            blank_lines = 0;
            output.extend(
                line.chars()
                    .filter(|character| !character.is_control() || *character == '\t'),
            );
            output.push('\n');
        }
    }

    output.trim().to_string()
}

/// Compose the injectable text from a release name and body.
fn release_text(release: &GithubRelease) -> String {
    let body = release.body.as_deref().map(clean_text).unwrap_or_default();
    if body.is_empty() {
        return String::new();
    }

    let name = release
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty());
    let tag = release
        .tag_name
        .as_deref()
        .map(str::trim)
        .filter(|tag| !tag.is_empty());

    match name {
        Some(name) if Some(name) != tag => format!("{name}\n\n{body}"),
        _ => body,
    }
}

/// Truncate to `limit` characters (not bytes) and mark the cut with `…`.
pub fn truncate_chars(text: &str, limit: usize) -> String {
    if limit == 0 {
        return String::new();
    }
    if text.chars().count() <= limit {
        return text.to_string();
    }

    let mut truncated: String = text.chars().take(limit).collect();
    truncated.push('…');
    truncated
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    fn fetcher(server: &MockServer) -> ChangelogFetcher {
        ChangelogFetcher {
            http: Some(reqwest::Client::new()),
            github_token: None,
            github_api: server.uri(),
            crates_api: format!("{}/crates", server.uri()),
            npm_registry: format!("{}/npm", server.uri()),
            pypi_api: format!("{}/pypi", server.uri()),
            packagist_api: format!("{}/p2", server.uri()),
        }
    }

    #[test]
    fn parses_github_repository_urls() {
        for url in [
            "https://github.com/tokio-rs/tokio",
            "https://github.com/tokio-rs/tokio.git",
            "git+https://github.com/tokio-rs/tokio.git",
            "git@github.com:tokio-rs/tokio.git",
            "ssh://git@github.com/tokio-rs/tokio",
            "https://www.github.com/tokio-rs/tokio/tree/main",
        ] {
            assert_eq!(
                github_slug(url),
                Some(("tokio-rs".to_string(), "tokio".to_string())),
                "{url}"
            );
        }

        for url in [
            "https://gitlab.com/tokio-rs/tokio",
            "https://notgithub.com/tokio-rs/tokio",
            "https://github.com.evil.example/tokio-rs/tokio",
            "https://github.com/",
        ] {
            assert_eq!(github_slug(url), None, "{url}");
        }
    }

    #[test]
    fn builds_tag_candidates_for_both_prefix_styles() {
        assert_eq!(tag_candidates("2.0.0"), ["2.0.0", "v2.0.0"]);
        assert_eq!(tag_candidates("v1.10.0"), ["v1.10.0", "1.10.0"]);
        assert_eq!(tag_candidates("v"), ["v"]);
    }

    #[test]
    fn truncates_on_character_boundaries() {
        assert_eq!(truncate_chars("short", 10), "short");
        assert_eq!(truncate_chars("aébc", 3), "aéb…");
        assert_eq!(truncate_chars("anything", 0), "");
        assert_eq!(truncate_chars("abcd", 2).chars().count(), 3);
    }

    #[test]
    fn cleans_control_characters_and_blank_runs() {
        let text = clean_text("Title\r\n\r\n\r\n- fix\u{0} one\r\n\r\n- fix two   ");
        assert_eq!(text, "Title\n\n- fix one\n\n- fix two");
    }

    #[test]
    fn composes_name_and_body_without_repeating_the_tag() {
        let release = GithubRelease {
            name: Some("Version 2.0.0".to_string()),
            tag_name: Some("v2.0.0".to_string()),
            body: Some("Changes".to_string()),
        };
        assert_eq!(release_text(&release), "Version 2.0.0\n\nChanges");

        let release = GithubRelease {
            name: Some("v2.0.0".to_string()),
            tag_name: Some("v2.0.0".to_string()),
            body: Some("Changes".to_string()),
        };
        assert_eq!(release_text(&release), "Changes");
    }

    #[tokio::test]
    async fn fetches_cargo_release_notes_through_github() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/crates/tokio"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "crate": { "repository": "https://github.com/tokio-rs/tokio.git" }
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/repos/tokio-rs/tokio/releases/tags/v2.0.0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "name": "Tokio 2.0.0",
                "tag_name": "v2.0.0",
                "body": "### Breaking\n- `spawn` now returns a handle."
            })))
            .mount(&server)
            .await;

        let notes = fetcher(&server)
            .fetch(Ecosystem::Cargo, "tokio", "2.0.0")
            .await
            .expect("release notes");

        assert!(notes.contains("Tokio 2.0.0"));
        assert!(notes.contains("`spawn` now returns a handle."));
    }

    #[tokio::test]
    async fn resolves_npm_pypi_composer_and_go_repositories() {
        let server = MockServer::start().await;
        let release = serde_json::json!({
            "name": "Release",
            "tag_name": "v2.0.0",
            "body": "Notes"
        });

        Mock::given(method("GET"))
            .and(path("/npm/@scope%2Fpkg/latest"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "repository": { "type": "git", "url": "git+https://github.com/acme/pkg.git" }
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/pypi/requests/json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "info": {
                    "project_urls": { "Source": "https://github.com/psf/requests" }
                }
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/p2/acme/lib.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "packages": {
                    "acme/lib": [
                        { "version": "v2.0.0", "source": { "url": "https://github.com/acme/lib.git" } }
                    ]
                }
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/repos/acme/pkg/releases/tags/v2.0.0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(release.clone()))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/repos/psf/requests/releases/tags/v2.0.0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(release.clone()))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/repos/acme/lib/releases/tags/v2.0.0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(release.clone()))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/repos/gin-gonic/gin/releases/tags/v2.0.0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(release))
            .mount(&server)
            .await;

        let fetcher = fetcher(&server);
        assert_eq!(
            fetcher.fetch(Ecosystem::Npm, "@scope/pkg", "2.0.0").await,
            Some("Release\n\nNotes".to_string())
        );
        assert_eq!(
            fetcher.fetch(Ecosystem::Pypi, "requests", "2.0.0").await,
            Some("Release\n\nNotes".to_string())
        );
        assert_eq!(
            fetcher
                .fetch(Ecosystem::Composer, "acme/lib", "2.0.0")
                .await,
            Some("Release\n\nNotes".to_string())
        );
        assert_eq!(
            fetcher
                .fetch(Ecosystem::Go, "github.com/gin-gonic/gin", "2.0.0")
                .await,
            Some("Release\n\nNotes".to_string())
        );
    }

    #[tokio::test]
    async fn missing_notes_degrade_to_none() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/crates/serde"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "crate": { "repository": "https://gitlab.com/serde-rs/serde" }
            })))
            .mount(&server)
            .await;

        assert!(fetcher(&server)
            .fetch(Ecosystem::Cargo, "serde", "2.0.0")
            .await
            .is_none());

        // A non-GitHub repository must not trigger a GitHub request.
        assert!(server
            .received_requests()
            .await
            .expect("requests")
            .iter()
            .all(|request| !request.url.path().starts_with("/repos/")));
    }

    #[tokio::test]
    async fn a_disabled_fetcher_never_requests() {
        let server = MockServer::start().await;
        assert!(ChangelogFetcher::disabled()
            .fetch(Ecosystem::Cargo, "tokio", "2.0.0")
            .await
            .is_none());
        assert!(server
            .received_requests()
            .await
            .expect("requests")
            .is_empty());
    }

    #[test]
    fn matches_workspace_prefixed_release_tags() {
        let candidates = tag_candidates("1.40.0");
        assert!(tag_matches("1.40.0", &candidates));
        assert!(tag_matches("v1.40.0", &candidates));
        assert!(tag_matches("tokio-1.40.0", &candidates));
        assert!(tag_matches("crates/tokio/1.40.0", &candidates));
        assert!(!tag_matches("tokio-1.40.1", &candidates));
        assert!(!tag_matches("other-1.40.0-rc.1", &candidates));
    }

    #[tokio::test]
    async fn falls_back_to_the_release_list_for_prefixed_tags() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/crates/tokio"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "crate": { "repository": "https://github.com/tokio-rs/tokio" }
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/repos/tokio-rs/tokio/releases"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "name": "Tokio 2.0.0",
                    "tag_name": "tokio-2.0.0",
                    "body": "### Breaking\n- `spawn` returns a handle."
                }
            ])))
            .mount(&server)
            .await;

        let notes = fetcher(&server)
            .fetch(Ecosystem::Cargo, "tokio", "2.0.0")
            .await
            .expect("release notes");

        assert!(notes.contains("Tokio 2.0.0"));
        assert!(notes.contains("`spawn` returns a handle."));
    }
}
