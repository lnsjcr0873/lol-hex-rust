use axum::extract::{Path as AxPath, Query, State};
use axum::http::{header, StatusCode, Uri};
use axum::response::{IntoResponse, Json, Response};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

use crate::config::{get_preset_definitions, ConfigManager};
use crate::data::loader::Database;
use crate::engine::player_history::{
    compute_radar, evaluate_match_participants, CachedPlayerRecord, MatchParticipantInput, OngoingMatchResponse,
    ParsedMatchRecord, PlayerHistoryManager,
};
use crate::lcu::LcuClient;
use crate::overlay::{BenchOverlay, InGameOverlay, ItemOverlay};
use crate::web::assets::StaticAssets;
use crate::web::ws::WsBroadcaster;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub lcu: Arc<LcuClient>,
    pub history: Arc<PlayerHistoryManager>,
    pub config: Arc<ConfigManager>,
    pub item_overlay: Arc<ItemOverlay>,
    pub ingame_overlay: Arc<InGameOverlay>,
    pub bench_overlay: Arc<BenchOverlay>,
    pub broadcaster: Arc<WsBroadcaster>,
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<usize>,
    pub queue_id: Option<i32>,
    #[serde(rename = "queueId")]
    pub queue_id_camel: Option<i32>,
    pub beg_index: Option<usize>,
    #[serde(rename = "begIndex")]
    pub beg_index_camel: Option<usize>,
    pub end_index: Option<usize>,
    #[serde(rename = "endIndex")]
    pub end_index_camel: Option<usize>,
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    #[serde(rename = "pageSize")]
    pub page_size_camel: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ChatRecordsPayload {
    pub chat_room_id: Option<String>,
    #[serde(rename = "chatRoomId")]
    pub chat_room_id_camel: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigUpdatePayload {
    #[serde(rename = "resolveConflicts")]
    pub resolve_conflicts_camel: Option<bool>,
    pub resolve_conflicts: Option<bool>,
    #[serde(rename = "hexSynergyWeight")]
    pub hex_synergy_weight_camel: Option<f64>,
    pub hex_synergy_weight: Option<f64>,
    #[serde(rename = "archetypeStrictness")]
    pub archetype_strictness_camel: Option<f64>,
    pub archetype_strictness: Option<f64>,
    #[serde(rename = "counterIntensity")]
    pub counter_intensity_camel: Option<f64>,
    pub counter_intensity: Option<f64>,
    #[serde(rename = "goldCalcMode")]
    pub gold_calc_mode_camel: Option<String>,
    pub gold_calc_mode: Option<String>,
    #[serde(rename = "trioMode")]
    pub trio_mode_camel: Option<String>,
    pub trio_mode: Option<String>,
    pub preset: Option<String>,
    #[serde(rename = "currentPreset")]
    pub current_preset_camel: Option<String>,
    #[serde(rename = "autoAcceptMatch")]
    pub auto_accept_match_camel: Option<bool>,
    pub auto_accept_match: Option<bool>,
    #[serde(rename = "autoAcceptDelaySeconds")]
    pub auto_accept_delay_seconds_camel: Option<u64>,
    pub auto_accept_delay_seconds: Option<u64>,
    #[serde(rename = "autoSwapBench")]
    pub auto_swap_bench_camel: Option<bool>,
    pub auto_swap_bench: Option<bool>,
    #[serde(rename = "autoApplyRunes")]
    pub auto_apply_runes_camel: Option<bool>,
    pub auto_apply_runes: Option<bool>,
    #[serde(rename = "benchPreferredChampions")]
    pub bench_preferred_champions_camel: Option<Vec<String>>,
    pub bench_preferred_champions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct AutoAcceptConfigPayload {
    pub enabled: Option<bool>,
    #[serde(rename = "autoAcceptMatch")]
    pub auto_accept_match: Option<bool>,
    #[serde(rename = "delaySeconds")]
    pub delay_seconds: Option<u64>,
    #[serde(rename = "autoAcceptDelaySeconds")]
    pub auto_accept_delay_seconds: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct OcrScanRequest {
    #[serde(rename = "heroId")]
    pub hero_id: Option<String>,
    #[serde(rename = "currentStage")]
    pub current_stage: Option<i32>,
    #[serde(rename = "existingAugments")]
    pub existing_augments: Option<Vec<String>>,
}

// -------------------------------------------------------------
// Router Factory
// -------------------------------------------------------------
pub fn create_router(state: AppState) -> axum::Router {
    axum::Router::new()
        // WebSocket
        .route("/ws/live", axum::routing::get(ws_handler))
        // Akari Match & Profile APIs
        .route("/api/summoner/current", axum::routing::get(get_current_summoner))
        .route("/api/match/ongoing", axum::routing::get(get_ongoing_match))
        .route("/api/match/mock-ongoing", axum::routing::get(get_mock_ongoing))
        .route("/api/match/history/:puuid", axum::routing::get(get_match_history))
        .route("/api/match/raw-history/:puuid", axum::routing::get(get_raw_match_history))
        .route("/api/player/champion-mastery/:puuid", axum::routing::get(get_player_champion_mastery))
        .route("/api/match/detail/:game_id", axum::routing::get(get_match_detail))
        .route("/api/match/raw-detail/:game_id", axum::routing::get(get_raw_match_detail))
        .route("/api/match/timeline/:game_id", axum::routing::get(get_match_timeline))
        .route("/api/match/raw-timeline/:game_id", axum::routing::get(get_raw_match_timeline))
        .route("/api/chat/send-records", axum::routing::post(send_chat_records))
        // Settings & Preference APIs
        .route("/api/config", axum::routing::get(get_config).post(update_config))
        // Auto-Accept Matchmaking APIs
        .route("/api/auto-accept/status", axum::routing::get(get_auto_accept_status))
        .route("/api/auto-accept/config", axum::routing::post(update_auto_accept_config))
        .route("/api/auto-accept/trigger", axum::routing::post(trigger_auto_accept))
        .route("/api/auto-accept/decline", axum::routing::post(trigger_auto_decline))
        .route(
            "/api/item-overlay/preferences",
            axum::routing::get(get_item_preferences).post(update_item_preferences),
        )
        .route("/api/item-overlay/status", axum::routing::get(get_item_overlay_status))
        .route("/api/item-overlay/config", axum::routing::post(update_item_overlay_config))
        .route("/api/item-overlay/trigger", axum::routing::post(trigger_item_overlay))
        .route("/api/overlay/status", axum::routing::get(get_overlay_status))
        .route("/api/overlay/config", axum::routing::post(update_overlay_config))
        .route("/api/overlay/trigger", axum::routing::post(trigger_overlay))
        .route("/api/overlay/mock", axum::routing::post(mock_overlay))
        .route("/api/overlay/bench/status", axum::routing::get(get_bench_overlay_status))
        .route("/api/overlay/bench/config", axum::routing::post(update_bench_overlay_config))
        .route("/api/champ-select/reroll", axum::routing::post(champ_select_reroll))
        .route("/api/champ-select/mock", axum::routing::post(champ_select_mock))
        .route("/api/champ-select/stop-mock", axum::routing::post(champ_select_stop_mock))
        // OCR Scan & Compare API
        .route("/api/ocr/scan-and-compare", axum::routing::post(scan_and_compare))
        // Data & Meta APIs
        .route("/api/meta", axum::routing::get(get_meta))
        .route("/api/heroes", axum::routing::get(get_heroes))
        .route("/api/augments", axum::routing::get(get_augments))
        .route("/api/hero/:id", axum::routing::get(get_hero_detail))
        .route("/api/augment/compare", axum::routing::post(compare_augments))
        .route("/api/team/synergy", axum::routing::post(team_synergy))
        .route("/api/license/status", axum::routing::get(get_license_status))
        .route("/api/license/activate", axum::routing::post(activate_license))
        // Static file fallback
        .fallback(static_asset_handler)
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin([
                    "http://127.0.0.1:8765".parse().unwrap(),
                    "http://localhost:8765".parse().unwrap(),
                    "http://127.0.0.1:3000".parse().unwrap(),
                    "http://localhost:3000".parse().unwrap(),
                    "http://127.0.0.1:5173".parse().unwrap(),
                    "http://localhost:5173".parse().unwrap(),
                    "tauri://localhost".parse().unwrap(),
                    "https://tauri.localhost".parse().unwrap(),
                ])
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::PUT,
                    axum::http::Method::DELETE,
                    axum::http::Method::OPTIONS,
                ])
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                    axum::http::header::ACCEPT,
                ])
        )
        .with_state(state)
}

// -------------------------------------------------------------
// WebSocket Handler
// -------------------------------------------------------------
async fn ws_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| crate::web::ws::handle_socket(socket, state.broadcaster))
}

// -------------------------------------------------------------
// Akari Realtime Match & Summoner APIs
// -------------------------------------------------------------
async fn get_current_summoner(State(state): State<AppState>) -> Json<Value> {
    if let Some(sum) = state.lcu.get_current_summoner().await {
        let puuid = sum.get("puuid").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let name = sum
            .get("displayName")
            .or_else(|| sum.get("gameName"))
            .and_then(|v| v.as_str())
            .unwrap_or("召唤师")
            .to_string();
        let tag_line = sum.get("tagLine").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let level = sum.get("summonerLevel").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let icon_id = sum.get("profileIconId").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

        let ranked = if !puuid.is_empty() {
            state.lcu.get_player_ranked_stats(&puuid).await
        } else {
            None
        };

        return Json(json!({
            "summoner": {
                "puuid": puuid,
                "name": name,
                "gameName": name,
                "tag_line": tag_line,
                "tagLine": tag_line,
                "level": level,
                "profile_icon_id": icon_id,
                "profileIconId": icon_id
            },
            "ranked": ranked.unwrap_or(json!({
                "solo": {
                    "tier": "UNRANKED",
                    "division": "",
                    "tier_str": "未定级",
                    "lp": 0,
                    "wins": 0,
                    "losses": 0,
                    "win_rate": 0.0
                },
                "flex": {
                    "tier": "UNRANKED",
                    "division": "",
                    "tier_str": "未定级",
                    "lp": 0,
                    "wins": 0,
                    "losses": 0,
                    "win_rate": 0.0
                }
            }))
        }));
    }

    // When LCU is disconnected, return explicit disconnected state
    Json(json!({
        "is_connected": false,
        "summoner": null,
        "ranked": null,
        "message": "英雄联盟客户端未连接（请先启动游戏并登录）"
    }))
}

async fn get_ongoing_match(State(state): State<AppState>) -> Json<OngoingMatchResponse> {
    let resp = state.history.get_ongoing_match(&state.lcu, &state.db).await;
    Json(resp)
}

async fn get_mock_ongoing(State(state): State<AppState>) -> Json<OngoingMatchResponse> {
    let resp = state.history.get_mock_ongoing_match(&state.db);
    Json(resp)
}

async fn get_player_champion_mastery(
    AxPath(puuid): AxPath<String>,
    State(state): State<AppState>,
) -> Json<Value> {
    let target = if puuid == "me" || puuid == "current" {
        String::new()
    } else {
        puuid
    };
    if let Some(val) = state.lcu.get_champion_mastery(&target).await {
        return Json(val);
    }
    Json(json!([]))
}

