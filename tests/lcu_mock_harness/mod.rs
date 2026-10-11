pub mod fixtures;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{oneshot, RwLock};

#[derive(Default)]
#[allow(dead_code)]
pub struct MockLcuState {
    pub gameflow_phase: RwLock<String>,
    pub champ_select_session: RwLock<Value>,
    pub current_summoner: RwLock<Value>,
    pub conversations: RwLock<Vec<Value>>,
    pub chat_participants: RwLock<HashMap<String, Vec<Value>>>,
    pub chat_messages: RwLock<Vec<(String, String)>>, // (room_id, message)
    pub ranked_stats: RwLock<HashMap<String, Value>>,
    pub match_history: RwLock<HashMap<String, Value>>,
    pub match_details: RwLock<HashMap<String, Value>>,
    pub ready_check: RwLock<Value>,
    pub ready_check_accepted: RwLock<bool>,
    pub ready_check_declined: RwLock<bool>,
    pub auth_token: RwLock<String>,
    pub status_override: RwLock<Option<StatusCode>>,
    pub request_count: AtomicUsize,
}

#[allow(dead_code)]
impl MockLcuState {
    pub fn new(auth_token: &str) -> Self {
        Self {
            gameflow_phase: RwLock::new("ChampSelect".to_string()),
            champ_select_session: RwLock::new(Value::Null),
            current_summoner: RwLock::new(Value::Null),
            conversations: RwLock::new(Vec::new()),
            chat_participants: RwLock::new(HashMap::new()),
            chat_messages: RwLock::new(Vec::new()),
            ranked_stats: RwLock::new(HashMap::new()),
            match_history: RwLock::new(HashMap::new()),
            match_details: RwLock::new(HashMap::new()),
            ready_check: RwLock::new(json!({
                "state": "InProgress",
                "playerResponse": "None",
                "timer": 10.0
            })),
            ready_check_accepted: RwLock::new(false),
            ready_check_declined: RwLock::new(false),
            auth_token: RwLock::new(auth_token.to_string()),
            status_override: RwLock::new(None),
            request_count: AtomicUsize::new(0),
        }
    }

    pub async fn set_phase(&self, phase: &str) {
        let mut g = self.gameflow_phase.write().await;
        *g = phase.to_string();
    }

    pub async fn set_champ_select_session(&self, session: Value) {
        let mut g = self.champ_select_session.write().await;
        *g = session;
    }

    pub async fn set_current_summoner(&self, summoner: Value) {
        let mut g = self.current_summoner.write().await;
        *g = summoner;
    }

    pub async fn set_chat_participants(&self, room_id: &str, participants: Vec<Value>) {
        let mut g = self.chat_participants.write().await;
        g.insert(room_id.to_string(), participants);
    }

    pub async fn set_match_history(&self, puuid: &str, history: Value) {
        let mut g = self.match_history.write().await;
        g.insert(puuid.to_string(), history);
    }

    pub async fn set_match_detail(&self, game_id: i64, detail: Value) {
        let mut g = self.match_details.write().await;
        g.insert(game_id.to_string(), detail);
    }

    pub async fn set_ranked_stats(&self, puuid: &str, stats: Value) {
        let mut g = self.ranked_stats.write().await;
        g.insert(puuid.to_string(), stats);
    }

    pub async fn set_status_override(&self, status: Option<StatusCode>) {
        let mut g = self.status_override.write().await;
        *g = status;
    }

    pub async fn get_sent_messages(&self) -> Vec<(String, String)> {
        let g = self.chat_messages.read().await;
        g.clone()
    }

    pub fn get_request_count(&self) -> usize {
        self.request_count.load(Ordering::Relaxed)
    }
}

