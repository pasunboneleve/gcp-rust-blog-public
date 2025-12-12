use tokio::fs;
use gray_matter::{engine::YAML, Matter};
use pulldown_cmark::{html, Options, Parser};
use tracing::{error, info};

use crate::models::{FrontMatter, Post};
use crate::state::AppState;

const CONTENT_DIR: &str = "content";

pub async fn load_content() -> Result<(String, String, String, String, Vec<Post>), std::io::Error> {
    let banner_html = fs::read_to_string(format!("{}/banner.html", CONTENT_DIR)).await?;
    let layout_html = fs::read_to_string(format!("{}/layout.html", CONTENT_DIR)).await?;
    let not_found_html = fs::read_to_string(format!("{}/not_found.html", CONTENT_DIR)).await?;

    // 1. Load home content as Markdown
    let home_md_content = fs::read_to_string(format!("{}/home.md", CONTENT_DIR)).await?;
    
    let matter = Matter::<YAML>::new();
    let result = matter.parse::<FrontMatter>(&home_md_content);
    
    let markdown_body = result.unwrap().content;

    // 2. Render Markdown to HTML
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(&markdown_body, options);
    let mut home_html = String::new();
    html::push_html(&mut home_html, parser);

    // 3. Load posts metadata
    let mut posts: Vec<Post> = Vec::new();
    let mut entries = fs::read_dir(format!("{}/posts", CONTENT_DIR)).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "md") {
            let file_content = fs::read_to_string(&path).await?;
            let matter = Matter::<YAML>::new();
            let result = matter.parse::<FrontMatter>(&file_content);

            let front_matter = match result {
                Ok(parsed) => parsed.data,
                Err(e) => {
                    error!("Failed to parse front matter: {}", e);
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

pub async fn reload_content(app_state: &AppState) {
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
            error!("Failed to reload content: {}", e);
        }
    }
}