/// Helper to paginate matches safely without secondary truncation when the input slice is already paged.
pub fn paginate_matches<T: Clone>(matches: &[T], resolved_beg: usize, resolved_end: usize) -> Vec<T> {
    let count = (resolved_end + 1).saturating_sub(resolved_beg);
    if matches.len() >= resolved_beg + count {
        matches.iter().skip(resolved_beg).take(count).cloned().collect()
    } else if matches.len() <= count {
        matches.iter().take(count).cloned().collect()
    } else {
        let skip_count = resolved_beg.min(matches.len().saturating_sub(count));
        matches.iter().skip(skip_count).take(count).cloned().collect()
    }
}

async fn get_match_history(
    AxPath(puuid): AxPath<String>,
    Query(query): Query<HistoryQuery>,
    State(state): State<AppState>,
) -> Json<Value> {
    let queue_id = query.queue_id.or(query.queue_id_camel).unwrap_or(0);
    let beg_idx = query.beg_index.or(query.beg_index_camel);
    let end_idx = query.end_index.or(query.end_index_camel);
    let p_size = query.page_size.or(query.page_size_camel).or(query.limit);

    let (resolved_beg, resolved_end) = match (beg_idx, end_idx) {
        (Some(b), Some(e)) => (b, e.max(b)),
        (Some(b), None) => {
            let count = p_size.unwrap_or(20).clamp(1, 200);
            (b, b + count.saturating_sub(1))
        }
        (None, Some(e)) => (0, e),
        (None, None) => {
            let p = query.page.unwrap_or(1).max(1);
            let ps = p_size.unwrap_or(20).clamp(1, 200);
            let b = (p - 1) * ps;
            let e = b + ps - 1;
            (b, e)
        }
    };
    let resolved_end = resolved_end.min(resolved_beg + 199);

    let mut query_name = String::new();
    let mut resolved_puuid = if puuid == "me" || puuid == "current" {
        if let Some(sum) = state.lcu.get_current_summoner().await {
            sum.get("puuid").and_then(|v| v.as_str()).unwrap_or("").to_string()
        } else {
            String::new()
        }
    } else {
        puuid.clone()
    };

    // A Riot PUUID is 78 hex characters (modern CN/global LCU), a 36-char UUID with hyphens, 32-char hex, or mock_puuid_*
    let looks_like_puuid = resolved_puuid.starts_with("mock_puuid_")
        || (resolved_puuid.len() >= 32 && resolved_puuid.chars().all(|c| c.is_ascii_hexdigit() || c == '-'));
    if !looks_like_puuid {
        query_name = resolved_puuid.clone();
        if let Some(sum) = state.lcu.get_summoner_by_name(&query_name).await {
            if let Some(p) = sum.get("puuid").and_then(|v| v.as_str()) {
                resolved_puuid = p.to_string();
            }
        }
    }

    // Dynamic fetch with cache fallback:
    let full_record = if let Some(rec) = state.history.fetch_and_cache_player(&resolved_puuid, &query_name, &state.lcu, resolved_beg, resolved_end).await {
        rec
    } else if let Some(cached) = state.history.get_player_data_fresh(&resolved_puuid, std::time::Duration::from_secs(600)).await {
        cached
    } else if resolved_puuid.starts_with("mock_puuid_") {
        let parts: Vec<&str> = resolved_puuid.trim_start_matches("mock_puuid_").split('_').collect();
        let slot = parts.last().and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
        let mock_pool = [266, 64, 517, 145, 111];
        let mut mock = state.history.build_mock_player_profile(slot, mock_pool[slot % mock_pool.len()]);
        if !query_name.is_empty() {
            mock.summoner_name = query_name.clone();
            mock.raw_summoner["name"] = json!(query_name);
            mock.raw_summoner["gameName"] = json!(query_name);
        }
        mock
    } else {
        CachedPlayerRecord {
            puuid: resolved_puuid.clone(),
            summoner_name: if !query_name.is_empty() { query_name.clone() } else { "未连接客户端".to_string() },
            tier: "UNRANKED".to_string(),
            tier_str: "未定级".to_string(),
            tier_color: "#94a3b8".to_string(),
            lp: 0,
            ranked_wins: 0,
            ranked_losses: 0,
            matches: Vec::new(),
            fetched_at: std::time::Instant::now(),
            raw_summoner: json!({}),
            raw_ranked: json!({}),
        }
    };

    // 1. Filter by queue_id if specified (> 0)
    let queue_matches: Vec<ParsedMatchRecord> = if queue_id > 0 {
        full_record.matches.iter().filter(|m| m.queue_id == queue_id).cloned().collect()
    } else {
        full_record.matches.clone()
    };

    // 2. Paginated slice for the matches table/cards
    let paginated_matches: Vec<ParsedMatchRecord> = paginate_matches(&queue_matches, resolved_beg, resolved_end);

    // 3. Tactical profile evaluation MUST always evaluate against the player's full, globally newest merged match history
    let first_cid = queue_matches.first().map(|m| m.champion_id).unwrap_or(0);
    let mut rec_for_eval = full_record.clone();
    rec_for_eval.matches = queue_matches.clone();
    let mut tactical = state.history.compute_tactical_profile(&rec_for_eval, first_cid, 20);
    let radar = compute_radar(&queue_matches.iter().take(20).cloned().collect::<Vec<_>>());
    tactical.radar = radar.clone();

    Json(json!({
        "summoner": full_record.raw_summoner,
        "ranked": full_record.raw_ranked,
        "analysis": {
            "recent_win_rate_pct": tactical.recent_win_rate_pct,
            "recent_wins": tactical.recent_wins,
            "recent_losses": tactical.recent_losses,
            "avg_kda_ratio": tactical.avg_kda_ratio,
            "avg_kda_str": tactical.avg_kda_str,
            "top_champs": tactical.top_champs,
            "radar": radar,
            "badges": tactical.badges
        },
        "matches": paginated_matches
    }))
}

async fn get_match_detail(
    AxPath(game_id): AxPath<String>,
    State(state): State<AppState>,
) -> Json<Value> {
    // 1. Try LCU Match History Game Details
    if let Some(raw) = state.lcu.get_match_detail(&game_id).await {
        if let Some(parsed) = parse_riot_match_detail(&raw, &state.db) {
            return Json(parsed);
        }
    }

    // 2. Only return demo mock if explicitly requested with negative ID or demo ID
    let gid = game_id.parse::<i64>().unwrap_or(0);
    if gid < 0 || game_id.starts_with('-') || gid == 99991234 {
        let mock = build_mock_match_detail(gid, &state.db);
        return Json(mock);
    }

    Json(json!({ "error": "未能从客户端获取该场对局详情" }))
}

async fn get_match_timeline(
    AxPath(game_id): AxPath<String>,
    State(state): State<AppState>,
) -> Json<Value> {
    let gid = game_id.parse::<u64>().unwrap_or(0);
    if gid > 0 {
        if let Some(timeline) = state.lcu.get_game_timeline(gid).await {
            return Json(timeline);
        }
    }

    if gid == 0 || game_id.starts_with('-') || gid == 99991234 || gid == 100200300 {
        let mock = build_mock_match_timeline(gid);
        return Json(mock);
    }

    Json(json!({ "error": "未能从客户端获取该场对局时间轴" }))
}

