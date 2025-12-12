use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::models::Post;

pub type RefreshBroadcaster = broadcast::Sender<()>;

pub struct AppState {
    pub banner_html: RwLock<String>,
    pub layout_html: RwLock<String>,
    pub home_html: RwLock<String>,
    pub not_found_html: RwLock<String>, // supports {{slug}} placeholder
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
