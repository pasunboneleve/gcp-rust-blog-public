use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{Path, State},
    response::Html,
    routing::{get, get_service},
    Router,
};
use gray_matter::{engine::YAML, Matter};
use tokio::{fs, net::TcpListener, sync::RwLock};
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing::{error, info};

mod models;
mod state;
mod content_loader;
mod hot_reload;
mod markdown;

use models::{Post, FrontMatter};
use state::{AppState, RouterState};
use content_loader::load_content;
use hot_reload::{ws_handler, start_content_watcher};
use markdown::render_markdown_to_html;

// Load the hot reload script content at compile time
const HOT_RELOAD_SCRIPT: &str = include_str!("hot_reload.js");
const HOT_RELOAD_TAG_START: &str = "<script>";
const HOT_RELOAD_TAG_END: &str = "</script>";

fn render_post_list(posts: &[Post]) -> String {
    let mut list_items = String::new();
    for post in posts {
        list_items.push_str(&format!(
            "<li><a href=\"/posts/{}\" class=\"text-yellow no-underline\">{}</a></li>",
            post.slug, post.title
        ));
    }
    list_items
}

fn render_with_layout(
    layout: &str,
    banner: &str,
    content: &str,
    posts: &[Post],
    is_development: bool,
) -> String {
    let list_items = render_post_list(posts);

    let mut page = layout
        .replace("{{ banner }}", banner)
        .replace("{{ posts }}", &list_items)
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

async fn homepage(
    State(state): State<Arc<AppState>>,
) -> Html<String> {
    let banner = state.banner_html.read().await;
    let layout = state.layout_html.read().await;
    let home = state.home_html.read().await;
    let posts = state.posts.read().await;

    let page = render_with_layout(
        &layout,
        &banner,
        &home,
        &posts,
        state.is_development,
    );
    Html(page)
}

async fn render_post(
    Path(slug): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Html<String> {
    let path = format!("content/posts/{}.md", slug);
    let file_content = match fs::read_to_string(&path).await {
        Ok(c) => c,
        Err(_) => {
            let not_found_html = state.not_found_html.read().await;
            let body = not_found_html.replace("{{slug}}", &slug);
            
            let layout = state.layout_html.read().await;
            let banner = state.banner_html.read().await;
            let posts = state.posts.read().await;

            let page = render_with_layout(
                &layout,
                &banner,
                &body,
                &posts,
                state.is_development,
            );
            return Html(page);
        }
    };

    let matter = Matter::<YAML>::new();
    let result = matter.parse::<FrontMatter>(&file_content);

    let front_matter = match result {
        Ok(ref parsed) => parsed.data.clone(),
        Err(ref e) => {
            tracing::error!("Failed to parse front matter: {}", e);
            Some(FrontMatter {
                title: "Error".to_string(),
                date: "Error".to_string(),
                slug: "Error".to_string(),
            })
        }
    };
    let markdown_body = match result {
        Ok(parsed) => parsed.content,
        Err(e) => {
            error!("Failed to parse post front matter content for '{}': {}", slug, e);
            file_content
        }
    };

    let html_out = render_markdown_to_html(&markdown_body);

    let body = match front_matter {
        Some(fm) => format!(
            "<h1>{}</h1><p style=\"font-size: smaller; color: #888;\">{}</p>{}",
            fm.title, fm.date, html_out
        ),
        None => format!("<h1>Error: No Front Matter</h1>{}", html_out),
    };
    
    let layout = state.layout_html.read().await;
    let banner = state.banner_html.read().await;
    let posts = state.posts.read().await;

    let page = render_with_layout(
        &layout,
        &banner,
        &body,
        &posts,
        state.is_development,
    );
    Html(page)
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
    use super::render_with_layout;
    use crate::models::Post;

    fn test_layout() -> &'static str {
        "<html><body>{{ banner }}<main>{{ content }}</main><ul>{{ posts }}</ul></body></html>"
    }

    fn test_posts() -> Vec<Post> {
        vec![Post {
            title: "First post".to_string(),
            slug: "first-post".to_string(),
        }]
    }

    #[test]
    fn injects_hot_reload_script_once_in_development() {
        let page =
            render_with_layout(test_layout(), "<header>banner</header>", "content", &test_posts(), true);
        assert_eq!(page.matches("new WebSocket").count(), 1);
        assert_eq!(page.matches("<script>").count(), 1);
    }

    #[test]
    fn injects_script_at_end_when_body_tag_is_missing() {
        let layout = "<html><div>{{ banner }}</div><main>{{ content }}</main></html>";
        let page = render_with_layout(layout, "banner", "content", &test_posts(), true);
        assert!(page.ends_with("</script>"));
        assert_eq!(page.matches("new WebSocket").count(), 1);
    }

    #[test]
    fn does_not_inject_script_in_non_development() {
        let page = render_with_layout(test_layout(), "banner", "content", &test_posts(), false);
        assert_eq!(page.matches("new WebSocket").count(), 0);
    }

    #[test]
    fn does_not_replace_posts_placeholder_inside_content() {
        let content = "<p>literal {{ posts }}</p>";
        let page = render_with_layout(test_layout(), "banner", content, &test_posts(), false);
        assert!(page.contains("<p>literal {{ posts }}</p>"));
    }
}
