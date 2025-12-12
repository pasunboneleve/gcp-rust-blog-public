use std::{sync::Arc, time::Duration};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use notify_debouncer_full::{
    new_debouncer, DebouncedEvent,
    notify::{RecursiveMode, Watcher, Error as NotifyError},
};
use tracing::{debug, error, info};

use crate::content_loader::reload_content;
use crate::state::{AppState, RefreshBroadcaster};

const CONTENT_DIR: &str = "content";

pub async fn ws_handler(
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

pub fn start_content_watcher(tx: RefreshBroadcaster, app_state: Arc<AppState>) {
    info!("Starting content watcher for hot-reload...");
    tokio::spawn(async move {
        let (watcher_tx, mut watcher_rx) = tokio::sync::mpsc::channel(1);

        let mut debouncer = new_debouncer(Duration::from_millis(200), None, move |res: Result<Vec<DebouncedEvent>, Vec<NotifyError>>| {
            if let Ok(events) = res {
                // Filter out events that are just metadata changes or temporary files
                let relevant_events: Vec<&DebouncedEvent> = events.iter().filter(|event| {
                    // Check if the event type is relevant (modify, create, remove)
                    let is_relevant_kind = event.kind.is_modify()
                        || event.kind.is_create()
                        || event.kind.is_remove();

                    if !is_relevant_kind {
                        return false;
                    }

                    // Check paths for temporary files (Emacs: .#*, ~ backups)
                    let is_temp_file = event.event.paths.iter().any(|path| {
                        path.file_name()
                            .and_then(|name| name.to_str())
                            .map_or(false, |s| s.starts_with(".#") || s.ends_with('~'))
                    });

                    !is_temp_file
                }).collect();

                if !relevant_events.is_empty() {
                    debug!("Relevant file change detected: {:?}", relevant_events.iter().flat_map(|e| &e.event.paths).map(|p| p.display()).collect::<Vec<_>>());
                    if let Err(e) = watcher_tx.blocking_send(()) {
                        error!("Failed to send watcher event: {}", e);
                    }
                }
            } else if let Err(errors) = res {
                for e in errors {
                    error!("Watcher error: {}", e);
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
            
            reload_content(&app_state).await;

            // Send reload signal to all connected WebSocket clients
            if let Err(e) = tx.send(()) {
                error!("Failed to broadcast reload signal: {}", e);
            }
        }
    });
}
