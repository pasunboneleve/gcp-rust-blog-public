use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct FrontMatter {
    pub title: String,
    pub date: String,
    pub slug: String,
}

#[derive(Clone)]
pub struct Post {
    pub title: String,
    pub slug: String,
}
