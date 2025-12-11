use std::{net::SocketAddr, sync::Arc};

use axum::{extract::Path, extract::State, response::Html, routing::get, Router};
use tower_http::services::{ServeDir, ServeFile};
use pulldown_cmark::{html, Options, Parser};
use tokio::{fs, net::TcpListener};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

struct AppState {
    banner_html: String,
    layout_html: String,
    home_html: String,
    not_found_html: String, // supports {{slug}} placeholder
}

fn render_with_layout(layout: &str, banner: &str, content: &str) -> String {
    layout
        .replace("{{ banner }}", banner)
        .replace("{{ content }}", content)
}

async fn homepage(State(state): State<Arc<AppState>>) -> Html<String> {
    let page = render_with_layout(&state.layout_html, &state.banner_html, &state.home_html);
    Html(page)
}

async fn render_post(Path(slug): Path<String>, State(state): State<Arc<AppState>>) -> Html<String> {
    let path = format!("content/posts/{}.md", slug);
    let md = match fs::read_to_string(&path).await {
        Ok(c) => c,
        Err(_) => {
            let body = state.not_found_html.replace("{{slug}}", &slug);
            let page = render_with_layout(&state.layout_html, &state.banner_html, &body);
            return Html(page);
        }
    };

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(&md, options);
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);

    let page = render_with_layout(&state.layout_html, &state.banner_html, &html_out);
    Html(page)
}

#[tokio::main]
async fn main() {
    // logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Strictly load HTML from content/ files (no inline HTML fallbacks)
    let banner_html = fs::read_to_string("content/banner.html")
        .await
        .expect("Missing content/banner.html");

    let layout_html = fs::read_to_string("content/layout.html")
        .await
        .expect("Missing content/layout.html");

    let home_html = fs::read_to_string("content/home.html")
        .await
        .expect("Missing content/home.html");

    let not_found_html = fs::read_to_string("content/not_found.html")
        .await
        .expect("Missing content/not_found.html");

    let state = Arc::new(AppState {
        banner_html,
        layout_html,
        home_html,
        not_found_html,
    });

    let static_dir = ServeDir::new("content/static");
    let favicon_ico = ServeFile::new("content/static/favicon.ico");
    let favicon_png = ServeFile::new("content/static/favicon.png");

    let app = Router::new()
        .route("/", get(homepage))
        .route("/posts/{slug}", get(render_post))
        .nest_service("/static", static_dir)
        .route_service("/favicon.ico", favicon_ico)
        .route_service("/favicon.png", favicon_png)
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!(%addr, "listening");
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