#[allow(dead_code)]
pub struct MockLcuServer {
    pub addr: SocketAddr,
    pub port: u16,
    pub auth_token: String,
    pub state: Arc<MockLcuState>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

#[allow(dead_code)]
impl MockLcuServer {
    /// Start the in-process Mock LCU HTTP Server on an ephemeral port
    pub async fn start() -> Self {
        let auth_token = format!("mock-token-{}", std::process::id());
        let state = Arc::new(MockLcuState::new(&auth_token));

        let app = Router::new()
            .route("/lol-gameflow/v1/gameflow-phase", get(handle_gameflow_phase))
            .route("/lol-champ-select/v1/session", get(handle_champ_select_session))
            .route("/lol-chat/v1/conversations", get(handle_conversations))
            .route(
                "/lol-chat/v1/conversations/:room_id/participants",
                get(handle_participants),
            )
            .route(
                "/lol-chat/v1/conversations/:room_id/messages",
                post(handle_messages),
            )
            .route(
                "/lol-match-history/v1/products/lol/:puuid/matches",
                get(handle_match_history),
            )
            .route(
                "/lol-match-history/v1/games/:game_id",
                get(handle_game_detail),
            )
            .route("/lol-summoner/v1/current-summoner", get(handle_current_summoner))
            .route("/lol-ranked/v1/ranked-stats/:puuid", get(handle_ranked_stats))
            .route("/lol-matchmaking/v1/ready-check", get(handle_ready_check))
            .route(
                "/lol-matchmaking/v1/ready-check/accept",
                post(handle_ready_check_accept),
            )
            .route(
                "/lol-matchmaking/v1/ready-check/decline",
                post(handle_ready_check_decline),
            )
            .with_state(state.clone());

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral port");
        let addr = listener.local_addr().expect("local addr");
        let port = addr.port();

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await
                .ok();
        });

        Self {
            addr,
            port,
            auth_token,
            state,
            shutdown_tx: Some(shutdown_tx),
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{}", self.port, path)
    }

    pub fn lockfile_content(&self) -> String {
        format!("LeagueClient:{}:{}:{}:http", std::process::id(), self.port, self.auth_token)
    }

