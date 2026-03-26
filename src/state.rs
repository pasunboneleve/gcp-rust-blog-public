use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::models::{Post, SiteConfig};

pub type RefreshBroadcaster = broadcast::Sender<()>;

#[derive(Clone)]
pub struct DevloopEventClient {
    pub browser_path_url: String,
    pub token: String,
    pub client: reqwest::Client,
}

pub struct AppState {
    pub site_config: RwLock<SiteConfig>,
    pub banner_html: RwLock<String>,
    pub layout_html: RwLock<String>,
    pub home_post: RwLock<Post>,
    pub current_browser_path: RwLock<String>,
    pub devloop_event_client: Option<DevloopEventClient>,
    pub not_found_markdown: RwLock<String>, // supports {{slug}} placeholder
    pub posts: RwLock<Vec<Post>>,
    pub is_development: bool,
}

#[derive(Clone)]
pub struct RouterState {
    pub app_state: Arc<AppState>,
    pub broadcaster: RefreshBroadcaster,
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
