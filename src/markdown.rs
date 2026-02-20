use pulldown_cmark::{html, CowStr, Event, Options, Parser};

fn markdown_options() -> Options {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_MATH);
    options
}

pub fn render_markdown_to_html(markdown: &str) -> String {
    let normalized_markdown = normalize_latex_delimiters(markdown);
    let parser = Parser::new_ext(&normalized_markdown, markdown_options()).map(|event| match event {
        Event::InlineMath(math) => Event::Html(CowStr::Boxed(render_math_html(&math, false).into_boxed_str())),
        Event::DisplayMath(math) => Event::Html(CowStr::Boxed(render_math_html(&math, true).into_boxed_str())),
        other => other,
    });

    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    html_out
}

fn normalize_latex_delimiters(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut i = 0;

    while i < input.len() {
        if let Some((open, close, display_mode)) = delimiter_at(input, i) {
            let content_start = i + open.len();
            if let Some(close_at) = input[content_start..].find(close) {
                let content_end = content_start + close_at;
                let content = &input[content_start..content_end];
                if display_mode || content.contains('\n') {
                    out.push_str("$$");
                    out.push_str(content);
                    out.push_str("$$");
                } else {
                    out.push('$');
                    out.push_str(content);
                    out.push('$');
                }
                i = content_end + close.len();
                continue;
            }
        }

        if let Some(ch) = input[i..].chars().next() {
            out.push(ch);
            i += ch.len_utf8();
        } else {
            break;
        }
    }

    out
}

fn delimiter_at(input: &str, index: usize) -> Option<(&'static str, &'static str, bool)> {
    let tail = &input[index..];
    if tail.starts_with("\\(") {
        Some(("\\(", "\\)", false))
    } else if tail.starts_with("\\[") {
        Some(("\\[", "\\]", true))
    } else {
        None
    }
}

fn render_math_html(source: &str, display_mode: bool) -> String {
    let mut opts = katex::Opts::builder();
    opts.display_mode(display_mode);

    let rendered = match opts.build() {
        Ok(opts) => katex::render_with_opts(source, opts),
        Err(_) => return fallback_math_html(source, display_mode),
    };

    match rendered {
        Ok(html) => html,
        Err(_) => fallback_math_html(source, display_mode),
    }
}

fn fallback_math_html(source: &str, display_mode: bool) -> String {
    let class_name = if display_mode { "math math-display" } else { "math math-inline" };
    format!("<span class=\"{class_name}\">{source}</span>")
}

#[cfg(test)]
mod tests {
    use super::render_markdown_to_html;

    #[test]
    fn renders_math_with_latex_paren_and_bracket_delimiters() {
        let input = "\\(x^2\\) and \\[y^2\\]";
        let output = render_markdown_to_html(input);
        assert!(output.contains("katex"));
    }

    #[test]
    fn renders_multiline_paren_delimited_math() {
        let input = "Start \\( \\frac{2.24T}{2.08T}\n\\approx 1.077 \\) end";
        let output = render_markdown_to_html(input);
        assert!(output.contains("katex"));
    }

    #[test]
    fn renders_math_from_ai_feb_2026_post() {
        let post = include_str!("../content/posts/ai-feb-2026.md");
        let output = render_markdown_to_html(post);
        assert!(output.contains("katex"));
    }
}