    pub fn stop(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

impl Drop for MockLcuServer {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

// Axum Handlers

async fn check_override_and_auth(
    state: &Arc<MockLcuState>,
    headers: &HeaderMap,
) -> Option<Response> {
    state.request_count.fetch_add(1, Ordering::Relaxed);

    // 1. Status override check
    if let Some(status) = *state.status_override.read().await {
        return Some((status, "Simulated status override").into_response());
    }

    // 2. Auth token check (if header provided)
    if let Some(auth_val) = headers.get("authorization").and_then(|v| v.to_str().ok()) {
        if !auth_val.starts_with("Basic ") {
            return Some((StatusCode::UNAUTHORIZED, "Invalid authorization format").into_response());
        }
    }
    None
}

async fn handle_gameflow_phase(
    State(state): State<Arc<MockLcuState>>,
    headers: HeaderMap,
) -> Response {
    if let Some(resp) = check_override_and_auth(&state, &headers).await {
        return resp;
    }
    let phase = state.gameflow_phase.read().await;
    Json(phase.clone()).into_response()
}

async fn handle_champ_select_session(
    State(state): State<Arc<MockLcuState>>,
    headers: HeaderMap,
) -> Response {
    if let Some(resp) = check_override_and_auth(&state, &headers).await {
        return resp;
    }
    let session = state.champ_select_session.read().await;
    Json(session.clone()).into_response()
}

async fn handle_conversations(
    State(state): State<Arc<MockLcuState>>,
    headers: HeaderMap,
) -> Response {
    if let Some(resp) = check_override_and_auth(&state, &headers).await {
        return resp;
    }
    let convs = state.conversations.read().await;
    Json(convs.clone()).into_response()
}

async fn handle_participants(
    State(state): State<Arc<MockLcuState>>,
    Path(room_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    if let Some(resp) = check_override_and_auth(&state, &headers).await {
        return resp;
    }
    let map = state.chat_participants.read().await;
    let list = map.get(&room_id).cloned().unwrap_or_default();
    Json(list).into_response()
}

async fn handle_messages(
    State(state): State<Arc<MockLcuState>>,
    Path(room_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Response {
    if let Some(resp) = check_override_and_auth(&state, &headers).await {
        return resp;
    }
    let body = payload
        .get("body")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let mut msgs = state.chat_messages.write().await;
    msgs.push((room_id, body));
    StatusCode::OK.into_response()
}

async fn handle_match_history(
    State(state): State<Arc<MockLcuState>>,
    Path(puuid): Path<String>,
    headers: HeaderMap,
) -> Response {
    if let Some(resp) = check_override_and_auth(&state, &headers).await {
        return resp;
    }
    let map = state.match_history.read().await;
    if let Some(val) = map.get(&puuid) {
        Json(val.clone()).into_response()
    } else {
        (StatusCode::NOT_FOUND, "No match history found").into_response()
    }
}

async fn handle_game_detail(
    State(state): State<Arc<MockLcuState>>,
    Path(game_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    if let Some(resp) = check_override_and_auth(&state, &headers).await {
        return resp;
    }
    let map = state.match_details.read().await;
    if let Some(val) = map.get(&game_id) {
        Json(val.clone()).into_response()
    } else {
        (StatusCode::NOT_FOUND, "Game not found").into_response()
    }
}

async fn handle_current_summoner(
    State(state): State<Arc<MockLcuState>>,
    headers: HeaderMap,
) -> Response {
    if let Some(resp) = check_override_and_auth(&state, &headers).await {
        return resp;
    }
    let cur = state.current_summoner.read().await;
    if cur.is_null() {
        (StatusCode::NOT_FOUND, "No current summoner").into_response()
    } else {
        Json(cur.clone()).into_response()
    }
}

async fn handle_ranked_stats(
    State(state): State<Arc<MockLcuState>>,
    Path(puuid): Path<String>,
    headers: HeaderMap,
) -> Response {
    if let Some(resp) = check_override_and_auth(&state, &headers).await {
        return resp;
    }
    let map = state.ranked_stats.read().await;
    if let Some(val) = map.get(&puuid) {
        Json(val.clone()).into_response()
    } else {
        Json(fixtures::canned_ranked_stats("GOLD", 50, 20, 15)).into_response()
    }
}

async fn handle_ready_check(
    State(state): State<Arc<MockLcuState>>,
    headers: HeaderMap,
) -> Response {
    if let Some(resp) = check_override_and_auth(&state, &headers).await {
        return resp;
    }
    let rc = state.ready_check.read().await;
    Json(rc.clone()).into_response()
}

async fn handle_ready_check_accept(
    State(state): State<Arc<MockLcuState>>,
    headers: HeaderMap,
) -> Response {
    if let Some(resp) = check_override_and_auth(&state, &headers).await {
        return resp;
    }
    let mut g = state.ready_check_accepted.write().await;
    *g = true;
    StatusCode::OK.into_response()
}

async fn handle_ready_check_decline(
    State(state): State<Arc<MockLcuState>>,
    headers: HeaderMap,
) -> Response {
    if let Some(resp) = check_override_and_auth(&state, &headers).await {
        return resp;
    }
    let mut g = state.ready_check_declined.write().await;
    *g = true;
    StatusCode::OK.into_response()
}

/// Lightweight test client connecting to MockLcuServer
#[derive(Clone)]
#[allow(dead_code)]
pub struct MockLcuClient {
    client: reqwest::Client,
    base_url: String,
    auth_header: String,
}

#[allow(dead_code)]
impl MockLcuClient {
    pub fn new(port: u16, token: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(1500))
            .build()
            .unwrap_or_default();
        let basic = format!("riot:{}", token);
        let encoded = {
            const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
            let mut result = String::new();
            let bytes = basic.as_bytes();
            for chunk in bytes.chunks(3) {
                let b0 = chunk[0] as usize;
                let b1 = if chunk.len() > 1 { chunk[1] as usize } else { 0 };
                let b2 = if chunk.len() > 2 { chunk[2] as usize } else { 0 };
                let n = (b0 << 16) | (b1 << 8) | b2;
                result.push(CHARSET[(n >> 18) & 63] as char);
                result.push(CHARSET[(n >> 12) & 63] as char);
                if chunk.len() > 1 {
                    result.push(CHARSET[(n >> 6) & 63] as char);
                } else {
                    result.push('=');
                }
                if chunk.len() > 2 {
                    result.push(CHARSET[n & 63] as char);
                } else {
                    result.push('=');
                }
            }
            result
        };

        Self {
            client,
            base_url: format!("http://127.0.0.1:{}", port),
            auth_header: format!("Basic {}", encoded),
        }
    }

    pub fn from_server(server: &MockLcuServer) -> Self {
        Self::new(server.port, &server.auth_token)
    }

    pub async fn get_gameflow_phase(&self) -> Result<String, String> {
        let res = self
            .client
            .get(format!("{}/lol-gameflow/v1/gameflow-phase", self.base_url))
            .header("Authorization", &self.auth_header)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if res.status().is_success() {
            let text = res.text().await.map_err(|e| e.to_string())?;
            Ok(text.trim().trim_matches('"').to_string())
        } else {
            Err(format!("Status {}", res.status()))
        }
    }

    pub async fn get_champ_select_session(&self) -> Result<Value, String> {
        let res = self
            .client
            .get(format!("{}/lol-champ-select/v1/session", self.base_url))
            .header("Authorization", &self.auth_header)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if res.status().is_success() {
            res.json::<Value>().await.map_err(|e| e.to_string())
        } else {
            Err(format!("Status {}", res.status()))
        }
    }

    pub async fn get_current_summoner(&self) -> Result<Value, String> {
        let res = self
            .client
            .get(format!("{}/lol-summoner/v1/current-summoner", self.base_url))
            .header("Authorization", &self.auth_header)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if res.status().is_success() {
            res.json::<Value>().await.map_err(|e| e.to_string())
        } else {
            Err(format!("Status {}", res.status()))
        }
    }

    pub async fn get_chat_conversation_participants(
        &self,
        room_id: &str,
    ) -> Result<Vec<Value>, String> {
        let res = self
            .client
            .get(format!(
                "{}/lol-chat/v1/conversations/{}/participants",
                self.base_url, room_id
            ))
            .header("Authorization", &self.auth_header)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if res.status().is_success() {
            res.json::<Vec<Value>>().await.map_err(|e| e.to_string())
        } else {
            Err(format!("Status {}", res.status()))
        }
    }

    pub async fn send_chat_message(
        &self,
        room_id: &str,
        message: &str,
    ) -> Result<bool, String> {
        let payload = json!({ "body": message, "type": "chat" });
        let res = self
            .client
            .post(format!(
                "{}/lol-chat/v1/conversations/{}/messages",
                self.base_url, room_id
            ))
            .header("Authorization", &self.auth_header)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        Ok(res.status().is_success())
    }

    pub async fn get_player_match_history(
        &self,
        puuid: &str,
        end_idx: usize,
    ) -> Result<Value, String> {
        let res = self
            .client
            .get(format!(
                "{}/lol-match-history/v1/products/lol/{}/matches?begIndex=0&endIndex={}",
                self.base_url, puuid, end_idx
            ))
            .header("Authorization", &self.auth_header)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if res.status().is_success() {
            res.json::<Value>().await.map_err(|e| e.to_string())
        } else {
            Err(format!("Status {}", res.status()))
        }
    }

    pub async fn get_match_detail(&self, game_id: i64) -> Result<Value, String> {
        let res = self
            .client
            .get(format!(
                "{}/lol-match-history/v1/games/{}",
                self.base_url, game_id
            ))
            .header("Authorization", &self.auth_header)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if res.status().is_success() {
            res.json::<Value>().await.map_err(|e| e.to_string())
        } else {
            Err(format!("Status {}", res.status()))
        }
    }

    pub async fn get_player_ranked_stats(&self, puuid: &str) -> Result<Value, String> {
        let res = self
            .client
            .get(format!(
                "{}/lol-ranked/v1/ranked-stats/{}",
                self.base_url, puuid
            ))
            .header("Authorization", &self.auth_header)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if res.status().is_success() {
            res.json::<Value>().await.map_err(|e| e.to_string())
        } else {
            Err(format!("Status {}", res.status()))
        }
    }

    pub async fn accept_ready_check(&self) -> Result<bool, String> {
        let res = self
            .client
            .post(format!(
                "{}/lol-matchmaking/v1/ready-check/accept",
                self.base_url
            ))
            .header("Authorization", &self.auth_header)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        Ok(res.status().is_success())
    }
}
