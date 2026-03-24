use std::{io, net::SocketAddr, sync::Arc};

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, get_service},
    Router,
};
use tokio::{net::TcpListener, sync::RwLock};
use tower_http::services::{ServeDir, ServeFile};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod content_loader;
mod hot_reload;
mod markdown;
mod models;
mod page_meta;
mod state;

use content_loader::load_content;
use hot_reload::{start_content_watcher, ws_handler};
use markdown::render_markdown_to_html;
use models::{Post, SiteConfig};
use page_meta::{
    PageMeta, PostMetaInput, build_post_meta, default_not_found_meta, escape_html, page_url,
};
use state::{AppState, RouterState};

// Load the hot reload script content at compile time
const HOT_RELOAD_SCRIPT: &str = include_str!("hot_reload.js");
const HOT_RELOAD_TAG_START: &str = "<script>";
const HOT_RELOAD_TAG_END: &str = "</script>";

fn render_post_list(posts: &[Post]) -> String {
    // Single pass: collect ordered groups preserving first-seen role order.
    let mut groups: Vec<(Option<&str>, Vec<&Post>)> = Vec::new();
    for post in posts {
        let role = post.role.as_deref();
        if let Some(group) = groups.iter_mut().find(|(r, _)| *r == role) {
            group.1.push(post);
        } else {
            groups.push((role, vec![post]));
        }
    }

    let mut list_items = String::new();
    for (role, group_posts) in groups {
        if let Some(r) = role {
            let header = r
                .split('-')
                .map(|w| {
                    let mut c = w.chars();
                    match c.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + c.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            list_items.push_str(&format!(
                "<li class=\"sidebar-group-header\">{}</li>",
                escape_html(&header)
            ));
        }
        for post in group_posts {
            let subtitle_html = post
                .subtitle
                .as_deref()
                .map(|s| {
                    format!(
                        "<span class=\"sidebar-post-subtitle\">{}</span>",
                        escape_html(s)
                    )
                })
                .unwrap_or_default();
            list_items.push_str(&format!(
                "<li><a href=\"/posts/{}\" class=\"sidebar-post-link\"><span class=\"sidebar-post-title\">{}</span>{}</a></li>",
                post.slug, escape_html(&post.title), subtitle_html
            ));
        }
    }
    list_items
}

fn render_with_layout(
    layout: &str,
    banner: &str,
    content: &str,
    posts: &[Post],
    meta: &PageMeta,
    is_development: bool,
) -> String {
    let list_items = render_post_list(posts);
    let escaped_title = escape_html(&meta.title);
    let escaped_description = escape_html(&meta.description);
    let escaped_url = escape_html(&meta.url);
    let escaped_image = escape_html(&meta.image);
    let escaped_author = escape_html(&meta.author);

    let role_meta = meta
        .role
        .as_deref()
        .map(|r| {
            format!(
                "<meta property=\"article:section\" content=\"{}\" />",
                escape_html(r)
            )
        })
        .unwrap_or_default();
    let published_time_meta = meta
        .published_time
        .as_deref()
        .map(|published_time| {
            format!(
                "<meta property=\"article:published_time\" content=\"{}\" />",
                escape_html(published_time)
            )
        })
        .unwrap_or_default();

    let mut page = layout
        .replace("{{ banner }}", banner)
        .replace("{{ posts }}", &list_items)
        .replace("{{ page_title }}", &escaped_title)
        .replace("{{ page_description }}", &escaped_description)
        .replace("{{ page_url }}", &escaped_url)
        .replace("{{ page_image }}", &escaped_image)
        .replace("{{ page_author }}", &escaped_author)
        .replace("{{ page_published_time_meta }}", &published_time_meta)
        .replace("{{ page_role_meta }}", &role_meta)
        .replace("{{ content }}", content);

    if is_development {
        page = inject_hot_reload_script(page);
    }

    page
}

fn inject_hot_reload_script(page: String) -> String {
    if page.contains(HOT_RELOAD_SCRIPT) {
        return page;
    }

    let script_tag = format!("{HOT_RELOAD_TAG_START}{HOT_RELOAD_SCRIPT}{HOT_RELOAD_TAG_END}");
    if let Some((head, tail)) = page.rsplit_once("</body>") {
        format!("{head}{script_tag}</body>{tail}")
    } else {
        format!("{page}{script_tag}")
    }
}

async fn homepage(State(state): State<Arc<AppState>>) -> Html<String> {
    let home = state.home_post.read().await.clone();
    let page = render_markdown_page(&state, &home, "/", false).await;
    Html(page)
}

async fn get_current_path(State(state): State<Arc<AppState>>) -> Response {
    if !state.is_development {
        return StatusCode::NOT_FOUND.into_response();
    }

    let path = state.current_browser_path.read().await.clone();
    (StatusCode::OK, path).into_response()
}

async fn set_current_path(State(state): State<Arc<AppState>>, body: Bytes) -> StatusCode {
    if !state.is_development {
        return StatusCode::NOT_FOUND;
    }

    let Ok(raw) = std::str::from_utf8(&body) else {
        return StatusCode::BAD_REQUEST;
    };
    let Some(path) = normalize_browser_path(raw) else {
        return StatusCode::BAD_REQUEST;
    };

    {
        let mut current_path = state.current_browser_path.write().await;
        *current_path = path.clone();
    }
    info!("current browser url: {}", page_url(&path));
    StatusCode::NO_CONTENT
}

async fn render_post(Path(slug): Path<String>, State(state): State<Arc<AppState>>) -> Response {
    if !is_valid_post_slug(&slug) {
        return render_not_found_response(&state, &slug).await;
    }

    let maybe_post = {
        let posts = state.posts.read().await;
        posts.iter().find(|post| post.slug == slug).cloned()
    };
    let post = match maybe_post {
        Some(post) => post,
        None => return render_not_found_response(&state, &slug).await,
    };

    let page = render_markdown_page(&state, &post, &format!("/posts/{}", post.slug), true).await;
    Html(page).into_response()
}

async fn render_markdown_page(
    state: &Arc<AppState>,
    post: &Post,
    page_path: &str,
    include_post_header: bool,
) -> String {
    let html_out = render_markdown_to_html(&post.markdown_body);
    let body = if include_post_header {
        let role_span = post
            .role
            .as_deref()
            .map(|r| format!("<span class=\"post-role\">{}</span>", escape_html(r)))
            .unwrap_or_default();
        let subtitle_span = post
            .subtitle
            .as_deref()
            .map(|s| {
                format!(
                    "<span class=\"post-eyebrow-subtitle\">{}</span>",
                    escape_html(s)
                )
            })
            .unwrap_or_default();
        let eyebrow_html = if role_span.is_empty() && subtitle_span.is_empty() {
            String::new()
        } else {
            format!("<div class=\"post-eyebrow\">{role_span}{subtitle_span}</div>")
        };
        format!(
            "<header class=\"post-header\">{eyebrow_html}<h1>{title}</h1><p class=\"post-date\">{date}</p></header>{content}",
            title = escape_html(&post.title),
            date = escape_html(&post.date),
            content = html_out,
        )
    } else {
        html_out
    };

    let site_config = state.site_config.read().await;
    let meta = build_post_meta(
        page_path,
        &site_config,
        PostMetaInput {
            title: Some(&post.title),
            date: Some(&post.date),
            subtitle: post.subtitle.as_deref(),
            role: post.role.as_deref(),
            image: post.image.as_deref(),
            markdown_body: &post.markdown_body,
        },
    );

    let layout = state.layout_html.read().await;
    let banner = state.banner_html.read().await;
    let posts = state.posts.read().await;

    render_with_layout(&layout, &banner, &body, &posts, &meta, state.is_development)
}

fn is_valid_post_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn normalize_browser_path(path: &str) -> Option<String> {
    let path = path.trim();
    if path.is_empty() || !path.starts_with('/') || path.starts_with("//") {
        return None;
    }

    let path = path.split(['?', '#']).next().unwrap_or(path);

    if path.len() > 1 {
        Some(path.trim_end_matches('/').to_string())
    } else {
        Some("/".to_string())
    }
}

async fn render_not_found_response(state: &Arc<AppState>, slug: &str) -> Response {
    let not_found_markdown = state.not_found_markdown.read().await;
    let body = render_markdown_to_html(&not_found_markdown.replace("{{slug}}", slug));

    let site_config = state.site_config.read().await;
    let layout = state.layout_html.read().await;
    let banner = state.banner_html.read().await;
    let posts = state.posts.read().await;
    let meta = default_not_found_meta(slug, &site_config);
    let page = render_with_layout(&layout, &banner, &body, &posts, &meta, state.is_development);

    (StatusCode::NOT_FOUND, Html(page)).into_response()
}

fn setup_logging() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

async fn initialize_state() -> RouterState {
    let is_development = std::env::var("RUST_ENV")
        .map(|v| v == "development")
        .unwrap_or(false);

    info!("RUST_ENV is set to development: {}", is_development);

    let (site_config, banner_html, layout_html, home_post, not_found_markdown, posts) =
        match load_content().await {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to load initial content files: {}", e);
                (
                    SiteConfig::default(),
                    String::new(),
                    "<!doctype html><html><body>{{ content }}</body></html>".to_string(),
                    Post {
                        title: "Home".to_string(),
                        slug: "home".to_string(),
                        date: String::new(),
                        description: None,
                        image: None,
                        role: None,
                        subtitle: None,
                        markdown_body: "Content loading failed during startup.".to_string(),
                    },
                    "# Post not found\n\nNo post exists for slug `{{slug}}`.".to_string(),
                    Vec::new(),
                )
            }
        };

    let state = Arc::new(AppState {
        site_config: RwLock::new(site_config),
        banner_html: RwLock::new(banner_html),
        layout_html: RwLock::new(layout_html),
        home_post: RwLock::new(home_post),
        current_browser_path: RwLock::new("/".to_string()),
        not_found_markdown: RwLock::new(not_found_markdown),
        posts: RwLock::new(posts),
        is_development,
    });

    let (tx, _rx) = tokio::sync::broadcast::channel(1);

    if is_development {
        info!("Hot reload enabled. Check logs for file change events.");
        start_content_watcher(tx.clone(), state.clone());
    }

    state::RouterState {
        app_state: state,
        broadcaster: tx,
    }
}

fn setup_router(router_state: RouterState) -> Router {
    let static_dir = get_service(ServeDir::new("content/static"));
    let favicon_ico = get_service(ServeFile::new("content/static/favicon.ico"));
    let favicon_png = get_service(ServeFile::new("content/static/favicon.png"));

    Router::new()
        .route("/", get(homepage))
        .route("/posts/{slug}", get(render_post))
        .route("/__dev/current-path", get(get_current_path).post(set_current_path))
        .nest_service("/static", static_dir)
        .route_service("/favicon.ico", favicon_ico)
        .route_service("/favicon.png", favicon_png)
        .route("/ws", get(ws_handler))
        .with_state(router_state)
}

#[tokio::main]
async fn main() -> io::Result<()> {
    setup_logging();

    let router_state = initialize_state().await;
    let app = setup_router(router_state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!(%addr, "listening");
    let listener = TcpListener::bind(addr).await.map_err(|error| {
        io::Error::new(error.kind(), format!("failed to bind to {addr}: {error}"))
    })?;

    axum::serve(listener, app)
        .await
        .map_err(|error| io::Error::other(format!("server error while serving axum app: {error}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{is_valid_post_slug, normalize_browser_path, render_post_list, render_with_layout};
    use crate::models::{Post, SiteConfig};
    use crate::page_meta::PageMeta;

    fn test_layout() -> &'static str {
        "<html><head><title>{{ page_title }}</title><meta name=\"description\" content=\"{{ page_description }}\" /><meta name=\"author\" content=\"{{ page_author }}\" /><meta property=\"og:title\" content=\"{{ page_title }}\" /><meta property=\"og:description\" content=\"{{ page_description }}\" /><meta property=\"og:url\" content=\"{{ page_url }}\" /><meta property=\"og:image\" content=\"{{ page_image }}\" />{{ page_published_time_meta }}{{ page_role_meta }}<meta name=\"twitter:title\" content=\"{{ page_title }}\" /><meta name=\"twitter:description\" content=\"{{ page_description }}\" /><meta name=\"twitter:image\" content=\"{{ page_image }}\" /></head><body>{{ banner }}<main>{{ content }}</main><ul>{{ posts }}</ul></body></html>"
    }

    #[test]
    fn content_layout_contains_new_meta_placeholders() {
        let layout = include_str!("../content/layout.html");
        assert!(layout.contains("{{ page_author }}"));
        assert!(layout.contains("{{ page_published_time_meta }}"));
    }

    #[test]
    fn content_templates_use_site_placeholders() {
        let layout = include_str!("../content/layout.html");
        let banner = include_str!("../content/banner.html");
        assert!(layout.contains("{{ site_og_name }}"));
        assert!(banner.contains("{{ site_title }}"));
    }

    fn test_posts() -> Vec<Post> {
        vec![Post {
            title: "First post".to_string(),
            slug: "first-post".to_string(),
            date: "2026-03-04".to_string(),
            description: None,
            image: None,
            role: None,
            subtitle: None,
            markdown_body: "Body".to_string(),
        }]
    }

    fn test_meta() -> PageMeta {
        PageMeta {
            title: "Test title".to_string(),
            description: "Test description".to_string(),
            url: "https://example.com/posts/test".to_string(),
            image: "https://example.com/static/test.png".to_string(),
            author: "Daniel Vianna".to_string(),
            published_time: Some("2026-03-04T00:00:00Z".to_string()),
            role: Some("mechanism".to_string()),
        }
    }

    #[test]
    fn site_config_defaults_match_content_defaults() {
        let config = SiteConfig::default();
        assert_eq!(config.title, "Bon Élève Blog");
        assert_eq!(config.author, "Daniel Vianna");
    }

    #[test]
    fn injects_hot_reload_script_once_in_development() {
        let page = render_with_layout(
            test_layout(),
            "<header>banner</header>",
            "content",
            &test_posts(),
            &test_meta(),
            true,
        );
        assert_eq!(page.matches("new WebSocket").count(), 1);
        assert_eq!(page.matches("<script>").count(), 1);
    }

    #[test]
    fn injects_script_at_end_when_body_tag_is_missing() {
        let layout = "<html><div>{{ banner }}</div><main>{{ content }}</main></html>";
        let page = render_with_layout(
            layout,
            "banner",
            "content",
            &test_posts(),
            &test_meta(),
            true,
        );
        assert!(page.ends_with("</script>"));
        assert_eq!(page.matches("new WebSocket").count(), 1);
    }

    #[test]
    fn does_not_inject_script_in_non_development() {
        let page = render_with_layout(
            test_layout(),
            "banner",
            "content",
            &test_posts(),
            &test_meta(),
            false,
        );
        assert_eq!(page.matches("new WebSocket").count(), 0);
    }

    #[test]
    fn does_not_replace_posts_placeholder_inside_content() {
        let content = "<p>literal {{ posts }}</p>";
        let page = render_with_layout(
            test_layout(),
            "banner",
            content,
            &test_posts(),
            &test_meta(),
            false,
        );
        assert!(page.contains("<p>literal {{ posts }}</p>"));
    }

    #[test]
    fn renders_page_meta_tags_from_meta_input() {
        let meta = test_meta();
        let page = render_with_layout(
            test_layout(),
            "banner",
            "content",
            &test_posts(),
            &meta,
            false,
        );

        assert!(page.contains("<title>Test title</title>"));
        assert!(page.contains("<meta property=\"og:title\" content=\"Test title\" />"));
        assert!(page.contains("<meta property=\"og:description\" content=\"Test description\" />"));
        assert!(page
            .contains("<meta property=\"og:url\" content=\"https://example.com/posts/test\" />"));
        assert!(page.contains(
            "<meta property=\"og:image\" content=\"https://example.com/static/test.png\" />"
        ));
        assert!(page.contains("<meta name=\"twitter:title\" content=\"Test title\" />"));
        assert!(page.contains("<meta name=\"author\" content=\"Daniel Vianna\" />"));
        assert!(page.contains(
            "<meta property=\"article:published_time\" content=\"2026-03-04T00:00:00Z\" />"
        ));
        assert!(page.contains("<meta property=\"article:section\" content=\"mechanism\" />"));
    }

    #[test]
    fn validates_slug_characters_for_post_paths() {
        assert!(is_valid_post_slug(
            "2026-03-03-optimise-for-the-cheapest-change"
        ));
        assert!(!is_valid_post_slug("../secrets"));
        assert!(!is_valid_post_slug("post/with/slash"));
        assert!(!is_valid_post_slug("UPPERCASE"));
        assert!(!is_valid_post_slug("bad_slug"));
    }

    #[test]
    fn normalize_browser_path_accepts_root_and_post_paths() {
        assert_eq!(normalize_browser_path("/").as_deref(), Some("/"));
        assert_eq!(
            normalize_browser_path("/posts/example-post").as_deref(),
            Some("/posts/example-post")
        );
        assert_eq!(
            normalize_browser_path("/posts/example-post/?x=1#frag").as_deref(),
            Some("/posts/example-post")
        );
    }

    #[test]
    fn normalize_browser_path_rejects_invalid_inputs() {
        assert_eq!(normalize_browser_path("").as_deref(), None);
        assert_eq!(normalize_browser_path("posts/example").as_deref(), None);
        assert_eq!(normalize_browser_path("//example.com").as_deref(), None);
    }

    fn make_post(slug: &str, title: &str, role: Option<&str>, subtitle: Option<&str>) -> Post {
        Post {
            slug: slug.to_string(),
            title: title.to_string(),
            date: "2026-01-01".to_string(),
            role: role.map(ToString::to_string),
            subtitle: subtitle.map(ToString::to_string),
            description: None,
            image: None,
            markdown_body: String::new(),
        }
    }

    #[test]
    fn groups_posts_by_role_in_first_seen_order() {
        let posts = vec![
            make_post("a", "A", Some("mechanism"), None),
            make_post("b", "B", Some("strategy"), None),
            make_post("c", "C", Some("mechanism"), None),
        ];
        let html = render_post_list(&posts);
        let mech_pos = html.find("Mechanism").unwrap();
        let strat_pos = html.find("Strategy").unwrap();
        // Mechanism group header appears before Strategy
        assert!(mech_pos < strat_pos);
        // Post C (second mechanism post) appears after post A
        let a_pos = html.find("/posts/a").unwrap();
        let c_pos = html.find("/posts/c").unwrap();
        assert!(a_pos < c_pos);
        // All posts are present
        assert!(html.contains("/posts/b"));
    }

    #[test]
    fn escapes_html_in_sidebar_titles_and_subtitles() {
        let posts = vec![make_post(
            "xss",
            "<script>alert(1)</script>",
            Some("<role>"),
            Some("<b>bad</b>"),
        )];
        let html = render_post_list(&posts);
        assert!(!html.contains("<script>"));
        assert!(!html.contains("<b>bad</b>"));
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&lt;b&gt;"));
    }

    #[test]
    fn posts_without_role_render_without_group_header() {
        let posts = vec![make_post("no-role", "No Role", None, None)];
        let html = render_post_list(&posts);
        assert!(!html.contains("sidebar-group-header"));
        assert!(html.contains("/posts/no-role"));
    }
}
