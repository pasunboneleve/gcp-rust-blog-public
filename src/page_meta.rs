const DEFAULT_SITE_URL: &str = "https://boneleve.blog";
const DEFAULT_SOCIAL_IMAGE_PATH: &str = "/static/favicon.png";
const DEFAULT_PAGE_DESCRIPTION: &str = "Engineering notes on making change cheap.";

pub(crate) struct PageMeta {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) url: String,
    pub(crate) image: String,
}

pub(crate) fn default_home_meta() -> PageMeta {
    PageMeta {
        title: "Bon Élève Blog".to_string(),
        description: DEFAULT_PAGE_DESCRIPTION.to_string(),
        url: site_url(),
        image: absolute_url(DEFAULT_SOCIAL_IMAGE_PATH),
    }
}

pub(crate) fn default_not_found_meta(slug: &str) -> PageMeta {
    let base = site_url();
    PageMeta {
        title: "Post not found | Bon Élève Blog".to_string(),
        description: format!("The post \"{}\" was not found.", slug),
        url: format!("{base}/posts/{slug}"),
        image: absolute_url(DEFAULT_SOCIAL_IMAGE_PATH),
    }
}

pub(crate) fn build_post_meta(
    slug: &str,
    title: Option<&str>,
    description: Option<&str>,
    image: Option<&str>,
    markdown_body: &str,
) -> PageMeta {
    let base = site_url();
    let title = title
        .map(ToString::to_string)
        .unwrap_or_else(|| "Bon Élève Blog".to_string());
    let description = description
        .map(ToString::to_string)
        .filter(|d| !d.trim().is_empty())
        .unwrap_or_else(|| extract_markdown_excerpt(markdown_body, 3, 220));
    let image_path = image
        .map(ToString::to_string)
        .filter(|i| !i.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_SOCIAL_IMAGE_PATH.to_string());

    PageMeta {
        title,
        description: if description.is_empty() {
            DEFAULT_PAGE_DESCRIPTION.to_string()
        } else {
            description
        },
        url: format!("{base}/posts/{slug}"),
        image: absolute_url(&image_path),
    }
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

fn strip_markdown_inline(text: &str) -> String {
    text.replace("**", "")
        .replace("__", "")
        .replace(['*', '`', '_'], "")
}

fn is_markup_line(trimmed: &str) -> bool {
    trimmed.starts_with('#')
        || trimmed.starts_with("```")
        || trimmed.starts_with("![")
        || trimmed.starts_with('>')
        || (trimmed.starts_with('<') && trimmed.ends_with('>'))
}

fn extract_markdown_excerpt(markdown: &str, max_lines: usize, max_chars: usize) -> String {
    let mut lines: Vec<String> = Vec::new();

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || is_markup_line(trimmed) {
            continue;
        }

        lines.push(strip_markdown_inline(trimmed));
        if lines.len() >= max_lines {
            break;
        }
    }

    let mut out = lines.join(" ");
    if out.len() > max_chars {
        out.truncate(max_chars);
        if let Some(last_space) = out.rfind(' ') {
            out.truncate(last_space);
        }
        out.push_str("...");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{absolute_url, build_post_meta, escape_html, extract_markdown_excerpt};
    use crate::models::FrontMatter;

    fn front_matter_with(description: Option<&str>, image: Option<&str>) -> FrontMatter {
        FrontMatter {
            title: "Test Post".to_string(),
            date: "2026-03-04".to_string(),
            slug: "test-post".to_string(),
            description: description.map(ToString::to_string),
            image: image.map(ToString::to_string),
        }
    }

    #[test]
    fn excerpt_uses_first_non_empty_non_markup_lines() {
        let markdown = r#"
# Heading

First useful line.

<div>ignored</div>
Second useful line with **bold** and `code`.
![alt](img.png)
Third useful line.
Fourth useful line.
"#;

        let excerpt = extract_markdown_excerpt(markdown, 3, 220);
        assert_eq!(
            excerpt,
            "First useful line. Second useful line with bold and code. Third useful line."
        );
    }

    #[test]
    fn excerpt_truncates_at_word_boundary() {
        let markdown = "A long line with many words that should be truncated cleanly.";
        let excerpt = extract_markdown_excerpt(markdown, 1, 24);

        assert_eq!(excerpt, "A long line with many...");
    }

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
    fn build_post_meta_prefers_front_matter_description_and_image() {
        let fm = front_matter_with(Some("From front matter"), Some("/static/custom.png"));
        let meta = build_post_meta(
            "test-post",
            Some(&fm.title),
            fm.description.as_deref(),
            fm.image.as_deref(),
            "Body fallback should not be used",
        );

        assert_eq!(meta.title, "Test Post");
        assert_eq!(meta.description, "From front matter");
        assert!(meta.url.ends_with("/posts/test-post"));
        assert!(meta.image.ends_with("/static/custom.png"));
    }

    #[test]
    fn build_post_meta_uses_markdown_excerpt_and_favicon_fallback() {
        let fm = front_matter_with(None, None);
        let markdown = "First line.\n\nSecond line.";
        let meta = build_post_meta(
            "test-post",
            Some(&fm.title),
            fm.description.as_deref(),
            fm.image.as_deref(),
            markdown,
        );

        assert_eq!(meta.description, "First line. Second line.");
        assert!(meta.image.ends_with("/static/favicon.png"));
    }

    #[test]
    fn escape_html_escapes_meta_sensitive_characters() {
        let escaped = escape_html(r#""A&B" <tag> 'q'"#);
        assert_eq!(escaped, "&quot;A&amp;B&quot; &lt;tag&gt; &#x27;q&#x27;");
    }

    #[test]
    fn excerpt_keeps_non_tag_line_starting_with_angle_bracket() {
        let markdown = "<5% of failures were random.\nSecond line.";
        let excerpt = extract_markdown_excerpt(markdown, 2, 220);
        assert_eq!(excerpt, "<5% of failures were random. Second line.");
    }
}
