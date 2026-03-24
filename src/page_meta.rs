use chrono::{DateTime, NaiveDate, SecondsFormat};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use serde_json::Value;

use crate::models::SiteConfig;

const DEFAULT_SITE_URL: &str = "https://boneleve.blog";
const DEFAULT_SOCIAL_IMAGE_PATH: &str = "/static/favicon.png";

pub(crate) struct PageMeta {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) url: String,
    pub(crate) image: String,
    pub(crate) author: String,
    pub(crate) published_time: Option<String>,
    pub(crate) role: Option<String>,
}

pub(crate) struct PostMetaInput<'a> {
    pub(crate) title: Option<&'a str>,
    pub(crate) date: Option<&'a str>,
    pub(crate) subtitle: Option<&'a str>,
    pub(crate) role: Option<&'a str>,
    pub(crate) image: Option<&'a str>,
    pub(crate) markdown_body: &'a str,
}

pub(crate) fn default_home_meta(site_config: &SiteConfig) -> PageMeta {
    PageMeta {
        title: site_config.title.clone(),
        description: site_config.description.clone(),
        url: site_url(),
        image: absolute_url(DEFAULT_SOCIAL_IMAGE_PATH),
        author: site_config.author.clone(),
        published_time: None,
        role: None,
    }
}

pub(crate) fn default_not_found_meta(slug: &str, site_config: &SiteConfig) -> PageMeta {
    let base = site_url();
    PageMeta {
        title: format!("Post not found | {}", site_config.title),
        description: format!("The post \"{}\" was not found.", slug),
        url: format!("{base}/posts/{slug}"),
        image: absolute_url(DEFAULT_SOCIAL_IMAGE_PATH),
        author: site_config.author.clone(),
        published_time: None,
        role: None,
    }
}

pub(crate) fn build_post_meta(
    slug: &str,
    site_config: &SiteConfig,
    input: PostMetaInput<'_>,
) -> PageMeta {
    let base = site_url();
    let title = input
        .title
        .map(ToString::to_string)
        .unwrap_or_else(|| site_config.title.clone());

    let description = build_social_description(input.subtitle, input.markdown_body);
    let description = if description.is_empty() {
        site_config.description.clone()
    } else {
        description
    };

    let image_path = input
        .image
        .map(ToString::to_string)
        .filter(|i| !i.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_SOCIAL_IMAGE_PATH.to_string());

    PageMeta {
        title,
        description,
        url: format!("{base}/posts/{slug}"),
        image: absolute_url(&image_path),
        author: site_config.author.clone(),
        published_time: input.date.and_then(iso_published_time),
        role: input.role.map(ToString::to_string),
    }
}

fn iso_published_time(date: &str) -> Option<String> {
    let date = date.trim();
    if date.is_empty() {
        return None;
    }

    if let Ok(datetime) = DateTime::parse_from_rfc3339(date) {
        return Some(datetime.to_rfc3339_opts(SecondsFormat::Secs, true));
    }

    if let Ok(calendar_date) = NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        return calendar_date.and_hms_opt(0, 0, 0).map(|datetime| {
            datetime
                .and_utc()
                .to_rfc3339_opts(SecondsFormat::Secs, true)
        });
    }

    None
}

/// Generates a social-card description (≤160 chars):
/// - begins with subtitle, ensuring it ends with '.'
/// - appends normalised body text
/// - if combined text exceeds 160 chars, truncates at the last word
///   boundary before char 157 and appends "..."
pub(crate) fn build_social_description(subtitle: Option<&str>, markdown_body: &str) -> String {
    let prefix = match subtitle {
        Some(s) if !s.trim().is_empty() => {
            let s = s.trim();
            if matches!(s.chars().last(), Some('.' | '!' | '?')) {
                s.to_string()
            } else {
                format!("{}.", s)
            }
        }
        _ => String::new(),
    };

    let body_text = normalize_body_for_description(markdown_body);

    let combined = if prefix.is_empty() {
        body_text
    } else if body_text.is_empty() {
        prefix
    } else {
        format!("{} {}", prefix, body_text)
    };

    if combined.chars().count() <= 160 {
        return combined;
    }

    // Find the byte offset of char 157 (leaves room for "...")
    let cut = combined
        .char_indices()
        .nth(157)
        .map(|(i, _)| i)
        .unwrap_or(combined.len());

    // Step back to the last word boundary within the allowed slice
    let last_space = combined[..cut].rfind(' ').unwrap_or(cut);
    format!("{}...", combined[..last_space].trim_end())
}

