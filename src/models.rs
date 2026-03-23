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