async fn get_raw_match_history(
    AxPath(puuid): AxPath<String>,
    Query(query): Query<HistoryQuery>,
    State(state): State<AppState>,
) -> Json<Value> {
    let current_sum = state.lcu.get_current_summoner().await;
    let target_puuid = if puuid == "me" || puuid == "current" {
        current_sum
            .as_ref()
            .and_then(|s| s.get("puuid"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    } else {
        puuid
    };

    if target_puuid.is_empty() {
        return Json(json!({
            "accountId": 0,
            "games": {
                "games": []
            }
        }));
    }

    let b_idx = query.beg_index.or(query.beg_index_camel).unwrap_or(0);
    let p_size = query.page_size.or(query.page_size_camel).or(query.limit).unwrap_or(20).clamp(1, 200);
    let e_idx = query.end_index.or(query.end_index_camel).unwrap_or_else(|| b_idx.saturating_add(p_size).saturating_sub(1)).max(b_idx);
    let queue_id = query.queue_id.or(query.queue_id_camel);

    if let Some(mut raw_history) = state.lcu.get_raw_match_history(&target_puuid, b_idx, e_idx).await {
        if let Some(qid) = queue_id {
            if qid > 0 {
                let filter_fn = |arr: &mut Vec<Value>| {
                    arr.retain(|game| {
                        game.get("queueId").and_then(|q| q.as_i64()).map(|q| q == qid as i64).unwrap_or(false)
                    });
                };

                if let Some(g_val) = raw_history.get_mut("games") {
                    if let Some(arr) = g_val.as_array_mut() {
                        filter_fn(arr);
                    } else if let Some(arr) = g_val.get_mut("games").and_then(|g| g.as_array_mut()) {
                        filter_fn(arr);
                    }
                } else if let Some(arr) = raw_history.as_array_mut() {
                    filter_fn(arr);
                }
            }
        }
        return Json(raw_history);
    }

    Json(json!({ "error": "无法从客户端获取原始战绩数据" }))
}

async fn get_raw_match_timeline(
    AxPath(game_id): AxPath<String>,
    State(state): State<AppState>,
) -> Json<Value> {
    let gid = game_id.parse::<u64>().unwrap_or(0);
    if gid > 0 {
        if let Some(timeline) = state.lcu.get_raw_match_timeline(gid).await {
            return Json(timeline);
        }
    }

    if gid == 0 || game_id.starts_with('-') || gid == 99991234 || gid == 100200300 {
        let mock = build_mock_match_timeline(gid);
        return Json(mock);
    }

    Json(json!({ "error": "无法获取原始时间线数据" }))
}

async fn get_raw_match_detail(
    AxPath(game_id): AxPath<String>,
    State(state): State<AppState>,
) -> Json<Value> {
    let gid = game_id.parse::<u64>().unwrap_or(0);
    if gid > 0 {
        if let Some(detail) = state.lcu.get_raw_match_detail(gid).await {
            return Json(detail);
        }
    }

    if gid == 0 || game_id.starts_with('-') || gid == 99991234 || gid == 100200300 {
        let mock = build_mock_match_detail(gid as i64, &state.db);
        return Json(mock);
    }

    Json(json!({ "error": "无法获取原始详情数据" }))
}

pub fn spell_meta(spell_id: i32) -> (&'static str, &'static str) {
    match spell_id {
        1 => ("净化", "https://game.gtimg.cn/images/lol/act/img/spell/SummonerBoost.png"),
        3 => ("虚弱", "https://game.gtimg.cn/images/lol/act/img/spell/SummonerExhaust.png"),
        4 => ("闪现", "https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png"),
        6 => ("幽灵疾步", "https://game.gtimg.cn/images/lol/act/img/spell/SummonerHaste.png"),
        7 => ("治疗术", "https://game.gtimg.cn/images/lol/act/img/spell/SummonerHeal.png"),
        11 | 54 | 55 => ("惩戒", "https://game.gtimg.cn/images/lol/act/img/spell/SummonerSmite.png"),
        12 => ("传送", "https://game.gtimg.cn/images/lol/act/img/spell/SummonerTeleport.png"),
        13 => ("清晰术", "https://game.gtimg.cn/images/lol/act/img/spell/SummonerMana.png"),
        14 => ("引燃", "https://game.gtimg.cn/images/lol/act/img/spell/SummonerDot.png"),
        21 => ("屏障", "https://game.gtimg.cn/images/lol/act/img/spell/SummonerBarrier.png"),
        30 => ("王者魄罗", "https://game.gtimg.cn/images/lol/act/img/spell/SummonerPoroRecall.png"),
        31 => ("魄罗投掷", "https://game.gtimg.cn/images/lol/act/img/spell/SummonerPoroThrow.png"),
        32 | 39 => ("雪球", "https://game.gtimg.cn/images/lol/act/img/spell/SummonerSnowball.png"),
        2201 => ("逃跑", "https://game.gtimg.cn/images/lol/act/img/spell/SummonerCherryFlee.png"),
        2202 => ("闪现", "https://game.gtimg.cn/images/lol/act/img/spell/SummonerCherryFlash.png"),
        _ => ("召唤师技能", "https://game.gtimg.cn/images/lol/act/img/spell/SummonerFlash.png"),
    }
}

#[allow(dead_code)]
pub fn convert_ongoing_card(card: &crate::engine::player_history::OngoingPlayerCard, team_str: &'static str, db: &Database) -> Value {
    let (s1_name, s1_icon) = spell_meta(card.spell1_id);
    let (s2_name, s2_icon) = spell_meta(card.spell2_id);
    let cname = if card.champion_id > 0 {
        db.resolve_hero_name(&card.champion_id.to_string())
            .unwrap_or_else(|| format!("英雄{}", card.champion_id))
    } else {
        "选择中...".to_string()
    };
    let cicon = if card.champion_id > 0 {
        format!("https://game.gtimg.cn/images/lol/act/img/skinloading/{}000.jpg", card.champion_id)
    } else {
        "https://game.gtimg.cn/images/lol/act/img/skinloading/142000.jpg".to_string()
    };

    let badges_json: Vec<Value> = card
        .tactical_analysis
        .badges
        .iter()
        .map(|b| {
            let color = if b.fg.contains("38bdf8") || b.fg.contains("cyan") {
                "cyan"
            } else if b.fg.contains("f87171") || b.fg.contains("red") {
                "red"
            } else if b.fg.contains("fbbf24") || b.fg.contains("gold") || b.fg.contains("amber") {
                "gold"
            } else if b.fg.contains("a5b4fc") || b.fg.contains("purple") {
                "purple"
            } else {
                "blue"
            };
            json!({
                "id": format!("b_{}_{}", card.slot_index, b.text),
                "label": b.text,
                "icon": if b.text.contains("连胜") { "🔥" } else if b.text.contains("连败") { "❄️" } else if b.text.contains("绝活") { "👑" } else if b.text.contains("通天代") { "⚡" } else if b.text.contains("车队") { "🚗" } else { "🎯" },
                "color": color,
                "description": b.desc
            })
        })
        .collect();

    let premade_party = card.premade_group.map(|gid| {
        json!({
            "party_id": gid,
            "label": card.premade_tag.clone().unwrap_or_else(|| "🚗 车队".to_string()),
            "color_class": if gid == 1 { "party-border-pink" } else { "party-border-yellow" }
        })
    });

    let recent_matches: Vec<&'static str> = card
        .tactical_analysis
        .recent_dots
        .iter()
        .map(|&w| if w { "win" } else { "loss" })
        .collect();
    let radar = &card.tactical_analysis.radar;

    json!({
        "puuid": card.puuid,
        "summoner_id": card.puuid,
        "game_name": if !card.summoner_name.is_empty() { card.summoner_name.clone() } else { cname.clone() },
        "tag_line": card.full_name.split('#').nth(1).filter(|s| !s.is_empty()).unwrap_or(""),
        "profile_icon_id": card.profile_icon_id,
        "summoner_level": card.level,
        "team": team_str,
        "cell_id": card.slot_index,
        "assigned_position": card.position,
        "champion_id": card.champion_id,
        "champion_name": cname,
        "champion_title": "",
        "champion_icon": cicon,
        "is_locked": card.champion_id > 0,
        "spell1_id": card.spell1_id,
        "spell1_name": s1_name,
        "spell1_icon": s1_icon,
        "spell2_id": card.spell2_id,
        "spell2_name": s2_name,
        "spell2_icon": s2_icon,
        "tier": card.tactical_analysis.tier,
        "division": card.tactical_analysis.division,
        "lp": card.tactical_analysis.lp,
        "recent_kda": card.tactical_analysis.avg_kda_str,
        "recent_kda_ratio": card.tactical_analysis.avg_kda_ratio,
        "recent_win_rate": card.tactical_analysis.recent_win_rate_pct,
        "recent_games_count": card.tactical_analysis.recent_count,
        "champ_recent_games": card.tactical_analysis.cur_champ_games,
        "champ_recent_win_rate": card.tactical_analysis.cur_champ_win_rate_pct,
        "top_champs": card.tactical_analysis.top_champs.iter().map(|tc| {
            json!({
                "champion_id": tc.champion_id,
                "champion_name": tc.champion_name,
                "games": tc.games,
                "win_rate": tc.win_rate,
                "win_rate_pct": tc.win_rate_pct,
                "kda": tc.kda,
            })
        }).collect::<Vec<Value>>(),
        "recent_matches": recent_matches,
        "matches": card.matches.iter().take(20).map(|m| {
            json!({
                "game_id": m.game_id,
                "game_creation": m.game_creation,
                "game_duration": m.game_duration,
                "game_duration_str": m.game_duration_str,
                "queue_id": m.queue_id,
                "mode_cn": m.mode_cn,
                "champion_id": m.champion_id,
                "champion_name": m.champion_name,
                "champion_avatar": m.champion_avatar,
                "win": m.win,
                "is_win": m.win,
                "kills": m.kills,
                "deaths": m.deaths,
                "assists": m.assists,
                "kda_str": m.kda_str,
                "kda_ratio": m.kda_ratio,
                "cs": m.cs,
                "cs_per_min": m.cs_per_min,
                "damage": m.damage,
                "damage_share_pct": m.damage_share_pct,
                "damage_taken": m.damage_taken,
                "damage_mitigated": m.damage_mitigated,
                "gold": m.gold,
                "vision_score": m.vision_score,
                "kp_pct": m.kp_pct,
                "is_mvp": m.is_mvp,
                "is_svp": m.is_svp,
                "lane": m.lane,
                "role": m.role,
                "role_cn": m.role_cn,
                "spell1_id": m.spell1_id,
                "spell2_id": m.spell2_id,
                "items": m.items,
                "all_items": m.all_items
            })
        }).collect::<Vec<Value>>(),
        "badges": badges_json,
        "premade_party": premade_party,
        "radar": {
            "output": radar.get("damage").copied().unwrap_or(0),
            "kda": radar.get("kda").copied().unwrap_or(0),
            "join_rate": radar.get("kp").copied().unwrap_or(0),
            "survival": radar.get("survival").copied().unwrap_or(0),
            "gold": radar.get("economy").copied().unwrap_or(0),
            "vision": radar.get("vision").copied().unwrap_or(0)
        }
    })
}

pub fn build_chat_broadcast_lines(
    my_team: &[crate::engine::player_history::OngoingPlayerCard],
    format: Option<&str>,
) -> Vec<String> {
    let mut lines = Vec::new();
    if format == Some("concise") {
        lines.push("⚡【HexBrain 战绩简报】".to_string());
        let mut summaries = Vec::new();
        for p in my_team.iter().take(5) {
            let pos = match p.position.to_uppercase().as_str() {
                "TOP" => "上",
                "JUNGLE" => "野",
                "MID" | "MIDDLE" => "中",
                "BOTTOM" | "BOT" | "ADC" => "下",
                "UTILITY" | "SUPPORT" => "辅",
                _ => "",
            };
            let cname = if !p.champion_name.is_empty() && p.champion_name != "选择中..." {
                p.champion_name.clone()
            } else {
                p.summoner_name.clone()
            };
            let prefix = if !pos.is_empty() { format!("{}:", pos) } else { String::new() };
            if p.tactical_analysis.recent_count > 0 {
                summaries.push(format!("{}{}({}%胜)", prefix, cname, p.tactical_analysis.recent_win_rate_pct));
            } else {
                summaries.push(format!("{}{}", prefix, cname));
            }
        }
        lines.push(summaries.join(" | "));
    } else {
        lines.push("⚡【HexBrain 智脑战绩透视】⚡".to_string());

        for (idx, p) in my_team.iter().take(5).enumerate() {
            let pos = match p.position.to_uppercase().as_str() {
                "TOP" => "上路",
                "JUNGLE" => "打野",
                "MID" | "MIDDLE" => "中路",
                "BOTTOM" | "BOT" | "ADC" => "射手",
                "UTILITY" | "SUPPORT" => "辅助",
                _ => "",
            };
            let pos_str = if !pos.is_empty() { format!("{} ", pos) } else { String::new() };
            let wr = p.tactical_analysis.recent_win_rate_pct;
            let tag = p.premade_tag.as_deref().unwrap_or("");
            let tag_suffix = if !tag.is_empty() { format!(" [{}]", tag) } else { String::new() };
            lines.push(format!(
                "{}. {}[{}] {} (近{}场胜率{}%){}",
                idx + 1,
                pos_str,
                p.summoner_name,
                p.tactical_analysis.tier_str,
                p.tactical_analysis.recent_count,
                wr,
                tag_suffix
            ));
        }
    }
    lines
}

fn resolve_participant_role(p: &Value, pid: i64) -> String {
    let lane = p.get("timeline").and_then(|t| t.get("lane")).and_then(|v| v.as_str()).unwrap_or("");
    let timeline_role = p.get("timeline").and_then(|t| t.get("role")).and_then(|v| v.as_str()).unwrap_or("");
    let assigned = p.get("assignedPosition").and_then(|v| v.as_str()).unwrap_or("");

    match (assigned.to_uppercase().as_str(), lane.to_uppercase().as_str(), timeline_role) {
        ("TOP", _, _) | (_, "TOP", _) => "TOP",
        ("JUNGLE", _, _) | (_, "JUNGLE", _) => "JUNGLE",
        ("MID" | "MIDDLE", _, _) | (_, "MIDDLE" | "MID", _) => "MIDDLE",
        ("BOTTOM" | "BOT", _, _) | (_, "BOTTOM", "DUO_CARRY") => "BOTTOM",
        ("UTILITY" | "SUPPORT", _, _) | (_, "BOTTOM", "DUO_SUPPORT") => "UTILITY",
        (_, "BOTTOM", _) => "BOTTOM",
        _ => match (pid.max(1) - 1) % 5 {
            0 => "TOP",
            1 => "JUNGLE",
            2 => "MIDDLE",
            3 => "BOTTOM",
            _ => "UTILITY",
        },
    }.to_string()
}

pub fn parse_riot_match_detail(raw: &Value, db: &Database) -> Option<Value> {
    let game_id = raw.get("gameId")
        .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok())))
        .or_else(|| raw.get("id").and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))))?;
    let game_creation = raw.get("gameCreation").and_then(|v| v.as_i64()).unwrap_or(0);
    let raw_duration = raw.get("gameDuration").and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f as i64))).unwrap_or(0).max(0);
    let start_ts = raw.get("gameStartTimestamp").and_then(|v| v.as_i64()).unwrap_or(0);
    let end_ts = raw.get("gameEndTimestamp").and_then(|v| v.as_i64()).unwrap_or(0);
    let game_duration = crate::engine::player_history::normalize_game_duration(raw_duration, start_ts, end_ts);
    let game_duration_str = format!("{}:{:02}", game_duration / 60, game_duration % 60);
    let queue_id = raw.get("queueId").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let game_mode = raw.get("gameMode").and_then(|v| v.as_str()).unwrap_or("CLASSIC").to_string();

    let mode_cn = match queue_id {
        420 => "排位赛(单双)",
        440 => "排位赛(灵活)",
        450 => "狂暴大乱斗",
        400 => "匹配模式",
        1700 | 1710 => "斗魂竞技场",
        490 => "快速对局",
        1090 | 1100 => "云顶之弈",
        _ => if game_mode == "ARAM" { "狂暴大乱斗" } else { "匹配对局" },
    };
    let game_mode_cn = match queue_id {
        420 | 440 | 400 | 490 => "召唤师峡谷",
        450 => "极地大乱斗",
        1700 | 1710 => "斗魂竞技场",
        _ => if game_mode == "ARAM" { "极地大乱斗" } else { "召唤师峡谷" },
    };

    let p_identities = raw.get("participantIdentities").and_then(|v| v.as_array());
    let participants = raw.get("participants").and_then(|v| v.as_array())?;

    let mut name_map = HashMap::new();
    if let Some(identities) = p_identities {
        for pi in identities {
            let pid = pi.get("participantId").and_then(|v| v.as_i64()).unwrap_or(0);
            let player = pi.get("player");
            let name = player
                .and_then(|p| p.get("summonerName").or_else(|| p.get("gameName")))
                .and_then(|v| v.as_str())
                .unwrap_or("召唤师");
            let puuid = player.and_then(|p| p.get("puuid")).and_then(|v| v.as_str()).unwrap_or("");
            let tag_line = player.and_then(|p| p.get("tagLine")).and_then(|v| v.as_str()).unwrap_or("");
            name_map.insert(pid, (name.to_string(), puuid.to_string(), tag_line.to_string()));
        }
    }

    let mut blue_turrets = 0i32;
    let mut blue_dragons = 0i32;
    let mut blue_barons = 0i32;
    let mut blue_win_flag = None;

    let mut red_turrets = 0i32;
    let mut red_dragons = 0i32;
    let mut red_barons = 0i32;
    let mut red_win_flag = None;

    if let Some(teams_arr) = raw.get("teams").and_then(|v| v.as_array()) {
        for t in teams_arr {
            let tid = t.get("teamId").and_then(|v| v.as_i64()).unwrap_or(0);
            let win = t.get("win").map(|w| {
                w.as_bool().unwrap_or_else(|| w.as_str() == Some("Win") || w.as_i64() == Some(1))
            }).unwrap_or(false);
            let turrets = t.get("towerKills").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let dragons = t.get("dragonKills").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let barons = t.get("baronKills").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

            if tid == 100 {
                blue_turrets = turrets;
                blue_dragons = dragons;
                blue_barons = barons;
                blue_win_flag = Some(win);
            } else if tid == 200 {
                red_turrets = turrets;
                red_dragons = dragons;
                red_barons = barons;
                red_win_flag = Some(win);
            }
        }
    }

    struct RawParsedP {
        pid: i64,
        team_id: i64,
        win: bool,
        puuid: String,
        summoner_name: String,
        tag_line: String,
        team_str: &'static str,
        champion_id: i32,
        champion_name: String,
        champion_level: i32,
        spell1_id: i32,
        spell2_id: i32,
        spell1_icon: &'static str,
        spell2_icon: &'static str,
        kills: i32,
        deaths: i32,
        assists: i32,
        kda_ratio: f64,
        kda_str: String,
        total_damage: i64,
        total_damage_taken: i64,
        damage_mitigated: i64,
        gold_earned: i64,
        minions_killed: i32,
        items: Vec<i32>,
        role: String,
        tier: String,
        vision_score: i32,
    }

    let mut raw_players = Vec::new();
    let mut blue_participant_win = false;

    for p in participants {
        let team_id = p.get("teamId").and_then(|v| v.as_i64()).unwrap_or(0);
        let pid = p.get("participantId").and_then(|v| v.as_i64()).unwrap_or(0);
        let cid = p.get("championId").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let cname = db.resolve_hero_name(&cid.to_string()).unwrap_or_else(|| format!("英雄{}", cid));
        let (raw_sname, mut puuid, mut tag_line) = name_map.get(&pid).cloned().unwrap_or_default();
        let sname = if !raw_sname.is_empty() {
            raw_sname
        } else if let Some(n) = p.get("summonerName").or_else(|| p.get("gameName")).and_then(|v| v.as_str()) {
            n.to_string()
        } else {
            format!("玩家{}", pid)
        };
        if puuid.is_empty() {
            if let Some(p_id) = p.get("puuid").and_then(|v| v.as_str()) {
                puuid = p_id.to_string();
            }
        }
        if tag_line.is_empty() {
            if let Some(t) = p.get("tagLine").and_then(|v| v.as_str()) {
                tag_line = t.to_string();
            }
        }

        let stats = p.get("stats");
        let win = stats.and_then(|s| s.get("win")).map(|w| {
            w.as_bool().unwrap_or_else(|| w.as_str() == Some("Win") || w.as_i64() == Some(1))
        }).unwrap_or(false);

        if team_id == 100 && win {
            blue_participant_win = true;
        }

        let k = stats.and_then(|s| s.get("kills")).and_then(|v| v.as_i64()).unwrap_or(0).max(0) as i32;
        let d = stats.and_then(|s| s.get("deaths")).and_then(|v| v.as_i64()).unwrap_or(0).max(0) as i32;
        let a = stats.and_then(|s| s.get("assists")).and_then(|v| v.as_i64()).unwrap_or(0).max(0) as i32;
        let lvl = stats.and_then(|s| s.get("champLevel")).and_then(|v| v.as_i64()).unwrap_or(1).clamp(1, 30) as i32;
        let dmg = stats.and_then(|s| s.get("totalDamageDealtToChampions")).and_then(|v| v.as_i64()).unwrap_or(0).max(0);
        let taken = stats.and_then(|s| s.get("totalDamageTaken")).and_then(|v| v.as_i64()).unwrap_or(0).max(0);
        let mitig = stats.and_then(|s| s.get("damageSelfMitigated")).and_then(|v| v.as_i64()).unwrap_or(0).max(0);
        let gold = stats.and_then(|s| s.get("goldEarned")).and_then(|v| v.as_i64()).unwrap_or(0).max(0);
        let cs = (stats.and_then(|s| s.get("totalMinionsKilled")).and_then(|v| v.as_i64()).unwrap_or(0).max(0)
            + stats.and_then(|s| s.get("neutralMinionsKilled")).and_then(|v| v.as_i64()).unwrap_or(0).max(0)) as i32;
        let vs = stats.and_then(|s| s.get("visionScore")).and_then(|v| v.as_i64()).unwrap_or(0).max(0) as i32;

        let s1 = p.get("spell1Id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let s2 = p.get("spell2Id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let (_, spell1_icon) = spell_meta(s1);
        let (_, spell2_icon) = spell_meta(s2);

        let mut items = Vec::new();
        for i in 0..7 {
            let item_id = stats.and_then(|s| s.get(format!("item{}", i))).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            items.push(item_id);
        }

        let kda_ratio = if d > 0 { ((k + a) as f64 / d as f64 * 10.0).round() / 10.0 } else { (k + a) as f64 };
        let kda_str = format!("{:.1}", kda_ratio);

        let role = resolve_participant_role(p, pid);
        let tier = p.get("highestAchievedSeasonTier")
            .and_then(|v| v.as_str())
            .unwrap_or("UNRANKED")
            .to_string();

        let team_str = if team_id == 100 { "blue" } else { "red" };

        raw_players.push(RawParsedP {
            pid,
            team_id,
            win,
            puuid,
            summoner_name: sname,
            tag_line,
            team_str,
            champion_id: cid,
            champion_name: cname,
            champion_level: lvl,
            spell1_id: s1,
            spell2_id: s2,
            spell1_icon,
            spell2_icon,
            kills: k,
            deaths: d,
            assists: a,
            kda_ratio,
            kda_str,
            total_damage: dmg,
            total_damage_taken: taken,
            damage_mitigated: mitig,
            gold_earned: gold,
            minions_killed: cs,
            items,
            role,
            tier,
            vision_score: vs,
        });
    }

    let final_blue_win = blue_win_flag.unwrap_or(blue_participant_win);
    let final_red_win = red_win_flag.unwrap_or(!final_blue_win);

    // In ARAM or special modes where dragons/barons don't exist, ensure 0
    if queue_id == 450 || queue_id == 860 {
        blue_dragons = 0;
        red_dragons = 0;
        blue_barons = 0;
        red_barons = 0;
    }

    let eval_inputs: Vec<MatchParticipantInput> = raw_players.iter().map(|p| {
        let is_win = if queue_id == 1700 || queue_id == 1710 || (p.team_id != 100 && p.team_id != 200) {
            p.win
        } else if p.team_id == 100 {
            final_blue_win
        } else {
            final_red_win
        };
        MatchParticipantInput {
            participant_id: p.pid,
            puuid: p.puuid.clone(),
            team_id: p.team_id,
            is_win,
            kills: p.kills,
            deaths: p.deaths,
            assists: p.assists,
            damage: p.total_damage,
            damage_taken: p.total_damage_taken,
            damage_mitigated: p.damage_mitigated,
            gold: p.gold_earned,
            cs: p.minions_killed,
            vision_score: p.vision_score,
        }
    }).collect();

    let evaluations = evaluate_match_participants(game_duration, &eval_inputs);

    let mut participants_json = Vec::new();
    let mut blue_players = Vec::new();
    let mut red_players = Vec::new();
    let mut blue_kills = 0i32;
    let mut red_kills = 0i32;
    let mut blue_dmg_total = 0i64;
    let mut red_dmg_total = 0i64;
    let mut blue_gold_total = 0i64;
    let mut red_gold_total = 0i64;

    for (idx, p) in raw_players.into_iter().enumerate() {
        let ev = &evaluations[idx];

        if p.team_id == 100 {
            blue_kills += p.kills;
            blue_dmg_total += p.total_damage;
            blue_gold_total += p.gold_earned;
        } else {
            red_kills += p.kills;
            red_dmg_total += p.total_damage;
            red_gold_total += p.gold_earned;
        }

        let item_trinket = p.items.get(6).copied().unwrap_or(0);
        let cicon = if p.champion_id > 0 {
            format!("https://game.gtimg.cn/images/lol/act/img/skinloading/{}000.jpg", p.champion_id)
        } else {
            "https://game.gtimg.cn/images/lol/act/img/skinloading/142000.jpg".to_string()
        };

        let p_val = json!({
            "participant_id": p.pid,
            "team_id": p.team_id,
            "puuid": p.puuid,
            "summoner_name": p.summoner_name,
            "game_name": p.summoner_name,
            "tag_line": p.tag_line,
            "team": p.team_str,
            "champion_id": p.champion_id,
            "champion_name": p.champion_name,
            "champion_title": "",
            "champion_icon": cicon,
            "champion_level": p.champion_level,
            "role": p.role,
            "spell1_id": p.spell1_id,
            "spell1_icon": p.spell1_icon,
            "spell2_id": p.spell2_id,
            "spell2_icon": p.spell2_icon,
            "tier": p.tier,
            "kills": p.kills,
            "deaths": p.deaths,
            "assists": p.assists,
            "kda_str": p.kda_str,
            "kda_ratio": p.kda_ratio,
            "is_mvp": ev.is_mvp,
            "is_svp": ev.is_svp,
            "total_damage": p.total_damage,
            "damage": p.total_damage,
            "damage_share_pct": ev.damage_share_pct,
            "is_highest_damage": ev.is_highest_damage,
            "total_damage_taken": p.total_damage_taken,
            "damage_taken": p.total_damage_taken,
            "damage_taken_share_pct": ev.damage_taken_share_pct,
            "damage_mitigated": p.damage_mitigated,
            "gold_earned": p.gold_earned,
            "gold": p.gold_earned,
            "minions_killed": p.minions_killed,
            "cs": p.minions_killed,
            "items": p.items,
            "item_trinket": item_trinket,
            "vision_score": p.vision_score,
            "visionScore": p.vision_score,
            "badges": []
        });

        if p.team_id == 100 {
            blue_players.push(p_val.clone());
        } else {
            red_players.push(p_val.clone());
        }
        participants_json.push(p_val);
    }

    Some(json!({
        "game_id": game_id,
        "game_creation": game_creation,
        "game_duration": game_duration,
        "game_duration_str": game_duration_str,
        "game_mode": game_mode,
        "game_mode_cn": game_mode_cn,
        "queue_id": queue_id,
        "queue_name": mode_cn,
        "mode_cn": mode_cn,
        "blue_team_kills": blue_kills,
        "blue_team_turrets": blue_turrets,
        "blue_team_dragons": blue_dragons,
        "blue_team_barons": blue_barons,
        "blue_team_win": final_blue_win,
        "red_team_kills": red_kills,
        "red_team_turrets": red_turrets,
        "red_team_dragons": red_dragons,
        "red_team_barons": red_barons,
        "red_team_win": final_red_win,
        "participants": participants_json,
        "teams": {
            "blue": {
                "win": final_blue_win,
                "kills": blue_kills,
                "towers": blue_turrets,
                "dragons": blue_dragons,
                "damage": blue_dmg_total,
                "gold": blue_gold_total,
                "players": blue_players
            },
            "red": {
                "win": final_red_win,
                "kills": red_kills,
                "towers": red_turrets,
                "dragons": red_dragons,
                "damage": red_dmg_total,
                "gold": red_gold_total,
                "players": red_players
            }
        },
        "raw_game": raw.clone(),
        "raw": raw.clone()
    }))
}

