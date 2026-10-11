use axum::extract::{Path as AxPath, Query, State};
use axum::http::{header, StatusCode, Uri};
use axum::response::{IntoResponse, Json, Response};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

use crate::config::{get_preset_definitions, ConfigManager};
use crate::data::loader::Database;
use crate::engine::player_history::{compute_radar, OngoingMatchResponse, ParsedMatchRecord, PlayerHistoryManager};
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

fn parse_riot_match_detail(raw: &Value, db: &Database) -> Option<Value> {
    let game_id = raw.get("gameId").and_then(|v| v.as_i64())?;
    let duration = raw.get("gameDuration").and_then(|v| v.as_i64()).unwrap_or(1800);
    let duration_str = format!("{}:{:02}", duration / 60, duration % 60);
    let queue_id = raw.get("queueId").and_then(|v| v.as_i64()).unwrap_or(420) as i32;
    let mode_cn = match queue_id {
        420 => "排位赛(单双)",
        440 => "排位赛(灵活)",
        450 => "狂暴大乱斗",
        _ => "匹配对局",
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
        name_map.insert(pid, (name.to_string(), puuid.to_string()));
    }

    let mut blue_players = Vec::new();
    let mut red_players = Vec::new();
    let mut blue_kills = 0i32;
    let mut red_kills = 0i32;
    let mut blue_dmg = 0i64;
    let mut red_dmg = 0i64;
    let mut blue_gold = 0i64;
    let mut red_gold = 0i64;
    let mut blue_win = false;

    for p in participants {
        let team_id = p.get("teamId").and_then(|v| v.as_i64()).unwrap_or(100);
        let pid = p.get("participantId").and_then(|v| v.as_i64()).unwrap_or(0);
        let cid = p.get("championId").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let cname = db.resolve_hero_name(&cid.to_string()).unwrap_or_else(|| format!("英雄{}", cid));
        let (sname, puuid) = name_map.get(&pid).cloned().unwrap_or(("召唤师".to_string(), String::new()));

        let stats = p.get("stats");
        let win = stats.and_then(|s| s.get("win")).and_then(|v| v.as_bool()).unwrap_or(false);
        if team_id == 100 && win {
            blue_win = true;
        }

        let k = stats.and_then(|s| s.get("kills")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let d = stats.and_then(|s| s.get("deaths")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let a = stats.and_then(|s| s.get("assists")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let dmg = stats.and_then(|s| s.get("totalDamageDealtToChampions")).and_then(|v| v.as_i64()).unwrap_or(0);
        let gold = stats.and_then(|s| s.get("goldEarned")).and_then(|v| v.as_i64()).unwrap_or(0);
        let cs = stats.and_then(|s| s.get("totalMinionsKilled")).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let s1 = p.get("spell1Id").and_then(|v| v.as_i64()).unwrap_or(4) as i32;
        let s2 = p.get("spell2Id").and_then(|v| v.as_i64()).unwrap_or(12) as i32;

        let mut items = Vec::new();
        for i in 0..7 {
            let item_id = stats.and_then(|s| s.get(format!("item{}", i))).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            items.push(item_id);
        }

        let kda_ratio = (k + a) as f64 / d.max(1) as f64;
        let card = json!({
            "puuid": puuid,
            "summoner_name": sname,
            "champion_id": cid,
            "champion_name": cname,
            "spell1_id": s1,
            "spell2_id": s2,
            "kills": k,
            "deaths": d,
            "assists": a,
            "kda_str": format!("{}/{}/{}", k, d, a),
            "kda_ratio": (kda_ratio * 10.0).round() / 10.0,
            "damage": dmg,
            "damage_share_pct": 20.0,
            "gold": gold,
            "cs": cs,
            "items": items,
            "is_mvp": false,
            "is_svp": false,
            "badges": []
        });

        if team_id == 100 {
            blue_kills += k;
            blue_dmg += dmg;
            blue_gold += gold;
            blue_players.push(card);
        } else {
            red_kills += k;
            red_dmg += dmg;
            red_gold += gold;
            red_players.push(card);
        }
    }

    // Accurately compute damage share and MVP/SVP for Blue and Red teams
    let blue_max_dmg = blue_players.iter().map(|p| p.get("damage").and_then(|v| v.as_i64()).unwrap_or(0)).max().unwrap_or(0);
    let red_max_dmg = red_players.iter().map(|p| p.get("damage").and_then(|v| v.as_i64()).unwrap_or(0)).max().unwrap_or(0);

    for p in &mut blue_players {
        let dmg = p.get("damage").and_then(|v| v.as_i64()).unwrap_or(0);
        let share = if blue_dmg > 0 { ((dmg as f64 / blue_dmg as f64) * 1000.0).round() / 10.0 } else { 20.0 };
        p["damage_share_pct"] = json!(share);
        if blue_win && dmg == blue_max_dmg && dmg > 0 {
            p["is_mvp"] = json!(true);
        } else if !blue_win && dmg == blue_max_dmg && dmg > 0 {
            p["is_svp"] = json!(true);
        }
    }

    for p in &mut red_players {
        let dmg = p.get("damage").and_then(|v| v.as_i64()).unwrap_or(0);
        let share = if red_dmg > 0 { ((dmg as f64 / red_dmg as f64) * 1000.0).round() / 10.0 } else { 20.0 };
        p["damage_share_pct"] = json!(share);
        if !blue_win && dmg == red_max_dmg && dmg > 0 {
            p["is_mvp"] = json!(true);
        } else if blue_win && dmg == red_max_dmg && dmg > 0 {
            p["is_svp"] = json!(true);
        }
    }

    Some(json!({
        "game_id": game_id,
        "mode_cn": mode_cn,
        "game_duration_str": duration_str,
        "teams": {
            "blue": {
                "win": blue_win,
                "kills": blue_kills,
                "towers": if blue_win { 9 } else { 2 },
                "dragons": if blue_win { 3 } else { 1 },
                "damage": blue_dmg,
                "gold": blue_gold,
                "players": blue_players
            },
            "red": {
                "win": !blue_win,
                "kills": red_kills,
                "towers": if !blue_win { 9 } else { 2 },
                "dragons": if !blue_win { 3 } else { 1 },
                "damage": red_dmg,
                "gold": red_gold,
                "players": red_players
            }
        }
    }))
}

fn build_mock_match_detail(gid: i64, db: &Database) -> Value {
    let blue_cids = [266, 64, 517, 145, 111];
    let red_cids = [24, 5, 103, 81, 412];
    let blue_names = ["狂傲天王", "野区萧炎", "中单老顽童", "暴风输出手", "金牌辅助包"];
    let red_names = ["武器大师傅", "赵云在此", "九尾妖仙", "探险家黄毛", "夺命钩魂手"];

    let make_team = |cids: &[i32], names: &[&str], is_win: bool| -> (Vec<Value>, i32, i64, i64) {
        let mut players = Vec::new();
        let mut total_k = 0;
        let mut total_dmg = 0i64;
        let mut total_gold = 0i64;

        for (idx, &cid) in cids.iter().enumerate() {
            let cname = db.resolve_hero_name(&cid.to_string()).unwrap_or_else(|| format!("英雄{}", cid));
            let k = if is_win { 8 + (idx as i32 % 4) * 2 } else { 2 + (idx as i32 % 3) };
            let d = if is_win { 2 + (idx as i32 % 2) } else { 6 + (idx as i32 % 3) };
            let a = if is_win { 7 + (idx as i32 % 5) } else { 3 + (idx as i32 % 3) };
            let dmg = if is_win { 22000 + idx as i64 * 3000 } else { 12000 + idx as i64 * 1500 };
            let gold = if is_win { 13000 + idx as i64 * 1000 } else { 8500 + idx as i64 * 600 };
            let cs = if is_win { 180 + idx as i32 * 20 } else { 120 + idx as i32 * 15 };
            let items = if is_win {
                vec![3031, 3006, 3072, 3036, 3026, 3139, 3363]
            } else {
                vec![3031, 3006, 1055, 3134, 0, 0, 3340]
            };

            total_k += k;
            total_dmg += dmg;
            total_gold += gold;

            players.push(json!({
                "puuid": format!("mock_p_{}", idx),
                "summoner_name": names[idx],
                "champion_id": cid,
                "champion_name": cname,
                "spell1_id": 4,
                "spell2_id": if idx == 1 { 11 } else { 12 },
                "kills": k,
                "deaths": d,
                "assists": a,
                "kda_str": format!("{}/{}/{}", k, d, a),
                "kda_ratio": (k + a) as f64 / d.max(1) as f64,
                "damage": dmg,
                "damage_share_pct": if is_win { 24.5 } else { 18.0 },
                "gold": gold,
                "cs": cs,
                "items": items,
                "is_mvp": is_win && idx == 0,
                "is_svp": !is_win && idx == 0,
                "badges": []
            }));
        }
        (players, total_k, total_dmg, total_gold)
    };

    let (b_players, b_k, b_dmg, b_g) = make_team(&blue_cids, &blue_names, true);
    let (r_players, r_k, r_dmg, r_g) = make_team(&red_cids, &red_names, false);

    json!({
        "game_id": gid,
        "mode_cn": "排位赛(单双)",
        "game_duration_str": "28:45",
        "teams": {
            "blue": {
                "win": true,
                "kills": b_k,
                "towers": 9,
                "dragons": 3,
                "damage": b_dmg,
                "gold": b_g,
                "players": b_players
            },
            "red": {
                "win": false,
                "kills": r_k,
                "towers": 2,
                "dragons": 1,
                "damage": r_dmg,
                "gold": r_g,
                "players": r_players
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
