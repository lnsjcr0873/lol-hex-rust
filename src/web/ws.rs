use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::broadcast;

pub static BROADCAST_CHANNEL_CAPACITY: usize = 128;

#[derive(Clone)]
pub struct WsBroadcaster {
    sender: broadcast::Sender<String>,
}

impl Default for WsBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

impl WsBroadcaster {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(BROADCAST_CHANNEL_CAPACITY);
        Self { sender }
    }

    pub fn send_event(&self, event_name: &str, payload: serde_json::Value) {
        let mut msg = json!({ "event": event_name });
        if let Some(obj) = msg.as_object_mut() {
            if let Some(payload_obj) = payload.as_object() {
                for (k, v) in payload_obj {
                    obj.insert(k.clone(), v.clone());
                }
            } else {
                obj.insert("data".to_string(), payload);
            }
        }
        let _ = self.sender.send(msg.to_string());
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.sender.subscribe()
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(broadcaster): State<Arc<WsBroadcaster>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, broadcaster))
}

pub async fn handle_socket(mut socket: WebSocket, broadcaster: Arc<WsBroadcaster>) {
    let mut rx = broadcaster.subscribe();

    // Send initial handshake
    let init_msg = json!({
        "event": "INIT_STATE",
        "lcuConnected": true,
        "gamePhase": "None"
    });
    let _ = socket.send(Message::Text(init_msg.to_string())).await;

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
                    Some(Ok(Message::Ping(payload))) => {
                        if socket.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}