pub fn build_mock_match_detail(gid: i64, db: &Database) -> Value {
    let blue_cids = [266, 64, 517, 145, 111];
    let red_cids = [24, 5, 103, 81, 412];
    let blue_names = ["狂傲天王", "野区萧炎", "中单老顽童", "暴风输出手", "金牌辅助包"];
    let red_names = ["武器大师傅", "赵云在此", "九尾妖仙", "探险家黄毛", "夺命钩魂手"];
    let roles = ["TOP", "JUNGLE", "MIDDLE", "BOTTOM", "UTILITY"];
    let tiers = ["DIAMOND I", "DIAMOND II", "EMERALD I", "DIAMOND IV", "EMERALD II"];

    let mut participants = Vec::new();
    let mut blue_players = Vec::new();
    let mut red_players = Vec::new();
    let mut blue_team_kills = 0i32;
    let mut red_team_kills = 0i32;
    let mut blue_dmg_total = 0i64;
    let mut red_dmg_total = 0i64;
    let mut blue_gold_total = 0i64;
    let mut red_gold_total = 0i64;

    for i in 0..5 {
        let cid = blue_cids[i];
        let cname = db.resolve_hero_name(&cid.to_string()).unwrap_or_else(|| format!("英雄{}", cid));
        let k = 8 + (i as i32 % 3) * 2;
        let d = 2 + (i as i32 % 2);
        let a = 7 + (i as i32 % 4);
        blue_team_kills += k;
        let kda_ratio = ((k + a) as f64 / d.max(1) as f64 * 10.0).round() / 10.0;
        let dmg = 25000 + (i as i64 * 3500);
        let dmg_taken = 18000 + (i as i64 * 2200);
        let gold = 14500 + (i as i64 * 800);
        let cs = 210 + (i as i32 * 15);
        blue_dmg_total += dmg;
        blue_gold_total += gold;

        let s1 = if i == 1 { 11 } else { 4 };
        let s2 = if i == 0 { 12 } else if i == 3 { 7 } else { 14 };
        let (_, s1_icon) = spell_meta(s1);
        let (_, s2_icon) = spell_meta(s2);
        let items = vec![3074, 3053, 3111, 6631, 3071, 3026, 3340];

        let p_val = json!({
            "puuid": format!("mock-blue-{}", i + 1),
            "summoner_name": blue_names[i],
            "game_name": blue_names[i],
            "tag_line": "CN1",
            "team": "blue",
            "champion_id": cid,
            "champion_name": cname,
            "champion_title": "",
            "champion_icon": format!("https://game.gtimg.cn/images/lol/act/img/skinloading/{}000.jpg", cid),
            "champion_level": 14 + (i as i32 % 3),
            "role": roles[i],
            "spell1_id": s1,
            "spell1_icon": s1_icon,
            "spell2_id": s2,
            "spell2_icon": s2_icon,
            "tier": tiers[i],
            "kills": k,
            "deaths": d,
            "assists": a,
            "kda_str": format!("{:.1}", kda_ratio),
            "kda_ratio": kda_ratio,
            "is_mvp": i == 0,
            "is_svp": false,
            "total_damage": dmg,
            "damage": dmg,
            "damage_share_pct": 28.5,
            "is_highest_damage": i == 0,
            "total_damage_taken": dmg_taken,
            "damage_taken": dmg_taken,
            "damage_taken_share_pct": 24.2,
            "damage_mitigated": 25000 + (i as i64 * 2000),
            "gold_earned": gold,
            "gold": gold,
            "minions_killed": cs,
            "cs": cs,
            "items": items,
            "item_trinket": 3340,
            "badges": []
        });
        blue_players.push(p_val.clone());
        participants.push(p_val);
    }

    for i in 0..5 {
        let cid = red_cids[i];
        let cname = db.resolve_hero_name(&cid.to_string()).unwrap_or_else(|| format!("英雄{}", cid));
        let k = 3 + (i as i32 % 2);
        let d = 6 + (i as i32 % 3);
        let a = 4 + (i as i32 % 2);
        red_team_kills += k;
        let kda_ratio = ((k + a) as f64 / d.max(1) as f64 * 10.0).round() / 10.0;
        let dmg = 15000 + (i as i64 * 2500);
        let dmg_taken = 22000 + (i as i64 * 3000);
        let gold = 9500 + (i as i64 * 600);
        let cs = 150 + (i as i32 * 12);
        red_dmg_total += dmg;
        red_gold_total += gold;

        let s1 = if i == 1 { 11 } else { 4 };
        let s2 = if i == 0 { 12 } else if i == 3 { 7 } else { 14 };
        let (_, s1_icon) = spell_meta(s1);
        let (_, s2_icon) = spell_meta(s2);
        let items = vec![3031, 3006, 1055, 3134, 0, 0, 3340];

        let p_val = json!({
            "puuid": format!("mock-red-{}", i + 1),
            "summoner_name": red_names[i],
            "game_name": red_names[i],
            "tag_line": "CN2",
            "team": "red",
            "champion_id": cid,
            "champion_name": cname,
            "champion_title": "",
            "champion_icon": format!("https://game.gtimg.cn/images/lol/act/img/skinloading/{}000.jpg", cid),
            "champion_level": 13 + (i as i32 % 2),
            "role": roles[i],
            "spell1_id": s1,
            "spell1_icon": s1_icon,
            "spell2_id": s2,
            "spell2_icon": s2_icon,
            "tier": tiers[i],
            "kills": k,
            "deaths": d,
            "assists": a,
            "kda_str": format!("{:.1}", kda_ratio),
            "kda_ratio": kda_ratio,
            "is_mvp": false,
            "is_svp": i == 0,
            "total_damage": dmg,
            "damage": dmg,
            "damage_share_pct": 22.0,
            "is_highest_damage": false,
            "total_damage_taken": dmg_taken,
            "damage_taken": dmg_taken,
            "damage_taken_share_pct": 21.0,
            "damage_mitigated": 18000 + (i as i64 * 1500),
            "gold_earned": gold,
            "gold": gold,
            "minions_killed": cs,
            "cs": cs,
            "items": items,
            "item_trinket": 3340,
            "badges": []
        });
        red_players.push(p_val.clone());
        participants.push(p_val);
    }

    let mut raw_participants = Vec::new();
    let mut raw_identities = Vec::new();

    for (idx, p) in participants.iter().enumerate() {
        let pid = (idx + 1) as i64;
        let is_blue = idx < 5;
        let puuid = p.get("puuid").and_then(|v| v.as_str()).unwrap_or("");
        let sname = p.get("summoner_name").and_then(|v| v.as_str()).unwrap_or("");
        let cid = p.get("champion_id").and_then(|v| v.as_i64()).unwrap_or(0);
        let s1 = p.get("spell1_id").and_then(|v| v.as_i64()).unwrap_or(0);
        let s2 = p.get("spell2_id").and_then(|v| v.as_i64()).unwrap_or(0);
        let k = p.get("kills").and_then(|v| v.as_i64()).unwrap_or(0);
        let d = p.get("deaths").and_then(|v| v.as_i64()).unwrap_or(0);
        let a = p.get("assists").and_then(|v| v.as_i64()).unwrap_or(0);
        let dmg = p.get("total_damage").and_then(|v| v.as_i64()).unwrap_or(0);
        let taken = p.get("total_damage_taken").and_then(|v| v.as_i64()).unwrap_or(0);
        let mitig = p.get("damage_mitigated").and_then(|v| v.as_i64()).unwrap_or(0);
        let gold = p.get("gold_earned").and_then(|v| v.as_i64()).unwrap_or(0);
        let cs = p.get("minions_killed").and_then(|v| v.as_i64()).unwrap_or(0);
        let lvl = p.get("champion_level").and_then(|v| v.as_i64()).unwrap_or(1);
        let role = p.get("role").and_then(|v| v.as_str()).unwrap_or("");

        raw_identities.push(json!({
            "participantId": pid,
            "player": {
                "puuid": puuid,
                "summonerName": sname,
                "gameName": sname,
                "tagLine": "CN",
                "profileIcon": 548
            }
        }));

        let mut p_stats = serde_json::Map::new();
        p_stats.insert("win".to_string(), json!(is_blue));
        p_stats.insert("kills".to_string(), json!(k));
        p_stats.insert("deaths".to_string(), json!(d));
        p_stats.insert("assists".to_string(), json!(a));
        p_stats.insert("champLevel".to_string(), json!(lvl));
        p_stats.insert("totalDamageDealtToChampions".to_string(), json!(dmg));
        p_stats.insert("physicalDamageDealtToChampions".to_string(), json!((dmg as f64 * 0.7) as i64));
        p_stats.insert("magicDamageDealtToChampions".to_string(), json!((dmg as f64 * 0.25) as i64));
        p_stats.insert("trueDamageDealtToChampions".to_string(), json!((dmg as f64 * 0.05) as i64));
        p_stats.insert("totalDamageTaken".to_string(), json!(taken));
        p_stats.insert("physicalDamageTaken".to_string(), json!((taken as f64 * 0.6) as i64));
        p_stats.insert("magicDamageTaken".to_string(), json!((taken as f64 * 0.35) as i64));
        p_stats.insert("trueDamageTaken".to_string(), json!((taken as f64 * 0.05) as i64));
        p_stats.insert("damageSelfMitigated".to_string(), json!(mitig));
        p_stats.insert("goldEarned".to_string(), json!(gold));
        p_stats.insert("totalMinionsKilled".to_string(), json!(cs));
        p_stats.insert("neutralMinionsKilled".to_string(), json!(if role == "JUNGLE" { 80 } else { 0 }));
        p_stats.insert("visionScore".to_string(), json!(32));
        p_stats.insert("wardsPlaced".to_string(), json!(11));
        p_stats.insert("wardsKilled".to_string(), json!(3));
        p_stats.insert("item0".to_string(), json!(3031));
        p_stats.insert("item1".to_string(), json!(3046));
        p_stats.insert("item2".to_string(), json!(3006));
        p_stats.insert("item3".to_string(), json!(0));
        p_stats.insert("item4".to_string(), json!(0));
        p_stats.insert("item5".to_string(), json!(0));
        p_stats.insert("item6".to_string(), json!(3340));
        p_stats.insert("perkPrimaryStyle".to_string(), json!(8000));
        p_stats.insert("perkSubStyle".to_string(), json!(8400));
        p_stats.insert("perk0".to_string(), json!(8010));
        p_stats.insert("perk0Var1".to_string(), json!(1250));
        p_stats.insert("perk0Var2".to_string(), json!(0));
        p_stats.insert("perk0Var3".to_string(), json!(0));
        p_stats.insert("perk1".to_string(), json!(9111));
        p_stats.insert("perk1Var1".to_string(), json!(450));
        p_stats.insert("perk1Var2".to_string(), json!(0));
        p_stats.insert("perk1Var3".to_string(), json!(0));
        p_stats.insert("perk2".to_string(), json!(9104));
        p_stats.insert("perk2Var1".to_string(), json!(18));
        p_stats.insert("perk2Var2".to_string(), json!(0));
        p_stats.insert("perk2Var3".to_string(), json!(0));
        p_stats.insert("perk3".to_string(), json!(8299));
        p_stats.insert("perk3Var1".to_string(), json!(680));
        p_stats.insert("perk3Var2".to_string(), json!(0));
        p_stats.insert("perk3Var3".to_string(), json!(0));
        p_stats.insert("perk4".to_string(), json!(8446));
        p_stats.insert("perk4Var1".to_string(), json!(850));
        p_stats.insert("perk4Var2".to_string(), json!(0));
        p_stats.insert("perk4Var3".to_string(), json!(0));
        p_stats.insert("perk5".to_string(), json!(8453));
        p_stats.insert("perk5Var1".to_string(), json!(520));
        p_stats.insert("perk5Var2".to_string(), json!(0));
        p_stats.insert("perk5Var3".to_string(), json!(0));
        p_stats.insert("statPerk0".to_string(), json!(5008));
        p_stats.insert("statPerk1".to_string(), json!(5008));
        p_stats.insert("statPerk2".to_string(), json!(5002));

        let p_timeline = json!({
            "lane": match role {
                "TOP" => "TOP",
                "JUNGLE" => "JUNGLE",
                "MIDDLE" => "MIDDLE",
                _ => "BOTTOM",
            },
            "role": match role {
                "UTILITY" => "DUO_SUPPORT",
                "BOTTOM" => "DUO_CARRY",
                "JUNGLE" => "NONE",
                _ => "SOLO",
            }
        });

        raw_participants.push(json!({
            "participantId": pid,
            "teamId": if is_blue { 100 } else { 200 },
            "championId": cid,
            "spell1Id": s1,
            "spell2Id": s2,
            "stats": p_stats,
            "timeline": p_timeline
        }));
    }

    let raw_game = json!({
        "gameId": gid,
        "gameCreation": 1712000000000i64,
        "gameDuration": 1720,
        "queueId": 420,
        "gameMode": "CLASSIC",
        "participantIdentities": raw_identities,
        "participants": raw_participants,
        "teams": [
            {
                "teamId": 100,
                "win": "Win",
                "towerKills": 9,
                "dragonKills": 3,
                "baronKills": 1
            },
            {
                "teamId": 200,
                "win": "Fail",
                "towerKills": 2,
                "dragonKills": 1,
                "baronKills": 0
            }
        ]
    });

    json!({
        "game_id": gid,
        "game_creation": 1712000000000i64,
        "game_duration": 1720,
        "game_duration_str": "28:40",
        "game_mode": "CLASSIC",
        "game_mode_cn": "排位赛(单双)",
        "queue_id": 420,
        "queue_name": "排位赛(单双)",
        "mode_cn": "排位赛(单双)",
        "blue_team_kills": blue_team_kills,
        "blue_team_turrets": 9,
        "blue_team_dragons": 3,
        "blue_team_barons": 1,
        "blue_team_win": true,
        "red_team_kills": red_team_kills,
        "red_team_turrets": 2,
        "red_team_dragons": 1,
        "red_team_barons": 0,
        "red_team_win": false,
        "participants": participants,
        "teams": {
            "blue": {
                "win": true,
                "kills": blue_team_kills,
                "towers": 9,
                "dragons": 3,
                "damage": blue_dmg_total,
                "gold": blue_gold_total,
                "players": blue_players
            },
            "red": {
                "win": false,
                "kills": red_team_kills,
                "towers": 2,
                "dragons": 1,
                "damage": red_dmg_total,
                "gold": red_gold_total,
                "players": red_players
            }
        },
        "raw_game": raw_game.clone(),
        "raw": raw_game
    })
}

