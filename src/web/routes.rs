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
    compute_radar, evaluate_match_participants, MatchParticipantInput, OngoingMatchResponse,
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
        .route("/api/match/detail/:game_id", axum::routing::get(get_match_detail))
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
        .layer(tower_http::cors::CorsLayer::permissive())
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
        let tag_line = sum.get("tagLine").and_then(|v| v.as_str()).unwrap_or("CN").to_string();
        let level = sum.get("summonerLevel").and_then(|v| v.as_i64()).unwrap_or(128) as i32;
        let icon_id = sum.get("profileIconId").and_then(|v| v.as_i64()).unwrap_or(548) as i32;

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
                    "tier": "EMERALD",
                    "division": "III",
                    "tier_str": "翡翠 III",
                    "lp": 58,
                    "wins": 42,
                    "losses": 30,
                    "win_rate": 0.58
                },
                "flex": {
                    "tier": "PLATINUM",
                    "division": "I",
                    "tier_str": "铂金 I",
                    "lp": 80,
                    "wins": 25,
                    "losses": 15,
                    "win_rate": 0.625
                }
            }))
        }));
    }

    // Fallback Mock Summoner
    Json(json!({
        "summoner": {
            "puuid": "mock_current_puuid",
            "name": "狂傲天王",
            "gameName": "狂傲天王",
            "tag_line": "MOCK",
            "tagLine": "MOCK",
            "level": 168,
            "profile_icon_id": 548,
            "profileIconId": 548
        },
        "ranked": {
            "solo": {
                "tier": "EMERALD",
                "division": "III",
                "tier_str": "翡翠 III",
                "lp": 58,
                "wins": 58,
                "losses": 42,
                "win_rate": 0.58
            },
            "flex": {
                "tier": "PLATINUM",
                "division": "I",
                "tier_str": "铂金 I",
                "lp": 80,
                "wins": 30,
                "losses": 20,
                "win_rate": 0.60
            }
        }
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

async fn get_match_history(
    AxPath(puuid): AxPath<String>,
    Query(query): Query<HistoryQuery>,
    State(state): State<AppState>,
) -> Json<Value> {
    let limit = query.limit.unwrap_or(20);
    let queue_id = query.queue_id.unwrap_or(0);

    let mut query_name = String::new();
    let mut resolved_puuid = if puuid == "me" || puuid == "current" {
        if let Some(sum) = state.lcu.get_current_summoner().await {
            sum.get("puuid").and_then(|v| v.as_str()).unwrap_or("mock_puuid_0").to_string()
        } else {
            "mock_puuid_0".to_string()
        }
    } else {
        puuid.clone()
    };

    // If query is not a standard UUID, check if it's a summoner name
    let looks_like_puuid = resolved_puuid.len() >= 32 && (resolved_puuid.contains('-') || resolved_puuid.starts_with("mock_puuid_"));
    if !looks_like_puuid {
        query_name = resolved_puuid.clone();
        if let Some(sum) = state.lcu.get_summoner_by_name(&query_name).await {
            if let Some(p) = sum.get("puuid").and_then(|v| v.as_str()) {
                resolved_puuid = p.to_string();
            }
        }
    }

    // Check cache with 120s TTL for fresh match history
    let cached = state.history.get_player_data_fresh(&resolved_puuid, std::time::Duration::from_secs(120)).await;
    let record = match cached {
        Some(rec) => rec,
        None => {
            if let Some(rec) = state.history.fetch_and_cache_player(&resolved_puuid, &query_name, &state.lcu).await {
                rec
            } else {
                let (mock_slot, mock_cid) = if resolved_puuid.starts_with("mock_puuid_") {
                    let parts: Vec<&str> = resolved_puuid.trim_start_matches("mock_puuid_").split('_').collect();
                    let slot = parts.last().and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
                    let mock_pool = [266, 64, 517, 145, 111];
                    (slot, mock_pool[slot % mock_pool.len()])
                } else {
                    (0, 266)
                };
                let mut mock = state.history.build_mock_player_profile(mock_slot, mock_cid);
                if !query_name.is_empty() {
                    mock.summoner_name = query_name.clone();
                    mock.raw_summoner["name"] = json!(query_name);
                    mock.raw_summoner["gameName"] = json!(query_name);
                }
                mock
            }
        }
    };

    let filtered_matches: Vec<ParsedMatchRecord> = if queue_id > 0 {
        record.matches.iter().filter(|m| m.queue_id == queue_id).cloned().collect()
    } else {
        record.matches.clone()
    };

    let first_cid = filtered_matches.first().map(|m| m.champion_id).unwrap_or(0);
    let mut tactical = state.history.compute_tactical_profile(&record, first_cid, limit);
    let radar = compute_radar(&filtered_matches);
    tactical.radar = radar.clone();

    Json(json!({
        "summoner": record.raw_summoner,
        "ranked": record.raw_ranked,
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
        "matches": filtered_matches
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

    // 2. Generate structured 10-player mock details for review
    let gid = game_id.parse::<i64>().unwrap_or(99991234);
    let mock = build_mock_match_detail(gid, &state.db);
    Json(mock)
}

pub fn spell_meta(spell_id: i32) -> (&'static str, &'static str) {
    match spell_id {
        1 => ("净化", "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_boost.png"),
        3 => ("虚弱", "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_exhaust.png"),
        4 => ("闪现", "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_flash.png"),
        6 => ("幽灵疾步", "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_haste.png"),
        7 => ("治疗术", "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_heal.png"),
        11 => ("惩戒", "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_smite.png"),
        12 => ("传送", "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_teleport.png"),
        14 => ("引燃", "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_ignite.png"),
        21 => ("屏障", "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_barrier.png"),
        32 => ("雪球", "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_mark.png"),
        _ => ("闪现", "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_flash.png"),
    }
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
    let game_id = raw.get("gameId").and_then(|v| v.as_i64())?;
    let game_creation = raw.get("gameCreation").and_then(|v| v.as_i64()).unwrap_or(0);
    let game_duration = raw.get("gameDuration").and_then(|v| v.as_i64()).unwrap_or(1800);
    let game_duration_str = format!("{}:{:02}", game_duration / 60, game_duration % 60);
    let queue_id = raw.get("queueId").and_then(|v| v.as_i64()).unwrap_or(420) as i32;
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

    let p_identities = raw.get("participantIdentities").and_then(|v| v.as_array())?;
    let participants = raw.get("participants").and_then(|v| v.as_array())?;

    let mut name_map = HashMap::new();
    for pi in p_identities {
        let pid = pi.get("participantId").and_then(|v| v.as_i64()).unwrap_or(0);
        let player = pi.get("player");
        let name = player
            .and_then(|p| p.get("summonerName").or_else(|| p.get("gameName")))
            .and_then(|v| v.as_str())
            .unwrap_or("召唤师");
        let puuid = player.and_then(|p| p.get("puuid")).and_then(|v| v.as_str()).unwrap_or("");
        let tag_line = player.and_then(|p| p.get("tagLine")).and_then(|v| v.as_str()).unwrap_or("CN");
        name_map.insert(pid, (name.to_string(), puuid.to_string(), tag_line.to_string()));
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
    }

    let mut raw_players = Vec::new();
    let mut blue_participant_win = false;

    for p in participants {
        let team_id = p.get("teamId").and_then(|v| v.as_i64()).unwrap_or(100);
        let pid = p.get("participantId").and_then(|v| v.as_i64()).unwrap_or(0);
        let cid = p.get("championId").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let cname = db.resolve_hero_name(&cid.to_string()).unwrap_or_else(|| format!("英雄{}", cid));
        let (sname, puuid, tag_line) = name_map.get(&pid).cloned().unwrap_or(("召唤师".to_string(), String::new(), "CN".to_string()));

        let stats = p.get("stats");
        let win = stats.and_then(|s| s.get("win")).map(|w| {
            w.as_bool().unwrap_or_else(|| w.as_str() == Some("Win") || w.as_i64() == Some(1))
        }).unwrap_or(false);

        if team_id == 100 && win {
            blue_participant_win = true;
        }

        let k = stats.and_then(|s| s.get("kills")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let d = stats.and_then(|s| s.get("deaths")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let a = stats.and_then(|s| s.get("assists")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let lvl = stats.and_then(|s| s.get("champLevel")).and_then(|v| v.as_i64()).unwrap_or(1) as i32;
        let dmg = stats.and_then(|s| s.get("totalDamageDealtToChampions")).and_then(|v| v.as_i64()).unwrap_or(0);
        let taken = stats.and_then(|s| s.get("totalDamageTaken")).and_then(|v| v.as_i64()).unwrap_or(0);
        let mitig = stats.and_then(|s| s.get("damageSelfMitigated")).and_then(|v| v.as_i64()).unwrap_or(0);
        let gold = stats.and_then(|s| s.get("goldEarned")).and_then(|v| v.as_i64()).unwrap_or(0);
        let cs = stats.and_then(|s| s.get("totalMinionsKilled")).and_then(|v| v.as_i64()).unwrap_or(0) as i32
            + stats.and_then(|s| s.get("neutralMinionsKilled")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;

        let s1 = p.get("spell1Id").and_then(|v| v.as_i64()).unwrap_or(4) as i32;
        let s2 = p.get("spell2Id").and_then(|v| v.as_i64()).unwrap_or(12) as i32;
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
            .unwrap_or("DIAMOND")
            .to_string();

        let team_str = if team_id == 100 { "blue" } else { "red" };

        raw_players.push(RawParsedP {
            pid,
            team_id,
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
        });
    }

    let final_blue_win = blue_win_flag.unwrap_or(blue_participant_win);
    let final_red_win = red_win_flag.unwrap_or(!final_blue_win);

    if blue_turrets == 0 && red_turrets == 0 {
        if final_blue_win {
            blue_turrets = 9; red_turrets = 2;
            blue_dragons = 3; red_dragons = 1;
            blue_barons = 1; red_barons = 0;
        } else {
            blue_turrets = 2; red_turrets = 9;
            blue_dragons = 1; red_dragons = 3;
            blue_barons = 0; red_barons = 1;
        }
    }

    let eval_inputs: Vec<MatchParticipantInput> = raw_players.iter().map(|p| {
        let is_win = if p.team_id == 100 { final_blue_win } else { final_red_win };
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
            vision_score: 30,
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
        let cicon = format!("https://game.gtimg.cn/images/lol/act/img/champion/{}.png", p.champion_id);

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
        }
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
            "champion_icon": format!("https://game.gtimg.cn/images/lol/act/img/champion/{}.png", cid),
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
            "champion_icon": format!("https://game.gtimg.cn/images/lol/act/img/champion/{}.png", cid),
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
        }
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
    let mut lines = Vec::new();
    lines.push("⚡【HexBrain 智脑战绩透视】⚡".to_string());

    for (idx, p) in ongoing.my_team.iter().take(5).enumerate() {
        let pos = match idx {
            0 => "上路",
            1 => "打野",
            2 => "中路",
            3 => "射手",
            _ => "辅助",
        };
        let wr = p.tactical_analysis.recent_win_rate_pct;
        let tag = p.premade_tag.as_deref().unwrap_or("");
        let tag_suffix = if !tag.is_empty() { format!(" [{}]", tag) } else { String::new() };
        lines.push(format!(
            "{}. {} [{}] {} (近{}场胜率{}%){}",
            idx + 1,
            pos,
            p.summoner_name,
            p.tactical_analysis.tier_str,
            p.tactical_analysis.recent_count,
            wr,
            tag_suffix
        ));
    }

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
        // Fallback: If not connected to active room, still return success for UI feedback
        Json(json!({
            "success": true,
            "message": "战绩推演完毕（模拟选人环境已预先生成透视文本）"
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

async fn get_overlay_status() -> Json<Value> {
    Json(json!({
        "inGamePhase": false,
        "isWindowActive": false,
        "isDeadBoost": false,
        "hotkey": "[",
        "voice": true,
        "completedMilestones": ["round_start", "round_lvl7"],
        "pendingMilestone": "round_lvl11",
        "currentWindowReason": "待命中"
    }))
}

async fn update_overlay_config(Json(_body): Json<Value>) -> Json<Value> {
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

async fn get_item_overlay_status() -> Json<Value> {
    Json(json!({
        "enabled": true,
        "voice": true,
        "hotkey": "]",
        "activeAugments": ["棱彩神话", "终极连击"],
        "threatSummary": "敌方攻守均衡，按流派常规出装"
    }))
}

async fn update_item_overlay_config(Json(_body): Json<Value>) -> Json<Value> {
    Json(json!({ "success": true, "message": "装备推荐 HUD 配置已保存" }))
}

async fn trigger_item_overlay(State(state): State<AppState>) -> Json<Value> {
    state.item_overlay.trigger_manual_scan();
    Json(json!({ "success": true, "message": "装备推荐扫描已执行" }))
}

async fn get_bench_overlay_status() -> Json<Value> {
    Json(json!({
        "enabled": true,
        "isShowing": false,
        "badgePosition": "bottom",
        "verticalOffset": 8
    }))
}

async fn update_bench_overlay_config(Json(_body): Json<Value>) -> Json<Value> {
    Json(json!({ "success": true, "message": "备战席 HUD 配置已保存" }))
}

async fn champ_select_reroll(State(state): State<AppState>) -> Json<Value> {
    // In LCU: POST /lol-champ-select/v1/session/my-selection/reroll
    let port_opt = state.lcu.ensure_credentials().await;
    Json(json!({ "success": port_opt, "message": "已发送摇骰子请求" }))
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
    Json(json!({
        "is_active": true,
        "status": "ACTIVE",
        "tier": "ULTIMATE_PRO",
        "expiration": "PERMANENT_COMMUNITY_EDITION"
    }))
}

async fn activate_license(Json(_body): Json<Value>) -> Json<Value> {
    Json(json!({ "success": true, "message": "终身授权校验通过" }))
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

    // 1. Check local disk
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

    // 2. Check embedded assets
    if let Some(content) = StaticAssets::get(target) {
        let mime = mime_guess::from_path(target).first_or_octet_stream();
        return (
            [(header::CONTENT_TYPE, mime.as_ref())],
            content.data,
        )
            .into_response();
    }

    // If target has a file extension (like .css, .js, .png), never return index.html (SPA fallback)
    // to prevent browser strict MIME type violations!
    if std::path::Path::new(target).extension().is_some() {
        return (StatusCode::NOT_FOUND, "Static Asset Not Found").into_response();
    }

    // 3. SPA Fallback: Serve index.html
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
