use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::{Html, IntoResponse},
    routing::{get, get_service},
    Router,
};
use gray_matter::{engine::YAML, Matter};
use notify_debouncer_full::{
    new_debouncer, DebouncedEvent,
    notify::{RecursiveMode, Watcher, Error as NotifyError},
};
use pulldown_cmark::{html, Options, Parser};
use serde::Deserialize;
use tokio::{fs, net::TcpListener, sync::{broadcast, RwLock}};
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing::{debug, info}; // Added info import for clarity

const HOT_RELOAD_SCRIPT: &str = r#"
<script>
    const socket = new WebSocket("ws://" + window.location.host + "/ws");
    socket.onmessage = (event) => {
        if (event.data === "reload") {
            window.location.reload();
        }
    };
</script>
"#;

const CONTENT_DIR: &str = "content";

#[derive(Deserialize, Debug, Clone)]
struct FrontMatter {
    title: String,
    date: String,
    slug: String,
}

struct Post {
    title: String,
    slug: String,
}

type RefreshBroadcaster = broadcast::Sender<()>;

struct AppState {
    banner_html: RwLock<String>,
    layout_html: RwLock<String>,
    home_html: RwLock<String>,
    not_found_html: RwLock<String>, // supports {{slug}} placeholder
    posts: RwLock<Vec<Post>>,
    is_development: bool,
}

// --- Router State Definition and FromRef implementations ---

#[derive(Clone)]
struct RouterState {
    app_state: Arc<AppState>,
    broadcaster: RefreshBroadcaster,
}

impl axum::extract::FromRef<RouterState> for Arc<AppState> {
    fn from_ref(state: &RouterState) -> Self {
        state.app_state.clone()
    }
}

impl axum::extract::FromRef<RouterState> for RefreshBroadcaster {
    fn from_ref(state: &RouterState) -> Self {
        state.broadcaster.clone()
    }
}

// --------------------------------------------------------------

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
        page = page.replace("</body>", &format!("{}</body>", HOT_RELOAD_SCRIPT));
    }

    page
}

// Helper function to load content, reused in main and reload_content
async fn load_content() -> Result<(String, String, String, String, Vec<Post>), std::io::Error> {
    let banner_html = fs::read_to_string("content/banner.html").await?;
    let layout_html = fs::read_to_string("content/layout.html").await?;
    let home_html = fs::read_to_string("content/home.html").await?;
    let not_found_html = fs::read_to_string("content/not_found.html").await?;

    let mut posts: Vec<Post> = Vec::new();
    let mut entries = fs::read_dir("content/posts").await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "md") {
            let file_content = fs::read_to_string(&path).await?;
            let matter = Matter::<YAML>::new();
            let result = matter.parse::<FrontMatter>(&file_content);

            let front_matter = match result {
                Ok(parsed) => parsed.data,
                Err(e) => {
                    tracing::error!("Failed to parse front matter: {}", e);
                    Some(FrontMatter {
                        title: "Error".to_string(),
                        date: "Error".to_string(),
                        slug: "Error".to_string(),
                    })
                }
            };

            posts.push(Post {
                title: front_matter
                    .clone()
                    .map(|fm| fm.title)
                    .unwrap_or("Error".to_string()),
                slug: front_matter
                    .clone()
                    .map(|fm| fm.slug)
                    .unwrap_or("error".to_string()),
            });
        }
    }
    Ok((banner_html, layout_html, home_html, not_found_html, posts))
}

async fn reload_content(app_state: &AppState) {
    info!("Reloading application content...");
    match load_content().await {
        Ok((banner, layout, home, not_found, posts)) => {
            *app_state.banner_html.write().await = banner;
            *app_state.layout_html.write().await = layout;
            *app_state.home_html.write().await = home;
            *app_state.not_found_html.write().await = not_found;
            *app_state.posts.write().await = posts;
            info!("Content successfully reloaded.");
        }
        Err(e) => {
            tracing::error!("Failed to reload content: {}", e);
        }
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

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(tx): State<RefreshBroadcaster>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, tx))
}

async fn handle_socket(mut socket: WebSocket, tx: RefreshBroadcaster) {
    let mut rx = tx.subscribe();

    // Wait for a reload signal
    if rx.recv().await.is_ok() {
        // Send reload message to client
        if socket.send(Message::Text("reload".to_string().into())).await.is_err() {
            debug!("Client disconnected before reload message could be sent");
        }
    }
    // The socket will close when this function returns
}

// Signature change: now accepts Arc<AppState>
fn start_content_watcher(tx: RefreshBroadcaster, app_state: Arc<AppState>) {
    info!("Starting content watcher for hot-reload...");
    tokio::spawn(async move {
        let (watcher_tx, mut watcher_rx) = tokio::sync::mpsc::channel(1);

        let mut debouncer = new_debouncer(Duration::from_millis(200), None, move |res: Result<Vec<DebouncedEvent>, Vec<NotifyError>>| {
            if let Ok(events) = res {
                // Filter out events that are just metadata changes or temporary files
                let relevant_change = events.iter().any(|event| {
                    event.kind.is_modify()
                        || event.kind.is_create()
                        || event.kind.is_remove()
                });

                if relevant_change {
                    debug!("Relevant file change detected: {:?}", events.iter().flat_map(|e| &e.event.paths).map(|p| p.display()).collect::<Vec<_>>());
                    if let Err(e) = watcher_tx.blocking_send(()) {
                        tracing::error!("Failed to send watcher event: {}", e);
                    }
                }
            } else if let Err(errors) = res {
                for e in errors {
                    tracing::error!("Watcher error: {}", e);
                }
            }
        })
        .expect("Failed to create debouncer");

        debouncer
            .watcher()
            .watch(CONTENT_DIR.as_ref(), RecursiveMode::Recursive)
            .expect("Failed to start watching content directory");

        // Keep the debouncer alive and wait for events
        while watcher_rx.recv().await.is_some() {
            info!("Content change detected, reloading content and sending signal...");
            
            // RELOAD CONTENT HERE
            reload_content(&app_state).await;

            // Send reload signal to all connected WebSocket clients
            if let Err(e) = tx.send(()) {
                tracing::error!("Failed to broadcast reload signal: {}", e);
            }
        }
    });
}

#[tokio::main]
async fn main() {
    let is_development = std::env::var("RUST_ENV")
        .map(|v| v == "development")
        .unwrap_or(false);

    // logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    info!("RUST_ENV is set to development: {}", is_development); // <-- Added log for confirmation

    // Load initial content
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

    // Hot-reload setup
    let (tx, _rx) = broadcast::channel(1);
    if is_development {
        info!("Hot reload enabled. Check logs for file change events.");
        start_content_watcher(tx.clone(), state.clone());
    }

    // Combine states into a single RouterState
    let router_state = RouterState {
        app_state: state,
        broadcaster: tx,
    };

    let static_dir = get_service(ServeDir::new("content/static"));
    let favicon_ico = get_service(ServeFile::new("content/static/favicon.ico"));
    let favicon_png = get_service(ServeFile::new("content/static/favicon.png"));

    let app = Router::new()
        .route("/", get(homepage))
        .route("/posts/{slug}", get(render_post))
        .nest_service("/static", static_dir)
        .route_service("/favicon.ico", favicon_ico)
        .route_service("/favicon.png", favicon_png)
        .route("/ws", get(ws_handler))
        .with_state(router_state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    info!(%addr, "listening");
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