pub fn build_mock_match_timeline(game_id: u64) -> Value {
    let mut frames = Vec::new();
    for minute in 0..=15 {
        let timestamp = (minute as i64) * 60_000;
        let mut p_frames = serde_json::Map::new();

        for pid in 1..=10 {
            let is_blue = pid <= 5;
            let role_idx = (pid - 1) % 5;
            let (pos_x, pos_y) = match (is_blue, role_idx) {
                _ if minute == 0 => {
                    if is_blue {
                        (600, 600)
                    } else {
                        (14200, 14200)
                    }
                }
                (true, 0) => (1800 + minute * 150, 11000 + minute * 100),
                (false, 0) => (2200 + minute * 120, 11500 + minute * 80),
                (true, 1) => (4500 + (minute % 4) * 800, 7500 + (minute % 3) * 600),
                (false, 1) => (9500 - (minute % 4) * 800, 6500 - (minute % 3) * 600),
                (true, 2) => (7000 + minute * 50, 7000 + minute * 50),
                (false, 2) => (7600 - minute * 40, 7600 - minute * 40),
                (true, 3) => (12000 + minute * 80, 2200 + minute * 60),
                (false, 3) => (12500 - minute * 60, 2600 - minute * 40),
                (true, 4) => (11800 + minute * 80, 2400 + minute * 60),
                (false, 4) => (12300 - minute * 60, 2800 - minute * 40),
                _ => (7000, 7000),
            };

            let level = 1 + (minute * 14 / 15).min(17) as i32;
            let cs = if role_idx == 4 {
                minute * 1
            } else if role_idx == 1 {
                minute * 5
            } else {
                minute * 8
            } as i32;
            let gold = 500 + (minute * 450) as i64 + (if is_blue { minute * 50 } else { 0 }) as i64;
            let xp = (minute * 600) as i64;

            p_frames.insert(
                pid.to_string(),
                json!({
                    "participantId": pid,
                    "currentGold": gold % 1000,
                    "totalGold": gold,
                    "level": level,
                    "xp": xp,
                    "minionsKilled": cs,
                    "jungleMinionsKilled": if role_idx == 1 { (minute * 4) as i32 } else { 0 },
                    "dominionScore": 0,
                    "teamScore": 0,
                    "position": {
                        "x": pos_x,
                        "y": pos_y
                    }
                }),
            );
        }

        let mut events = Vec::new();
        if minute == 1 {
            for pid in 1..=10 {
                events.push(json!({
                    "type": "ITEM_PURCHASED",
                    "timestamp": 15000 + (pid as i64 * 1000),
                    "participantId": pid,
                    "itemId": if (pid - 1) % 5 == 1 { 1101 } else { 1055 }
                }));
            }
        } else if minute == 3 {
            events.push(json!({
                "type": "CHAMPION_KILL",
                "timestamp": 195000,
                "killerId": 1,
                "victimId": 6,
                "assistingParticipantIds": [2],
                "position": { "x": 1950, "y": 11200 }
            }));
        } else if minute == 6 {
            events.push(json!({
                "type": "CHAMPION_KILL",
                "timestamp": 372000,
                "killerId": 4,
                "victimId": 9,
                "assistingParticipantIds": [2, 5],
                "position": { "x": 10200, "y": 4800 }
            }));
            events.push(json!({
                "type": "ELITE_MONSTER_KILL",
                "timestamp": 385000,
                "killerId": 2,
                "killerTeamId": 100,
                "monsterType": "DRAGON",
                "monsterSubType": "CHEMTECH_DRAGON",
                "position": { "x": 9866, "y": 4414 }
            }));
        } else if minute == 9 {
            events.push(json!({
                "type": "ELITE_MONSTER_KILL",
                "timestamp": 550000,
                "killerId": 7,
                "killerTeamId": 200,
                "monsterType": "RIFTHERALD",
                "monsterSubType": "",
                "position": { "x": 4900, "y": 10400 }
            }));
        } else if minute == 12 {
            events.push(json!({
                "type": "BUILDING_KILL",
                "timestamp": 730000,
                "killerId": 1,
                "assistingParticipantIds": [2],
                "teamId": 200,
                "buildingType": "TOWER_BUILDING",
                "towerType": "OUTER_TURRET",
                "laneType": "TOP_LANE",
                "position": { "x": 4318, "y": 13875 }
            }));
        }

        frames.push(json!({
            "timestamp": timestamp,
            "participantFrames": p_frames,
            "events": events
        }));
    }

    json!({
        "gameId": game_id,
        "frames": frames
    })
}

