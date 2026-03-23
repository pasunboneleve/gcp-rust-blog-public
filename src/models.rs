use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct FrontMatter {
    pub title: String,
    pub date: String,
    pub slug: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub role: Option<String>,
    pub subtitle: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SiteConfig {
    pub title: String,
    pub author: String,
    pub description: String,
    pub og_site_name: String,
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            title: "Bon Élève Blog".to_string(),
            author: "Daniel Vianna".to_string(),
            description: "Engineering notes on making change cheap.".to_string(),
            og_site_name: "Bon Élève Blog".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct Post {
    pub title: String,
    pub slug: String,
    pub date: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub role: Option<String>,
    pub subtitle: Option<String>,
    pub markdown_body: String,
}
