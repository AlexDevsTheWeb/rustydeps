//! Safe Markdown-to-HTML rendering for AI analyses.
//!
//! Provider output is untrusted text: raw HTML events are dropped, images are
//! removed, and dangerous link schemes are neutralised before the HTML reaches
//! the report. The result is inserted with Askama's `|safe` filter.

use pulldown_cmark::{html, Event, Options, Parser, Tag, TagEnd};

/// Render GitHub-flavored Markdown (tables and strikethrough enabled) as a
/// safe subset of HTML.
pub fn render(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let events = Parser::new_ext(markdown, options).filter(|event| {
        !matches!(
            event,
            Event::Html(_)
                | Event::InlineHtml(_)
                | Event::Start(Tag::Image { .. })
                | Event::End(TagEnd::Image)
        )
    });

    let mut output = String::new();
    html::push_html(&mut output, events);
    sanitize_hrefs(&output)
}

/// Replace `javascript:`, `data:`, and `vbscript:` link targets with `#`.
fn sanitize_hrefs(html: &str) -> String {
    let mut output = String::with_capacity(html.len());
    let mut rest = html;

    while let Some(position) = rest.find("href=\"") {
        output.push_str(&rest[..position + 6]);
        let after = &rest[position + 6..];

        let Some(end) = after.find('"') else {
            output.push_str(after);
            return output;
        };

        let url = &after[..end];
        let scheme = url.trim_start().to_ascii_lowercase();
        if scheme.starts_with("javascript:")
            || scheme.starts_with("data:")
            || scheme.starts_with("vbscript:")
        {
            output.push('#');
        } else {
            output.push_str(url);
        }
        output.push('"');
        rest = &after[end + 1..];
    }

    output.push_str(rest);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_headings_lists_and_code() {
        let html = render(
            "### 🚨 Breaking Changes & Deprecations\n\n\
             - `tokio::spawn` changed\n\n\
             ```rust\nlet runtime = Runtime::new();\n```",
        );

        assert!(html.contains("<h3>🚨 Breaking Changes &amp; Deprecations</h3>"));
        assert!(html.contains("<li><code>tokio::spawn</code> changed</li>"));
        assert!(html.contains("<pre><code class=\"language-rust\">"));
    }

    #[test]
    fn drops_raw_html_instead_of_passing_it_through() {
        let html = render("<img src=x onerror=alert(1)>\n\n**safe**");

        assert!(!html.contains("<img"));
        assert!(!html.contains("onerror"));
        assert!(html.contains("<strong>safe</strong>"));
    }

    #[test]
    fn neutralises_dangerous_links_and_removes_images() {
        let html = render(
            "[click](javascript:alert(1))\n\n\
             [data](data:text/html,<script>alert(1)</script>)\n\n\
             ![pixel](https://example.test/pixel.png)\n\n\
             [docs](https://example.test/docs)",
        );

        assert!(!html.contains("javascript:"));
        assert!(!html.contains("data:text/html"));
        assert!(!html.contains("<img"));
        assert!(html.contains("href=\"#\""));
        assert!(html.contains("href=\"https://example.test/docs\""));
        assert!(html.contains(">click</a>"));
    }

    #[test]
    fn empty_input_renders_nothing() {
        assert!(render("").trim().is_empty());
    }
}