async fn send_chat_records(
    State(state): State<AppState>,
    Json(payload): Json<ChatRecordsPayload>,
) -> Json<Value> {
    let mut room_id = payload
        .chat_room_id
        .or(payload.chat_room_id_camel)
        .unwrap_or_default();

    if room_id.is_empty() {
        let session = state.lcu.get_champ_select_session().await;
        if let Some(cd) = session.get("chatDetails") {
            room_id = cd
                .get("multiUserChatId")
                .or_else(|| cd.get("chatRoomId"))
                .or_else(|| cd.get("chatRoomName"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
        }
    }

    if room_id.is_empty() {
        let convs = state.lcu.get_conversations().await;
        for c in &convs {
            let ctype = c.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if ctype == "championSelect" {
                if let Some(id) = c.get("id").and_then(|v| v.as_str()) {
                    room_id = id.to_string();
                    break;
                }
            }
        }
    }

    let ongoing = state.history.get_ongoing_match(&state.lcu, &state.db).await;
    let lines = build_chat_broadcast_lines(&ongoing.my_team, None);

    if !room_id.is_empty() {
        let mut any_success = false;
        for line in &lines {
            if !line.is_empty() {
                let success = state.lcu.send_chat_message(&room_id, line).await;
                if success {
                    any_success = true;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(35)).await;
            }
        }
        Json(json!({
            "success": any_success,
            "message": if any_success { "战绩透视已成功广播至选人聊天室！" } else { "客户端拒绝发送，请确认当前处于选人阶段" }
        }))
    } else {
        Json(json!({
            "success": false,
            "message": "未检测到活跃选人聊天室，请在英雄选择阶段重试"
        }))
    }
}

// -------------------------------------------------------------
// Configuration & Preferences APIs
// -------------------------------------------------------------
async fn get_config(State(state): State<AppState>) -> Json<Value> {
    let cfg = state.config.get().await;
    let presets = get_preset_definitions();
    let preset_info = presets.get(&cfg.current_preset).cloned();

    Json(json!({
        "config": cfg,
        "preset_info": preset_info,
        "presets": presets
    }))
}

async fn update_config(
    State(state): State<AppState>,
    Json(payload): Json<ConfigUpdatePayload>,
) -> Json<Value> {
    let rc = payload.resolve_conflicts.or(payload.resolve_conflicts_camel);
    let hw = payload.hex_synergy_weight.or(payload.hex_synergy_weight_camel);
    let as_ = payload.archetype_strictness.or(payload.archetype_strictness_camel);
    let ci = payload.counter_intensity.or(payload.counter_intensity_camel);
    let gcm = payload.gold_calc_mode.or(payload.gold_calc_mode_camel);
    let tm = payload.trio_mode.or(payload.trio_mode_camel);
    let preset = payload.preset.or(payload.current_preset_camel);
    let aam = payload.auto_accept_match.or(payload.auto_accept_match_camel);
    let aads = payload.auto_accept_delay_seconds.or(payload.auto_accept_delay_seconds_camel);

    let updated = state.config.update(rc, hw, as_, ci, gcm, tm, preset, aam, aads).await;
    let asb = payload.auto_swap_bench.or(payload.auto_swap_bench_camel);
    let aar = payload.auto_apply_runes.or(payload.auto_apply_runes_camel);
    let bpc = payload.bench_preferred_champions.or(payload.bench_preferred_champions_camel);
    let updated = if asb.is_some() || aar.is_some() || bpc.is_some() {
        state.config.update_automation(aam, aads, asb, aar, bpc).await
    } else {
        updated
    };
    let presets = get_preset_definitions();
    let preset_info = presets.get(&updated.current_preset).cloned();

    Json(json!({
        "success": true,
        "config": updated,
        "preset_info": preset_info
    }))
}

async fn get_item_preferences(State(state): State<AppState>) -> Json<Value> {
    let cfg = state.config.get().await;
    let presets = get_preset_definitions();
    let preset_info = presets.get(&cfg.current_preset).cloned();

    Json(json!({
        "resolve_conflicts": cfg.resolve_conflicts,
        "hex_synergy_weight": cfg.hex_synergy_weight,
        "archetype_strictness": cfg.archetype_strictness,
        "counter_intensity": cfg.counter_intensity,
        "gold_calc_mode": cfg.gold_calc_mode,
        "trio_mode": cfg.trio_mode,
        "current_preset": cfg.current_preset,
        "preset_info": preset_info,
        "presets": presets
    }))
}

async fn update_item_preferences(
    State(state): State<AppState>,
    Json(payload): Json<ConfigUpdatePayload>,
) -> Json<Value> {
    let rc = payload.resolve_conflicts.or(payload.resolve_conflicts_camel);
    let hw = payload.hex_synergy_weight.or(payload.hex_synergy_weight_camel);
    let as_ = payload.archetype_strictness.or(payload.archetype_strictness_camel);
    let ci = payload.counter_intensity.or(payload.counter_intensity_camel);
    let gcm = payload.gold_calc_mode.or(payload.gold_calc_mode_camel);
    let tm = payload.trio_mode.or(payload.trio_mode_camel);
    let preset = payload.preset.or(payload.current_preset_camel);
    let aam = payload.auto_accept_match.or(payload.auto_accept_match_camel);
    let aads = payload.auto_accept_delay_seconds.or(payload.auto_accept_delay_seconds_camel);

    let updated = state.config.update(rc, hw, as_, ci, gcm, tm, preset, aam, aads).await;
    let presets = get_preset_definitions();
    let preset_info = presets.get(&updated.current_preset).cloned();

    Json(json!({
        "success": true,
        "resolve_conflicts": updated.resolve_conflicts,
        "hex_synergy_weight": updated.hex_synergy_weight,
        "archetype_strictness": updated.archetype_strictness,
        "counter_intensity": updated.counter_intensity,
        "gold_calc_mode": updated.gold_calc_mode,
        "trio_mode": updated.trio_mode,
        "current_preset": updated.current_preset,
        "preset_info": preset_info
    }))
}

async fn get_overlay_status(State(state): State<AppState>) -> Json<Value> {
    let cfg = state.ingame_overlay.config.read().await;
    Json(json!({
        "inGamePhase": state.ingame_overlay.is_showing.load(std::sync::atomic::Ordering::Relaxed),
        "isWindowActive": state.ingame_overlay.running.load(std::sync::atomic::Ordering::Relaxed),
        "hotkey": cfg.hotkey,
        "voice": cfg.voice_enabled,
        "autoSensingEnabled": cfg.auto_sensing_enabled,
    }))
}

async fn update_overlay_config(State(state): State<AppState>, Json(body): Json<Value>) -> Json<Value> {
    let mut cfg = state.ingame_overlay.config.write().await;
    if let Some(v) = body.get("autoSensingEnabled").and_then(|v| v.as_bool()) {
        cfg.auto_sensing_enabled = v;
    }
    if let Some(v) = body.get("voice").or_else(|| body.get("voiceEnabled")).and_then(|v| v.as_bool()) {
        cfg.voice_enabled = v;
    }
    if let Some(v) = body.get("hotkey").and_then(|v| v.as_str()) {
        cfg.hotkey = v.to_string();
    }
    Json(json!({ "success": true, "message": "Overlay 配置已更新" }))
}

async fn trigger_overlay(State(state): State<AppState>) -> Json<Value> {
    state.ingame_overlay.trigger_scan_with_db(&state.db);
    Json(json!({ "success": true, "message": "海克斯强化推演已触发" }))
}

async fn mock_overlay(State(state): State<AppState>) -> Json<Value> {
    state.ingame_overlay.trigger_scan_with_db(&state.db);
    Json(json!({ "success": true, "message": "模拟感知已激活" }))
}

async fn get_item_overlay_status(State(state): State<AppState>) -> Json<Value> {
    let cfg = state.item_overlay.config.read().await;
    Json(json!({
        "enabled": cfg.enabled,
        "voice": cfg.voice_enabled,
        "hotkey": cfg.hotkey,
    }))
}

async fn update_item_overlay_config(State(state): State<AppState>, Json(body): Json<Value>) -> Json<Value> {
    let mut cfg = state.item_overlay.config.write().await;
    if let Some(v) = body.get("enabled").and_then(|v| v.as_bool()) {
        cfg.enabled = v;
    }
    if let Some(v) = body.get("voice").or_else(|| body.get("voiceEnabled")).and_then(|v| v.as_bool()) {
        cfg.voice_enabled = v;
    }
    if let Some(v) = body.get("hotkey").and_then(|v| v.as_str()) {
        cfg.hotkey = v.to_string();
    }
    Json(json!({ "success": true, "message": "装备推荐 HUD 配置已保存" }))
}

async fn trigger_item_overlay(State(state): State<AppState>) -> Json<Value> {
    state.item_overlay.trigger_manual_scan();
    Json(json!({ "success": true, "message": "装备推荐扫描已执行" }))
}

async fn get_bench_overlay_status(State(state): State<AppState>) -> Json<Value> {
    let cfg = state.bench_overlay.config.read().await;
    Json(json!({
        "enabled": cfg.enabled,
        "clickToSwap": cfg.click_to_swap,
        "isShowing": state.bench_overlay.is_showing.load(std::sync::atomic::Ordering::Relaxed),
        "badgePosition": cfg.badge_position,
        "verticalOffset": cfg.vertical_offset
    }))
}

async fn update_bench_overlay_config(State(state): State<AppState>, Json(body): Json<Value>) -> Json<Value> {
    let mut cfg = state.bench_overlay.config.write().await;
    if let Some(v) = body.get("enabled").and_then(|v| v.as_bool()) {
        cfg.enabled = v;
    }
    if let Some(v) = body.get("clickToSwap").and_then(|v| v.as_bool()) {
        cfg.click_to_swap = v;
    }
    if let Some(v) = body.get("badgePosition").and_then(|v| v.as_str()) {
        cfg.badge_position = v.to_string();
    }
    if let Some(v) = body.get("verticalOffset").and_then(|v| v.as_i64()) {
        cfg.vertical_offset = v as i32;
    }
    Json(json!({ "success": true, "message": "备战席 HUD 配置已保存" }))
}

async fn champ_select_reroll(State(state): State<AppState>) -> Json<Value> {
    let success = state.lcu.reroll_champion().await;
    Json(json!({
        "success": success,
        "message": if success {
            "已成功消耗骰子重置英雄！"
        } else {
            "重随骰子失败：请确认处于极地大乱斗选人阶段且剩余骰子充足"
        }
    }))
}

async fn champ_select_mock(State(state): State<AppState>) -> Json<Value> {
    let mock_bench = vec![
        crate::overlay::BenchChampView {
            slot_index: 0,
            champion_id: 266,
            champion_name: "剑魔".to_string(),
            tier: "T1".to_string(),
            win_rate: 0.58,
        },
        crate::overlay::BenchChampView {
            slot_index: 1,
            champion_id: 64,
            champion_name: "盲僧".to_string(),
            tier: "T1".to_string(),
            win_rate: 0.56,
        },
    ];
    state.bench_overlay.show_bench_heroes(mock_bench);
    Json(json!({ "success": true, "message": "备战席模拟已展开" }))
}

async fn champ_select_stop_mock(State(state): State<AppState>) -> Json<Value> {
    state.bench_overlay.stop();
    Json(json!({ "success": true, "message": "备战席模拟已关闭" }))
}

// -------------------------------------------------------------
// Meta & BigData APIs
// -------------------------------------------------------------
async fn get_meta(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "patch": state.db.meta.report_patch,
        "report_patch": state.db.meta.report_patch,
        "report_date": state.db.meta.report_date,
        "heroes_count": state.db.heroes_summary.len(),
        "augments_count": state.db.augments_summary.len(),
        "total_matches": 11570000,
        "last_updated": state.db.meta.report_date
    }))
}

