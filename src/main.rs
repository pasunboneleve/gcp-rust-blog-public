use std::{net::SocketAddr, sync::Arc};

use axum::{extract::Path, extract::State, response::Html, routing::get, Router};
use gray_matter::{Matter, engine::YAML};
use pulldown_cmark::{html, Options, Parser};
use tokio::{fs, net::TcpListener};
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
struct FrontMatter {
    title: String,
    slug: String,
}

struct Post {
    title: String,
    slug: String,
}

struct AppState {
    banner_html: String,
    layout_html: String,
    home_html: String,
    not_found_html: String, // supports {{slug}} placeholder
    posts: Vec<Post>,
}

fn render_with_layout(layout: &str, banner: &str, content: &str) -> String {
    layout
        .replace("{{ banner }}", banner)
        .replace("{{ content }}", content)
}

async fn homepage(State(state): State<Arc<AppState>>) -> Html<String> {
    let mut list_items = String::new();
    for post in &state.posts {
        list_items.push_str(&format!(
            "<li><a href=\"/posts/{}\" class=\"text-blue no-underline\">{}</a></li>",
            post.slug, post.title
        ));
    }

    let home_html_with_posts = state
        .home_html
        .replace("{{posts}}", &list_items);

    let page = render_with_layout(&state.layout_html, &state.banner_html, &home_html_with_posts);
    Html(page)
}

async fn render_post(Path(slug): Path<String>, State(state): State<Arc<AppState>>) -> Html<String> {
    let path = format!("content/posts/{}.md", slug);
    let file_content = match fs::read_to_string(&path).await {
        Ok(c) => c,
        Err(_) => {
            let body = state.not_found_html.replace("{{slug}}", &slug);
            let page = render_with_layout(&state.layout_html, &state.banner_html, &body);
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
        Some(fm) => format!("<h1>{}</h1>{}", fm.title, html_out),
        None => format!("<h1>Error: No Front Matter</h1>{}", html_out),
    };
    let page = render_with_layout(&state.layout_html, &state.banner_html, &body);
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

    let mut posts: Vec<Post> = Vec::new();
    let mut entries = fs::read_dir("content/posts").await.unwrap();

    while let Some(entry) = entries.next_entry().await.unwrap() {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "md") {
            let file_content = fs::read_to_string(&path).await.unwrap();
            let matter = Matter::<YAML>::new();
            let result = matter.parse::<FrontMatter>(&file_content);

            let front_matter = match result {
                Ok(parsed) => parsed.data,
                Err(e) => {
                    tracing::error!("Failed to parse front matter: {}", e);
                    Some(FrontMatter {
                        title: "Error".to_string(),
                        slug: "Error".to_string(),
                    })
                }
            };

            posts.push(Post {
                title: front_matter.clone().map(|fm| fm.title).unwrap_or("Error".to_string()),
                slug: front_matter.clone().map(|fm| fm.slug).unwrap_or("error".to_string()),
            });
        }
    }

    let state = Arc::new(AppState {
        banner_html,
        layout_html,
        home_html,
        not_found_html,
        posts,
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
