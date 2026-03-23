use gray_matter::{engine::YAML, Matter};
use tokio::fs;
use tracing::{error, info};

use crate::markdown::render_markdown_to_html;
use crate::models::{FrontMatter, Post, SiteConfig};
use crate::state::AppState;

const CONTENT_DIR: &str = "content";

pub async fn load_content(
) -> Result<(SiteConfig, String, String, String, String, Vec<Post>), std::io::Error> {
    let site_config = load_site_config().await?;
    let banner_template = fs::read_to_string(format!("{}/banner.html", CONTENT_DIR)).await?;
    let layout_template = fs::read_to_string(format!("{}/layout.html", CONTENT_DIR)).await?;
    let banner_html = apply_site_config_template(&banner_template, &site_config);
    let layout_html = apply_site_config_template(&layout_template, &site_config);
    let not_found_markdown = fs::read_to_string(format!("{}/404.md", CONTENT_DIR)).await?;

    // 1. Load home content as Markdown
    let home_md_content = fs::read_to_string(format!("{}/home.md", CONTENT_DIR)).await?;

    let matter = Matter::<YAML>::new();
    let result = matter.parse::<FrontMatter>(&home_md_content);
    let markdown_body = match result {
        Ok(parsed) => parsed.content,
        Err(e) => {
            error!("Failed to parse home front matter: {}", e);
            home_md_content
        }
    };

    // 2. Render Markdown to HTML
    let home_html = render_markdown_to_html(&markdown_body);

    // 3. Load posts metadata
    let mut posts: Vec<Post> = Vec::new();
    let mut entries = fs::read_dir(format!("{}/posts", CONTENT_DIR)).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "md") {
            let file_content = fs::read_to_string(&path).await?;
            let matter = Matter::<YAML>::new();
            let result = matter.parse::<FrontMatter>(&file_content);

            let (front_matter, markdown_body) = match result {
                Ok(parsed) => (parsed.data, parsed.content),
                Err(e) => {
                    error!("Failed to parse front matter: {}", e);
                    (
                        Some(FrontMatter {
                            title: "Error".to_string(),
                            date: "Error".to_string(),
                            slug: "Error".to_string(),
                            description: None,
                            image: None,
                            role: None,
                            subtitle: None,
                        }),
                        file_content,
                    )
                }
            };

            posts.push(Post {
                title: front_matter
                    .as_ref()
                    .map(|fm| fm.title.clone())
                    .unwrap_or_else(|| "Error".to_string()),
                slug: front_matter
                    .as_ref()
                    .map(|fm| fm.slug.clone())
                    .unwrap_or_else(|| "error".to_string()),
                date: front_matter
                    .as_ref()
                    .map(|fm| fm.date.clone())
                    .unwrap_or_else(|| "Error".to_string()),
                description: front_matter.as_ref().and_then(|fm| fm.description.clone()),
                image: front_matter.as_ref().and_then(|fm| fm.image.clone()),
                role: front_matter.as_ref().and_then(|fm| fm.role.clone()),
                subtitle: front_matter.as_ref().and_then(|fm| fm.subtitle.clone()),
                markdown_body,
            });
        }
    }

    // Keep listing stable across environments and fs implementations.
    posts.sort_by(|a, b| b.slug.cmp(&a.slug));

    Ok((
        site_config,
        banner_html,
        layout_html,
        home_html,
        not_found_markdown,
        posts,
    ))
}

pub async fn reload_content(app_state: &AppState) {
    info!("Reloading application content...");
    match load_content().await {
        Ok((site_config, banner, layout, home, not_found, posts)) => {
            *app_state.site_config.write().await = site_config;
            *app_state.banner_html.write().await = banner;
            *app_state.layout_html.write().await = layout;
            *app_state.home_html.write().await = home;
            *app_state.not_found_markdown.write().await = not_found;
            *app_state.posts.write().await = posts;
            info!("Content successfully reloaded.");
        }
        Err(e) => {
            error!("Failed to reload content: {}", e);
        }
    }
}

async fn load_site_config() -> Result<SiteConfig, std::io::Error> {
    let raw = fs::read_to_string(format!("{}/site.toml", CONTENT_DIR)).await?;
    toml::from_str(&raw)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
}

fn apply_site_config_template(template: &str, site_config: &SiteConfig) -> String {
    template
        .replace("{{ site_title }}", &site_config.title)
        .replace("{{ site_author }}", &site_config.author)
        .replace("{{ site_description }}", &site_config.description)
        .replace("{{ site_og_name }}", &site_config.og_site_name)
}
