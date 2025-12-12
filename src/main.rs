use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{Path, State},
    response::Html,
    routing::{get, get_service},
    Router,
};
use gray_matter::{engine::YAML, Matter};
use pulldown_cmark::{html, Options, Parser};
use tokio::{fs, net::TcpListener, sync::RwLock};
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing::info;

mod models;
mod state;
mod content_loader;
mod hot_reload;

use models::{Post, FrontMatter};
use state::{AppState, RouterState};
use content_loader::load_content;
use hot_reload::{ws_handler, start_content_watcher};

// Load the hot reload script content at compile time
const HOT_RELOAD_SCRIPT: &str = include_str!("hot_reload.js");

fn render_with_layout(
    layout: &str,
    banner: &str,
    content: &str,
    posts: &Vec<Post>,
    is_development: bool,
) -> String {
    let mut list_items = String::new();
    for post in posts {
        list_items.push_str(&format!(
            "<li><a href=\"/posts/{}\" class=\"text-blue no-underline\">{}</a></li>",
            post.slug, post.title
        ));
    }

    let mut page = layout
        .replace("{{ banner }}", banner)
        .replace("{{ content }}", content)
        .replace("{{ posts }}", &list_items);

    if is_development {
        // Wrap the JS content in <script> tags for injection
        page = page.replace("</body>", &format!("<script>{}</script></body>", HOT_RELOAD_SCRIPT));
    }

    page
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
    let markdown_body = result.unwrap().content;

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(&markdown_body, options);
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);

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

    let (banner_html, layout_html, home_html, not_found_html, posts) = load_content()
        .await
        .expect("Failed to load initial content files");

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
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