/// Strips markdown block-level syntax, flattens lines to a single
/// normalised string suitable for sentence extraction.
fn normalize_body_for_description(markdown: &str) -> String {
    let mut out = String::new();
    let mut heading_depth = 0usize;
    let mut code_block_depth = 0usize;
    let mut image_depth = 0usize;

    for event in Parser::new(markdown) {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { .. } => heading_depth += 1,
                Tag::CodeBlock(_) => code_block_depth += 1,
                Tag::Image { .. } => image_depth += 1,
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Paragraph | TagEnd::Item | TagEnd::BlockQuote(_) | TagEnd::TableCell => {
                    append_normalized_fragment(&mut out, " ");
                }
                TagEnd::Heading(_) => heading_depth = heading_depth.saturating_sub(1),
                TagEnd::CodeBlock => code_block_depth = code_block_depth.saturating_sub(1),
                TagEnd::Image => image_depth = image_depth.saturating_sub(1),
                _ => {}
            },
            Event::Text(text) | Event::Code(text)
                if heading_depth == 0 && code_block_depth == 0 && image_depth == 0 =>
            {
                append_normalized_fragment(&mut out, &text);
            }
            Event::SoftBreak | Event::HardBreak
                if heading_depth == 0 && code_block_depth == 0 && image_depth == 0 =>
            {
                append_normalized_fragment(&mut out, " ");
            }
            _ => {}
        }
    }

    out.trim().to_string()
}

fn append_normalized_fragment(out: &mut String, fragment: &str) {
    let mut prev_space = out.ends_with(' ');

    for c in fragment.chars() {
        if c.is_whitespace() {
            if !prev_space && !out.is_empty() {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(c);
            prev_space = false;
        }
    }
}

pub(crate) fn escape_html(input: &str) -> String {
    htmlescape::encode_minimal(input)
}

fn site_url() -> String {
    if let Some(url) = devloop_site_url() {
        return url;
    }

    if let Some(url) = development_site_url() {
        return url;
    }

    std::env::var("SITE_URL")
        .ok()
        .map(|u| u.trim_end_matches('/').to_string())
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| DEFAULT_SITE_URL.to_string())
}

fn development_site_url() -> Option<String> {
    if std::env::var("RUST_ENV").ok().as_deref() != Some("development") {
        return None;
    }
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    Some(format!("http://127.0.0.1:{port}"))
}

fn devloop_site_url() -> Option<String> {
    let state_path = std::env::var("DEVLOOP_STATE").ok()?;
    let raw = std::fs::read_to_string(state_path).ok()?;
    let json: Value = serde_json::from_str(&raw).ok()?;
    json.get("tunnel_url")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.trim_end_matches('/').to_string())
}

fn absolute_url(path_or_url: &str) -> String {
    if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
        return path_or_url.to_string();
    }
    let base = site_url();
    if path_or_url.starts_with('/') {
        format!("{base}{path_or_url}")
    } else {
        format!("{base}/{path_or_url}")
    }
}

#[cfg(test)]
mod tests {
    use super::{
        absolute_url, build_post_meta, build_social_description, default_home_meta, escape_html,
        iso_published_time, site_url, PostMetaInput,
    };
    use crate::models::{FrontMatter, SiteConfig};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn front_matter_with(image: Option<&str>) -> FrontMatter {
        FrontMatter {
            title: "Test Post".to_string(),
            date: "2026-03-04".to_string(),
            slug: "test-post".to_string(),
            description: None,
            image: image.map(ToString::to_string),
            role: None,
            subtitle: None,
        }
    }

    fn test_site_config() -> SiteConfig {
        SiteConfig {
            title: "Configured Blog".to_string(),
            author: "Configured Author".to_string(),
            description: "Configured description.".to_string(),
            og_site_name: "Configured OG Name".to_string(),
        }
    }