async fn get_heroes(State(state): State<AppState>) -> Json<Value> {
    Json(json!(state.db.heroes_summary))
}

async fn get_augments(State(state): State<AppState>) -> Json<Value> {
    Json(json!(state.db.augments_summary))
}

async fn get_hero_detail(
    AxPath(id): AxPath<String>,
    State(state): State<AppState>,
) -> Json<Value> {
    if let Some(detail) = state.db.get_hero_detail(&id) {
        return Json(json!(*detail));
    }
    Json(json!({ "error": "Hero detail not found", "id": id }))
}

async fn compare_augments(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    let hero = payload.get("hero").and_then(|v| v.as_str()).unwrap_or("");
    let aug_names = payload.get("augments").and_then(|v| v.as_array());

    let mut results = Vec::new();
    if let Some(arr) = aug_names {
        for item in arr {
            let aname = item.as_str().unwrap_or("");
            if let Some(aug_id) = state.db.name_to_augment_id.get(aname) {
                if let Some(aug) = state.db.augments_map.get(aug_id) {
                    results.push(json!({
                        "name": aug.name,
                        "tier": aug.tier,
                        "win_rate": aug.win_rate,
                        "desc": aug.description
                    }));
                }
            }
        }
    }

    Json(json!({
        "hero": hero,
        "recommendations": results
    }))
}

