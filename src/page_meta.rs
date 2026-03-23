const DEFAULT_SITE_URL: &str = "https://boneleve.blog";
const DEFAULT_SOCIAL_IMAGE_PATH: &str = "/static/favicon.png";
const DEFAULT_PAGE_DESCRIPTION: &str = "Engineering notes on making change cheap.";

pub(crate) struct PageMeta {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) url: String,
    pub(crate) image: String,
    pub(crate) role: Option<String>,
}

pub(crate) fn default_home_meta() -> PageMeta {
    PageMeta {
        title: "Bon Élève Blog".to_string(),
        description: DEFAULT_PAGE_DESCRIPTION.to_string(),
        url: site_url(),
        image: absolute_url(DEFAULT_SOCIAL_IMAGE_PATH),
        role: None,
    }
}

pub(crate) fn default_not_found_meta(slug: &str) -> PageMeta {
    let base = site_url();
    PageMeta {
        title: "Post not found | Bon Élève Blog".to_string(),
        description: format!("The post \"{}\" was not found.", slug),
        url: format!("{base}/posts/{slug}"),
        image: absolute_url(DEFAULT_SOCIAL_IMAGE_PATH),
        role: None,
    }
}

pub(crate) fn build_post_meta(
    slug: &str,
    title: Option<&str>,
    subtitle: Option<&str>,
    role: Option<&str>,
    image: Option<&str>,
    markdown_body: &str,
) -> PageMeta {
    let base = site_url();
    let title = title
        .map(ToString::to_string)
        .unwrap_or_else(|| "Bon Élève Blog".to_string());

    let description = build_social_description(subtitle, markdown_body);
    let description = if description.is_empty() {
        DEFAULT_PAGE_DESCRIPTION.to_string()
    } else {
        description
    };

    let image_path = image
        .map(ToString::to_string)
        .filter(|i| !i.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_SOCIAL_IMAGE_PATH.to_string());

    PageMeta {
        title,
        description,
        url: format!("{base}/posts/{slug}"),
        image: absolute_url(&image_path),
        role: role.map(ToString::to_string),
    }
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
    let mut parts: Vec<String> = Vec::new();
    let mut in_code_block = false;

    for line in markdown.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block
            || trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with('>')
            || trimmed == "---"
            || trimmed.starts_with("![")
            || (trimmed.starts_with('<') && trimmed.ends_with('>'))
        {
            continue;
        }

        let stripped = strip_inline_for_description(trimmed);
        let stripped = stripped.trim().to_string();
        if !stripped.is_empty() {
            parts.push(stripped);
        }
    }

    let joined = parts.join(" ");
    let mut out = String::with_capacity(joined.len());
    let mut prev_space = false;
    for c in joined.chars() {
        if c.is_whitespace() {
            if !prev_space {
                out.push(' ');
            }
            prev_space = true;
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    out.trim().to_string()
}

/// Strips inline markdown: removes images, replaces links with their
/// text, removes bold/italic/code markers.
fn strip_inline_for_description(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(n);
    let mut i = 0;

    while i < n {
        // Image: ![alt](url) → skip entirely
        if chars[i] == '!' && i + 1 < n && chars[i + 1] == '[' {
            if let Some(cb) = chars[i + 2..].iter().position(|&c| c == ']') {
                let after = i + 2 + cb + 1;
                if after < n && chars[after] == '(' {
                    if let Some(cp) = chars[after + 1..].iter().position(|&c| c == ')') {
                        i = after + 1 + cp + 1;
                        continue;
                    }
                }
            }
        }

        // Link: [text](url) → text
        if chars[i] == '[' {
            if let Some(cb) = chars[i + 1..].iter().position(|&c| c == ']') {
                let after = i + 1 + cb + 1;
                if after < n && chars[after] == '(' {
                    if let Some(cp) = chars[after + 1..].iter().position(|&c| c == ')') {
                        for &c in &chars[i + 1..i + 1 + cb] {
                            out.push(c);
                        }
                        i = after + 1 + cp + 1;
                        continue;
                    }
                }
            }
        }

        // Double markers: ** __
        if i + 1 < n
            && ((chars[i] == '*' && chars[i + 1] == '*')
                || (chars[i] == '_' && chars[i + 1] == '_'))
        {
            i += 2;
            continue;
        }

        // Single markers: * _ `
        if matches!(chars[i], '*' | '_' | '`') {
            i += 1;
            continue;
        }

        out.push(chars[i]);
        i += 1;
    }

    out
}


pub(crate) fn escape_html(input: &str) -> String {
    htmlescape::encode_minimal(input)
}

fn site_url() -> String {
    std::env::var("SITE_URL")
        .ok()
        .map(|u| u.trim_end_matches('/').to_string())
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| DEFAULT_SITE_URL.to_string())
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
    use super::{absolute_url, build_post_meta, build_social_description, escape_html};
    use crate::models::FrontMatter;

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
        assert!(desc.chars().count() <= 160, "length was {}", desc.chars().count());
    }

    #[test]
    fn social_description_truncates_at_word_boundary_with_ellipsis() {
        // Build a string that is definitely over 160 chars
        let subtitle = "Subtitle";
        let body = "word ".repeat(40); // ~200 chars when combined
        let desc = build_social_description(Some(subtitle), &body);
        assert!(desc.chars().count() <= 160, "length was {}", desc.chars().count());
        assert!(desc.ends_with("..."), "should end with ellipsis, got: {desc:?}");
        // Must not cut mid-word
        let without_ellipsis = desc.trim_end_matches("...");
        assert!(!without_ellipsis.ends_with(' '), "should trim trailing space before ellipsis");
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

    // ── build_post_meta ───────────────────────────────────────────────────────

    #[test]
    fn build_post_meta_sets_title_url_image() {
        let fm = front_matter_with(Some("/static/custom.png"));
        let meta = build_post_meta(
            "test-post",
            Some(&fm.title),
            fm.subtitle.as_deref(),
            fm.role.as_deref(),
            fm.image.as_deref(),
            "Body sentence.",
        );
        assert_eq!(meta.title, "Test Post");
        assert!(meta.url.ends_with("/posts/test-post"));
        assert!(meta.image.ends_with("/static/custom.png"));
    }

    #[test]
    fn build_post_meta_description_comes_from_subtitle_and_body() {
        let meta = build_post_meta(
            "test-post",
            Some("Test Post"),
            Some("Punchy subtitle"),
            None,
            None,
            "First sentence. Second sentence.",
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
            Some(&fm.title),
            fm.subtitle.as_deref(),
            fm.role.as_deref(),
            fm.image.as_deref(),
            markdown,
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
            Some(&fm.title),
            fm.subtitle.as_deref(),
            fm.role.as_deref(),
            fm.image.as_deref(),
            "body",
        );
        assert_eq!(meta.role.as_deref(), Some("mechanism"));
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
