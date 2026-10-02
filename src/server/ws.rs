use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;

#[derive(Clone)]
pub struct WsBroadcaster {
    pub tx: broadcast::Sender<String>,
}

impl Default for WsBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

impl WsBroadcaster {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(100);
        Self { tx }
    }

    pub fn broadcast(&self, payload: &Value) {
        if let Ok(msg) = serde_json::to_string(payload) {
            let _ = self.tx.send(msg);
        }
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    broadcaster: Arc<WsBroadcaster>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, broadcaster))
}

async fn handle_socket(mut socket: WebSocket, broadcaster: Arc<WsBroadcaster>) {
    let mut rx = broadcaster.tx.subscribe();

    // Send initial connection event
    let init = serde_json::json!({
        "event": "INIT_STATE",
        "lcuConnected": false,
        "gamePhase": "None"
    });
    if let Ok(txt) = serde_json::to_string(&init) {
        let _ = socket.send(Message::Text(txt)).await;
    }

    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Ok(text) => {
                        if socket.send(Message::Text(text)).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            client_msg = socket.recv() => {
                match client_msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}