    fn unique_temp_path(name: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!("{name}-{unique}.json"))
    }

    // ── build_social_description ──────────────────────────────────────────────

    #[test]
    fn social_description_starts_with_subtitle_dot() {
        let desc = build_social_description(Some("A punchy hook"), "Body sentence.");
        assert!(desc.starts_with("A punchy hook."));
    }

    #[test]
    fn social_description_keeps_existing_terminal_punctuation_on_subtitle() {
        let desc = build_social_description(Some("Already ends."), "Body.");
        assert!(desc.starts_with("Already ends."));
        assert!(!desc.starts_with("Already ends.."));
    }

    #[test]
    fn social_description_appends_body_sentences_in_order() {
        let body = "First sentence. Second sentence. Third sentence.";
        let desc = build_social_description(Some("Hook"), body);
        let first = desc.find("First sentence.").unwrap();
        let second = desc.find("Second sentence.").unwrap();
        assert!(first < second);
    }

    #[test]
    fn social_description_never_exceeds_160_chars() {
        let subtitle = "Short subtitle";
        let body = "A moderately long sentence that contributes useful text here. \
                    A sentence that is far too long to fit within the remaining budget after the subtitle and first sentence have been included in full. \
                    Short one.";
        let desc = build_social_description(Some(subtitle), body);
        assert!(
            desc.chars().count() <= 160,
            "length was {}",
            desc.chars().count()
        );
    }

    #[test]
    fn social_description_truncates_at_word_boundary_with_ellipsis() {
        // Build a string that is definitely over 160 chars
        let subtitle = "Subtitle";
        let body = "word ".repeat(40); // ~200 chars when combined
        let desc = build_social_description(Some(subtitle), &body);
        assert!(
            desc.chars().count() <= 160,
            "length was {}",
            desc.chars().count()
        );
        assert!(
            desc.ends_with("..."),
            "should end with ellipsis, got: {desc:?}"
        );
        // Must not cut mid-word
        let without_ellipsis = desc.trim_end_matches("...");
        assert!(
            !without_ellipsis.ends_with(' '),
            "should trim trailing space before ellipsis"
        );
    }

    #[test]
    fn social_description_includes_trailing_fragment() {
        // Body ends mid-sentence — truncation still includes it (up to char limit)
        let body = "Complete sentence. Incomplete fragment with no stop";
        let desc = build_social_description(Some("Subtitle"), body);
        assert!(desc.contains("Incomplete fragment"));
    }

    #[test]
    fn social_description_strips_markdown_inline() {
        let body = "This has **bold** and `code` and [a link](http://example.com).";
        let desc = build_social_description(Some("Subtitle"), body);
        assert!(!desc.contains("**"));
        assert!(!desc.contains('`'));
        assert!(!desc.contains("[a link]"));
        assert!(desc.contains("a link"), "link text should be preserved");
    }

    #[test]
    fn social_description_without_subtitle_uses_body_sentences() {
        let body = "First sentence. Second one.";
        let desc = build_social_description(None, body);
        assert!(desc.starts_with("First sentence."));
    }

    #[test]
    fn social_description_skips_headings_and_code_blocks() {
        let body = "# Heading\n\n```\ncode here\n```\n\nReal sentence.";
        let desc = build_social_description(Some("Subtitle"), body);
        assert!(!desc.contains("Heading"));
        assert!(!desc.contains("code here"));
        assert!(desc.contains("Real sentence."));
    }

    #[test]
    fn social_description_keeps_angle_bracket_text() {
        let body = "<5% of failures were random.>";
        let desc = build_social_description(None, body);
        assert_eq!(desc, "<5% of failures were random.>");
    }

    // ── build_post_meta ───────────────────────────────────────────────────────

    #[test]
    fn build_post_meta_sets_title_url_image() {
        let fm = front_matter_with(Some("/static/custom.png"));
        let meta = build_post_meta(
            "test-post",
            &test_site_config(),
            PostMetaInput {
                title: Some(&fm.title),
                date: Some(&fm.date),
                subtitle: fm.subtitle.as_deref(),
                role: fm.role.as_deref(),
                image: fm.image.as_deref(),
                markdown_body: "Body sentence.",
            },
        );
        assert_eq!(meta.title, "Test Post");
        assert!(meta.url.ends_with("/posts/test-post"));
        assert!(meta.image.ends_with("/static/custom.png"));
        assert_eq!(meta.author, "Configured Author");
        assert_eq!(meta.published_time.as_deref(), Some("2026-03-04T00:00:00Z"));
    }

    #[test]
    fn build_post_meta_description_comes_from_subtitle_and_body() {
        let meta = build_post_meta(
            "test-post",
            &test_site_config(),
            PostMetaInput {
                title: Some("Test Post"),
                date: Some("2026-03-04"),
                subtitle: Some("Punchy subtitle"),
                role: None,
                image: None,
                markdown_body: "First sentence. Second sentence.",
            },
        );
        assert!(meta.description.starts_with("Punchy subtitle."));
        assert!(meta.description.contains("First sentence."));
    }

    #[test]
    fn build_post_meta_falls_back_to_body_sentences_when_no_subtitle() {
        let fm = front_matter_with(None);
        let markdown = "First line.\n\nSecond line.";
        let meta = build_post_meta(
            "test-post",
            &test_site_config(),
            PostMetaInput {
                title: Some(&fm.title),
                date: Some(&fm.date),
                subtitle: fm.subtitle.as_deref(),
                role: fm.role.as_deref(),
                image: fm.image.as_deref(),
                markdown_body: markdown,
            },
        );
        assert_eq!(meta.description, "First line. Second line.");
        assert!(meta.image.ends_with("/static/favicon.png"));
    }

    #[test]
    fn build_post_meta_sets_role_on_meta() {
        let mut fm = front_matter_with(None);
        fm.role = Some("mechanism".to_string());
        let meta = build_post_meta(
            "test-post",
            &test_site_config(),
            PostMetaInput {
                title: Some(&fm.title),
                date: Some(&fm.date),
                subtitle: fm.subtitle.as_deref(),
                role: fm.role.as_deref(),
                image: fm.image.as_deref(),
                markdown_body: "body",
            },
        );
        assert_eq!(meta.role.as_deref(), Some("mechanism"));
    }

    #[test]
    fn default_home_meta_uses_site_config() {
        let meta = default_home_meta(&test_site_config());
        assert_eq!(meta.title, "Configured Blog");
        assert_eq!(meta.author, "Configured Author");
        assert_eq!(meta.description, "Configured description.");
    }

    #[test]
    fn site_url_prefers_devloop_state_when_present() {
        let path = unique_temp_path("devloop-state");
        std::fs::write(
            &path,
            r#"{"tunnel_url":"https://preview.example.trycloudflare.com"}"#,
        )
        .expect("write state");
        std::env::set_var("DEVLOOP_STATE", &path);
        std::env::remove_var("SITE_URL");

        assert_eq!(site_url(), "https://preview.example.trycloudflare.com");

        std::env::remove_var("DEVLOOP_STATE");
        std::fs::remove_file(path).expect("cleanup state file");
    }

    #[test]
    fn site_url_falls_back_to_localhost_in_development() {
        std::env::remove_var("DEVLOOP_STATE");
        std::env::remove_var("SITE_URL");
        std::env::set_var("RUST_ENV", "development");
        std::env::set_var("PORT", "18080");

        assert_eq!(site_url(), "http://127.0.0.1:18080");

        std::env::remove_var("RUST_ENV");
        std::env::remove_var("PORT");
    }

    #[test]
    fn iso_published_time_expands_date_only_values() {
        assert_eq!(
            iso_published_time("2026-03-04").as_deref(),
            Some("2026-03-04T00:00:00Z")
        );
    }

    #[test]
    fn iso_published_time_keeps_existing_datetimes() {
        assert_eq!(
            iso_published_time("2026-03-04T09:30:00Z").as_deref(),
            Some("2026-03-04T09:30:00Z")
        );
    }

    #[test]
    fn iso_published_time_rejects_non_iso_datetimes() {
        assert_eq!(iso_published_time("2026-03-04T9am").as_deref(), None);
    }

    // ── other helpers ─────────────────────────────────────────────────────────

    #[test]
    fn absolute_url_keeps_absolute_inputs() {
        assert_eq!(
            absolute_url("https://example.com/image.png"),
            "https://example.com/image.png"
        );
        assert_eq!(
            absolute_url("http://example.com/image.png"),
            "http://example.com/image.png"
        );
    }

    #[test]
    fn absolute_url_expands_relative_inputs() {
        let root_relative = absolute_url("/static/favicon.png");
        assert!(root_relative.starts_with("https://") || root_relative.starts_with("http://"));
        assert!(root_relative.ends_with("/static/favicon.png"));

        let plain_relative = absolute_url("static/favicon.png");
        assert!(plain_relative.starts_with("https://") || plain_relative.starts_with("http://"));
        assert!(plain_relative.ends_with("/static/favicon.png"));
    }

    #[test]
    fn escape_html_escapes_meta_sensitive_characters() {
        let escaped = escape_html(r#""A&B" <tag> 'q'"#);
        assert_eq!(escaped, "&quot;A&amp;B&quot; &lt;tag&gt; &#x27;q&#x27;");
    }
}