async fn team_synergy(Json(_payload): Json<Value>) -> Json<Value> {
    Json(json!({
        "synergy_score": 85,
        "evaluation": "阵容前排坦度充足，且包含硬控与物理/魔法混伤",
        "missing_elements": []
    }))
}

async fn get_license_status() -> Json<Value> {
    let status_obj = json!({
        "licensed": true,
        "is_active": true,
        "hwid": "COMMUNITY_EDITION",
        "tier": "ULTIMATE_PRO",
        "expiresAt": 4102444800000_i64,
        "expiresAtStr": "永久社区版",
        "remainingDays": 99999,
        "isUniversal": true,
        "status": "ACTIVE",
        "expiration": "PERMANENT_COMMUNITY_EDITION",
        "message": "已永久授权"
    });
    Json(status_obj)
}

async fn activate_license(Json(_body): Json<Value>) -> Json<Value> {
    let status_obj = json!({
        "licensed": true,
        "is_active": true,
        "hwid": "COMMUNITY_EDITION",
        "tier": "ULTIMATE_PRO",
        "expiresAt": 4102444800000_i64,
        "expiresAtStr": "永久社区版",
        "remainingDays": 99999,
        "isUniversal": true,
        "status": "ACTIVE",
        "expiration": "PERMANENT_COMMUNITY_EDITION",
        "message": "授权校验通过"
    });
    Json(json!({
        "success": true,
        "message": "授权校验通过",
        "status": status_obj,
        "licenseInfo": status_obj
    }))
}

async fn scan_and_compare(
    State(state): State<AppState>,
    Json(payload): Json<OcrScanRequest>,
) -> Json<Value> {
    let hero_input = payload.hero_id.unwrap_or_else(|| "142".to_string());
    let hid = state.db.resolve_hero_id(&hero_input).unwrap_or_else(|| "142".to_string());
    let existing = payload.existing_augments.unwrap_or_default();

    // Collect candidate augment IDs for this hero
    let mut candidate_ids = Vec::new();
    if let Some(hero_detail) = state.db.get_hero_detail(&hid) {
        if let Some(arr) = hero_detail.augments.as_array() {
            for a in arr {
                if let Some(aid) = a.get("id").or_else(|| a.get("augmentId")).and_then(|v| match v {
                    Value::String(s) => Some(s.clone()),
                    Value::Number(n) => Some(n.to_string()),
                    _ => None,
                }) {
                    if !candidate_ids.contains(&aid) {
                        candidate_ids.push(aid);
                    }
                    if candidate_ids.len() >= 3 {
                        break;
                    }
                }
            }
        }
    }

    // Fallback augments if needed
    let fallback_pool = ["1113", "1053", "2095", "1030", "1388", "1068", "2132"];
    for fid in fallback_pool {
        let f_str = fid.to_string();
        if !candidate_ids.contains(&f_str) && candidate_ids.len() < 3 {
            candidate_ids.push(f_str);
        }
    }

    let result = crate::engine::decision::DecisionEngine::compare_augments(
        &state.db,
        &hid,
        &candidate_ids,
        payload.current_stage,
        &existing,
    );

    Json(result)
}

// -------------------------------------------------------------
// Auto-Accept Matchmaking Handlers
// -------------------------------------------------------------
async fn get_auto_accept_status(State(state): State<AppState>) -> Json<Value> {
    let cfg = state.config.get().await;
    let phase = state.lcu.get_gameflow_phase().await;
    let ready_check = state.lcu.get_ready_check().await;

    Json(json!({
        "enabled": cfg.auto_accept_match,
        "delaySeconds": cfg.auto_accept_delay_seconds,
        "gameflowPhase": phase,
        "readyCheck": ready_check
    }))
}

async fn update_auto_accept_config(
    State(state): State<AppState>,
    Json(payload): Json<AutoAcceptConfigPayload>,
) -> Json<Value> {
    let enabled = payload.enabled.or(payload.auto_accept_match).unwrap_or(false);
    let delay = payload.delay_seconds.or(payload.auto_accept_delay_seconds);
    let updated = state.config.set_auto_accept(enabled, delay).await;

    Json(json!({
        "success": true,
        "enabled": updated.auto_accept_match,
        "delaySeconds": updated.auto_accept_delay_seconds
    }))
}

async fn trigger_auto_accept(State(state): State<AppState>) -> Json<Value> {
    let success = state.lcu.accept_ready_check().await;
    Json(json!({
        "success": success,
        "message": if success { "已成功向客户端发送确认接受对局请求！" } else { "客户端未处于就绪确认阶段或请求失败" }
    }))
}

async fn trigger_auto_decline(State(state): State<AppState>) -> Json<Value> {
    let success = state.lcu.decline_ready_check().await;
    Json(json!({
        "success": success,
        "message": if success { "已成功向客户端发送拒绝匹配请求" } else { "客户端未处于就绪确认阶段或请求失败" }
    }))
}

// -------------------------------------------------------------
// Static Asset Fallback (Disk First, Embedded Fallback)
// -------------------------------------------------------------
async fn static_asset_handler(uri: Uri) -> Response {
    let mut path = uri.path().trim_start_matches('/');
    if let Some(stripped) = path.strip_prefix("static/") {
        path = stripped;
    }
    let target = if path.is_empty() { "index.html" } else { path };

    // 1. Check frontend/dist first (if frontend was built)
    let dist_path = std::path::Path::new("frontend/dist").join(target);
    if dist_path.exists() && dist_path.is_file() {
        if let Ok(bytes) = tokio::fs::read(&dist_path).await {
            let mime = mime_guess::from_path(&dist_path).first_or_octet_stream();
            return (
                [(header::CONTENT_TYPE, mime.as_ref())],
                bytes,
            )
                .into_response();
        }
    }

    // 2. Check local disk in static/
    let disk_path = std::path::Path::new("static").join(target);
    if disk_path.exists() && disk_path.is_file() {
        if let Ok(bytes) = tokio::fs::read(&disk_path).await {
            let mime = mime_guess::from_path(&disk_path).first_or_octet_stream();
            return (
                [(header::CONTENT_TYPE, mime.as_ref())],
                bytes,
            )
                .into_response();
        }
    }

    // 3. Check embedded assets
    if let Some(content) = StaticAssets::get(target) {
        let mime = mime_guess::from_path(target).first_or_octet_stream();
        return (
            [(header::CONTENT_TYPE, mime.as_ref())],
            content.data,
        )
            .into_response();
    }

    // 4. Asset fuzzy fallback: if a hashed asset like assets/index-*.js or assets/index-*.css was requested but hash changed
    if target.starts_with("assets/index-") {
        let ext = std::path::Path::new(target).extension().and_then(|s| s.to_str()).unwrap_or("");
        let search_dirs = [std::path::Path::new("frontend/dist/assets"), std::path::Path::new("static/assets")];
        for dir in &search_dirs {
            if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let file_name = entry.file_name();
                    let name_str = file_name.to_string_lossy();
                    if name_str.starts_with("index-") && name_str.ends_with(ext) {
                        if let Ok(bytes) = tokio::fs::read(entry.path()).await {
                            let mime = mime_guess::from_path(entry.path()).first_or_octet_stream();
                            return (
                                [(header::CONTENT_TYPE, mime.as_ref())],
                                bytes,
                            )
                                .into_response();
                        }
                    }
                }
            }
        }
    }

    // If target has a file extension (like .css, .js, .png), never return index.html (SPA fallback)
    // to prevent browser strict MIME type violations!
    if std::path::Path::new(target).extension().is_some() {
        return (StatusCode::NOT_FOUND, "Static Asset Not Found").into_response();
    }

    // 5. SPA Fallback: Serve index.html (frontend/dist first, then static, then embedded)
    let dist_index = std::path::Path::new("frontend/dist/index.html");
    if dist_index.exists() {
        if let Ok(bytes) = tokio::fs::read(&dist_index).await {
            return (
                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                bytes,
            )
                .into_response();
        }
    }

    let index_disk = std::path::Path::new("static").join("index.html");
    if index_disk.exists() {
        if let Ok(bytes) = tokio::fs::read(&index_disk).await {
            return (
                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                bytes,
            )
                .into_response();
        }
    }

    if let Some(content) = StaticAssets::get("index.html") {
        return (
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            content.data,
        )
            .into_response();
    }

    (StatusCode::NOT_FOUND, "Not Found").into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paginate_matches_prevents_secondary_truncation() {
        // Case 1: matches was already paged upstream by LCU [20..39] (20 items)
        let paged_items: Vec<i32> = (20..40).collect();
        let res = paginate_matches(&paged_items, 20, 39);
        assert_eq!(res.len(), 20, "Should NOT skip 20 when already paged");
        assert_eq!(res[0], 20);
        assert_eq!(res[19], 39);

        // Case 2: matches contains full history from index 0 [0..39] (40 items)
        let full_items: Vec<i32> = (0..40).collect();
        let res = paginate_matches(&full_items, 20, 39);
        assert_eq!(res.len(), 20);
        assert_eq!(res[0], 20);
        assert_eq!(res[19], 39);

        // Case 3: first page [0..19]
        let res = paginate_matches(&full_items, 0, 19);
        assert_eq!(res.len(), 20);
        assert_eq!(res[0], 0);
        assert_eq!(res[19], 19);

        // Case 4: filtered queue returns fewer items than page size
        let small_items: Vec<i32> = (20..25).collect();
        let res = paginate_matches(&small_items, 20, 39);
        assert_eq!(res.len(), 5);
        assert_eq!(res[0], 20);
        assert_eq!(res[4], 24);
    }
}
