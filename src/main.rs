use std::{net::SocketAddr, sync::Arc};

use axum::{
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
use models::Post;
use page_meta::{
    build_post_meta, default_home_meta, default_not_found_meta, escape_html, PageMeta,
};
use state::{AppState, RouterState};

// Load the hot reload script content at compile time
const HOT_RELOAD_SCRIPT: &str = include_str!("hot_reload.js");
const HOT_RELOAD_TAG_START: &str = "<script>";
const HOT_RELOAD_TAG_END: &str = "</script>";

fn render_post_list(posts: &[Post]) -> String {
    let mut list_items = String::new();
    for post in posts {
        let subtitle_html = post
            .subtitle
            .as_deref()
            .map(|s| format!("<span class=\"sidebar-subtitle\">{s}</span>"))
            .unwrap_or_default();
        list_items.push_str(&format!(
            "<li><a href=\"/posts/{}\" class=\"no-underline\"><span class=\"sidebar-title\">{}</span>{}</a></li>",
            post.slug, post.title, subtitle_html
        ));
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

    let mut page = layout
        .replace("{{ banner }}", banner)
        .replace("{{ posts }}", &list_items)
        .replace("{{ page_title }}", &escaped_title)
        .replace("{{ page_description }}", &escaped_description)
        .replace("{{ page_url }}", &escaped_url)
        .replace("{{ page_image }}", &escaped_image)
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
    let banner = state.banner_html.read().await;
    let layout = state.layout_html.read().await;
    let home = state.home_html.read().await;
    let posts = state.posts.read().await;
    let meta = default_home_meta();

    let page = render_with_layout(&layout, &banner, &home, &posts, &meta, state.is_development);
    Html(page)
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

    let html_out = render_markdown_to_html(&post.markdown_body);
    let role_html = post
        .role
        .as_deref()
        .map(|r| format!("<span class=\"post-role\">{r}</span>"))
        .unwrap_or_default();
    let subtitle_html = post
        .subtitle
        .as_deref()
        .map(|s| format!("<p class=\"post-subtitle\">{s}</p>"))
        .unwrap_or_default();
    let body = format!(
        "<header class=\"post-header\">{role_html}<h1>{title}</h1>{subtitle_html}<p class=\"post-date\">{date}</p></header>{content}",
        title = &post.title,
        date = &post.date,
        content = html_out,
    );
    let meta = build_post_meta(
        &post.slug,
        Some(&post.title),
        post.description.as_deref(),
        post.subtitle.as_deref(),
        post.role.as_deref(),
        post.image.as_deref(),
        &post.markdown_body,
    );

    let layout = state.layout_html.read().await;
    let banner = state.banner_html.read().await;
    let posts = state.posts.read().await;

    let page = render_with_layout(&layout, &banner, &body, &posts, &meta, state.is_development);
    Html(page).into_response()
}

fn is_valid_post_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

async fn render_not_found_response(state: &Arc<AppState>, slug: &str) -> Response {
    let not_found_html = state.not_found_html.read().await;
    let body = not_found_html.replace("{{slug}}", slug);

    let layout = state.layout_html.read().await;
    let banner = state.banner_html.read().await;
    let posts = state.posts.read().await;
    let meta = default_not_found_meta(slug);
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

    let (banner_html, layout_html, home_html, not_found_html, posts) = match load_content().await {
        Ok(content) => content,
        Err(e) => {
            error!("Failed to load initial content files: {}", e);
            (
                String::new(),
                "<!doctype html><html><body>{{ content }}</body></html>".to_string(),
                "<p>Content loading failed during startup.</p>".to_string(),
                "<p>Post '{{slug}}' not found.</p>".to_string(),
                Vec::new(),
            )
        }
    };

    let state = Arc::new(AppState {
        banner_html: RwLock::new(banner_html),
        layout_html: RwLock::new(layout_html),
        home_html: RwLock::new(home_html),
        not_found_html: RwLock::new(not_found_html),
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
        .nest_service("/static", static_dir)
        .route_service("/favicon.ico", favicon_ico)
        .route_service("/favicon.png", favicon_png)
        .route("/ws", get(ws_handler))
        .with_state(router_state)
}

#[tokio::main]
async fn main() {
    setup_logging();

    let router_state = initialize_state().await;
    let app = setup_router(router_state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!(%addr, "listening");
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind to {}: {}", addr, e);
            return;
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        error!("Server error: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::{is_valid_post_slug, render_with_layout};
    use crate::models::Post;
    use crate::page_meta::PageMeta;

    fn test_layout() -> &'static str {
        "<html><head><title>{{ page_title }}</title><meta name=\"description\" content=\"{{ page_description }}\" /><meta property=\"og:title\" content=\"{{ page_title }}\" /><meta property=\"og:description\" content=\"{{ page_description }}\" /><meta property=\"og:url\" content=\"{{ page_url }}\" /><meta property=\"og:image\" content=\"{{ page_image }}\" />{{ page_role_meta }}<meta name=\"twitter:title\" content=\"{{ page_title }}\" /><meta name=\"twitter:description\" content=\"{{ page_description }}\" /><meta name=\"twitter:image\" content=\"{{ page_image }}\" /></head><body>{{ banner }}<main>{{ content }}</main><ul>{{ posts }}</ul></body></html>"
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
            role: Some("mechanism".to_string()),
        }
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
}
