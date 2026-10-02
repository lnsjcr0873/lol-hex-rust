use crate::config::{get_preset_definitions, ConfigManager};
use crate::crawler::{Exporter, HexdataCrawler, OpggClient};
use crate::data::loader::Database;
use crate::data::models::AugmentSummary;
use crate::engine::decision::DecisionEngine;
use crate::engine::ranked_draft::RankedDraftManager;
use crate::lcu::{LcuClient, LiveClientDataReader};
use crate::overlay::{BenchOverlay, InGameOverlay, ItemOverlay, RankedOverlay};
use crate::server::assets::static_handler;
use crate::server::ws::{ws_handler, WsBroadcaster};

use axum::extract::{Path as AxPath, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Json, Response};
use axum::routing::{get, post};
use axum::Router;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub config_manager: Arc<ConfigManager>,
    pub lcu_client: Arc<LcuClient>,
    pub live_client: Arc<LiveClientDataReader>,
    pub ranked_draft: Arc<RwLock<RankedDraftManager>>,
    pub bench_overlay: Arc<BenchOverlay>,
    pub ranked_overlay: Arc<RankedOverlay>,
    pub ingame_overlay: Arc<InGameOverlay>,
    pub item_overlay: Arc<ItemOverlay>,
    pub opgg_client: Arc<OpggClient>,
    pub hexdata_client: Arc<HexdataCrawler>,
    pub ws_broadcaster: Arc<WsBroadcaster>,
}

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Meta & Heroes API
        .route("/api/meta", get(get_meta))
        .route("/api/heroes", get(get_heroes))
        .route("/api/hero/:hero_id", get(get_hero_decision))
        .route("/api/hero/:hero_id/augments-pool", get(get_hero_augments_pool))
        .route("/api/hero/:hero_id/augments/search", get(search_hero_augments))
        .route("/api/hero/:hero_id/item-recommendations", get(get_hero_item_recommendations))
        .route("/api/postmatch/:hero_id", get(get_postmatch))
        .route("/api/relationships/:hero_id", get(get_relationships))

        // Augments API
        .route("/api/augments", get(get_augments))
        .route("/api/augments/full", get(get_augments_full))
        .route("/api/augment/compare", post(compare_augments))
        .route("/api/ocr/scan-and-compare", post(scan_and_compare))
        .route("/api/team/synergy", post(team_synergy))

        // LCU API
        .route("/api/lcu/status", get(get_lcu_status))
        .route("/api/lcu/apply-spells", post(apply_summoner_spells))
        .route("/api/lcu/mock-sync", post(mock_live_sync))
        .route("/api/champ-select/swap", post(swap_bench_champion))
        .route("/api/champ-select/reroll", post(reroll_champion))

        // Overlays API
        .route("/api/overlay/trigger", post(trigger_ingame_overlay))
        .route("/api/overlay/status", get(get_overlay_status))
        .route("/api/overlay/config", post(update_overlay_config))
        .route("/api/overlay/bench/status", get(get_bench_overlay_status))
        .route("/api/overlay/bench/config", post(update_bench_overlay_config))

        // Item Overlay API
        .route("/api/item-overlay/status", get(get_item_overlay_status))
        .route("/api/item-overlay/config", post(update_item_overlay_config))
        .route("/api/item-overlay/trigger", post(trigger_item_overlay))
        .route("/api/item-overlay/preferences", get(get_item_preferences).post(update_item_preferences))

        // Ranked Draft API
        .route("/api/ranked/state", get(get_ranked_state))
        .route("/api/ranked/apply-runes", post(apply_ranked_runes))
        .route("/api/ranked/apply-spells", post(apply_ranked_spells))
        .route("/api/ranked/config", post(update_ranked_config))
        .route("/api/ranked/tiers", get(get_ranked_tiers))
        .route("/api/ranked/matchup", get(get_ranked_matchup))
        .route("/api/ranked/counters", get(get_ranked_counters))
        .route("/api/ranked/synergies", get(get_ranked_synergies))
        .route("/api/ranked/draft-decision", get(get_draft_decision))
        .route("/api/ranked/test-overlay", post(test_ranked_overlay))
        .route("/api/overlay/debug", get(get_overlay_debug))

        // Data Export API
        .route("/api/export/heroes.csv", get(export_heroes_csv))
        .route("/api/export/augments.csv", get(export_augments_csv))

        // WebSocket endpoint
        .route("/ws/live", get({
            let broadcaster = state.ws_broadcaster.clone();
            move |ws| ws_handler(ws, broadcaster)
        }))

        // Static files fallback
        .fallback(static_handler)
        .layer(cors)
        .with_state(state)
}

// ---------------- Handlers ----------------

async fn get_meta(State(state): State<AppState>) -> Json<Value> {
    Json(DecisionEngine::get_meta_info(&state.db))
}

#[derive(Deserialize)]
struct HeroesQuery {
    role: Option<String>,
    search: Option<String>,
    tier: Option<i32>,
    #[serde(default = "default_sort")]
    sort_by: String,
}

fn default_sort() -> String {
    "tier".to_string()
}

async fn get_heroes(
    State(state): State<AppState>,
    Query(q): Query<HeroesQuery>,
) -> Json<Vec<Value>> {
    let list = DecisionEngine::get_heroes_list(
        &state.db,
        q.role.as_deref(),
        q.search.as_deref(),
        q.tier,
        &q.sort_by,
    );
    Json(list)
}

async fn get_hero_decision(
    State(state): State<AppState>,
    AxPath(hero_id): AxPath<String>,
) -> Result<Json<Value>, StatusCode> {
    match DecisionEngine::get_hero_decision_profile(&state.db, &hero_id) {
        Some(val) => Ok(Json(val)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn get_hero_augments_pool(
    State(state): State<AppState>,
    AxPath(hero_id): AxPath<String>,
) -> Result<Json<Vec<Value>>, StatusCode> {
    let hid = state.db.resolve_hero_id(&hero_id).ok_or(StatusCode::NOT_FOUND)?;
    let mut augs = Vec::new();

    if let Some(detail) = state.db.get_hero_detail(&hid) {
        if let Some(arr) = detail.augments.as_array() {
            augs = arr.clone();
        } else if let Some(obj) = detail.augments.as_object() {
            for k in ["sTier", "aTier", "bTier", "traps"] {
                if let Some(arr) = obj.get(k).and_then(|v| v.as_array()) {
                    augs.extend(arr.clone());
                }
            }
        }
    }

    Ok(Json(augs))
}

#[derive(Deserialize)]
struct SearchAugmentsQuery {
    #[serde(default)]
    q: String,
}

async fn search_hero_augments(
    State(state): State<AppState>,
    AxPath(hero_id): AxPath<String>,
    Query(query): Query<SearchAugmentsQuery>,
) -> Json<Vec<Value>> {
    Json(DecisionEngine::search_augments_for_hero(&state.db, &hero_id, &query.q))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AugmentCompareReq {
    hero_id: String,
    augment_ids: Vec<String>,
    current_stage: Option<i32>,
    existing_augments: Option<Vec<String>>,
}

async fn compare_augments(
    State(state): State<AppState>,
    Json(req): Json<AugmentCompareReq>,
) -> Json<Value> {
    let existing = req.existing_augments.unwrap_or_default();
    let res = DecisionEngine::compare_augments(
        &state.db,
        &req.hero_id,
        &req.augment_ids,
        req.current_stage,
        &existing,
    );
    Json(res)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScanAndCompareReq {
    hero_id: String,
    current_stage: Option<i32>,
    existing_augments: Option<Vec<String>>,
    candidate_augments: Option<Vec<String>>,
}

async fn scan_and_compare(
    State(state): State<AppState>,
    Json(req): Json<ScanAndCompareReq>,
) -> Json<Value> {
    let candidates = req.candidate_augments.unwrap_or_else(|| {
        vec!["1238".to_string(), "1068".to_string(), "1124".to_string()]
    });
    let existing = req.existing_augments.unwrap_or_default();
    let res = DecisionEngine::compare_augments(
        &state.db,
        &req.hero_id,
        &candidates,
        req.current_stage,
        &existing,
    );
    Json(res)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TeamSynergyReq {
    hero_ids: Vec<String>,
}

async fn team_synergy(
    State(state): State<AppState>,
    Json(req): Json<TeamSynergyReq>,
) -> Json<Value> {
    Json(DecisionEngine::analyze_team_synergy(&state.db, &req.hero_ids))
}

async fn get_augments(State(state): State<AppState>) -> Json<Vec<AugmentSummary>> {
    Json(state.db.augments_summary.clone())
}

async fn get_augments_full(State(state): State<AppState>) -> Json<Vec<AugmentSummary>> {
    Json(state.db.augments_summary.clone())
}

async fn get_postmatch(
    State(state): State<AppState>,
    AxPath(hero_id): AxPath<String>,
) -> Result<Json<Value>, StatusCode> {
    let hid = state.db.resolve_hero_id(&hero_id).ok_or(StatusCode::NOT_FOUND)?;
    if let Some(pm) = state.db.postmatch_data.get(&hid) {
        Ok(Json(json!(pm)))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn get_relationships(
    State(state): State<AppState>,
    AxPath(hero_id): AxPath<String>,
) -> Result<Json<Value>, StatusCode> {
    let hid = state.db.resolve_hero_id(&hero_id).ok_or(StatusCode::NOT_FOUND)?;
    if let Some(detail) = state.db.get_hero_detail(&hid) {
        Ok(Json(json!({
            "strongAgainst": detail.strong_against,
            "weakAgainst": detail.weak_against,
            "teammateSynergies": detail.teammate_synergies
        })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// ---------------- LCU Handlers ----------------

async fn get_lcu_status(State(state): State<AppState>) -> Json<Value> {
    let phase = state.lcu_client.get_gameflow_phase().await;
    let session = if phase == "ChampSelect" {
        state.lcu_client.get_champ_select_session().await
    } else {
        Value::Null
    };

    Json(json!({
        "event": "LIVE_SYNC",
        "lcuConnected": phase != "None",
        "gamePhase": phase,
        "session": session
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApplySpellsReq {
    spell1_id: i64,
    spell2_id: i64,
}

async fn apply_summoner_spells(
    State(state): State<AppState>,
    Json(req): Json<ApplySpellsReq>,
) -> Json<Value> {
    let ok = state.lcu_client.apply_summoner_spells(req.spell1_id, req.spell2_id).await;
    Json(json!({
        "success": ok,
        "message": if ok { "召唤师技能已成功应用！" } else { "应用失败，请确保游戏客户端处于选人阶段" }
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MockLiveSyncReq {
    my_hero_id: String,
    #[serde(default)]
    bench_hero_ids: Vec<String>,
}

async fn mock_live_sync(
    State(state): State<AppState>,
    Json(req): Json<MockLiveSyncReq>,
) -> Json<Value> {
    let payload = json!({
        "event": "LIVE_SYNC",
        "lcuConnected": true,
        "gamePhase": "ChampSelect",
        "inChampSelect": true,
        "myHeroId": req.my_hero_id,
        "benchHeroIds": req.bench_hero_ids
    });
    state.ws_broadcaster.broadcast(&payload);
    Json(json!({ "status": "broadcast_sent", "payload": payload }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BenchSwapReq {
    champion_id: Value,
}

async fn swap_bench_champion(
    State(state): State<AppState>,
    Json(req): Json<BenchSwapReq>,
) -> Result<Json<Value>, StatusCode> {
    let cid = match req.champion_id {
        Value::String(s) => s,
        Value::Number(n) => n.to_string(),
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    if state.lcu_client.swap_bench_champion(&cid).await {
        Ok(Json(json!({ "status": "ok", "championId": cid })))
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

async fn reroll_champion(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    if state.lcu_client.reroll_champion().await {
        Ok(Json(json!({ "status": "ok" })))
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

// ---------------- Overlay Handlers ----------------

async fn trigger_ingame_overlay(State(state): State<AppState>) -> Json<Value> {
    state.ingame_overlay.trigger_scan();
    Json(json!({ "status": "triggered", "message": "游戏内置顶 HUD 弹窗已触发！" }))
}

async fn get_overlay_status(State(state): State<AppState>) -> Json<Value> {
    let cfg = state.ingame_overlay.config.read().await;
    Json(json!({
        "status": "ok",
        "autoSensingEnabled": cfg.auto_sensing_enabled,
        "voiceEnabled": cfg.voice_enabled,
        "hotkey": cfg.hotkey,
        "isShowing": state.ingame_overlay.is_showing.load(std::sync::atomic::Ordering::SeqCst)
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OverlayConfigReq {
    auto_sensing_enabled: Option<bool>,
    voice_enabled: Option<bool>,
    hotkey: Option<String>,
}

async fn update_overlay_config(
    State(state): State<AppState>,
    Json(req): Json<OverlayConfigReq>,
) -> Json<Value> {
    let mut cfg = state.ingame_overlay.config.write().await;
    if let Some(v) = req.auto_sensing_enabled {
        cfg.auto_sensing_enabled = v;
    }
    if let Some(v) = req.voice_enabled {
        cfg.voice_enabled = v;
    }
    if let Some(v) = req.hotkey {
        cfg.hotkey = v;
    }
    Json(json!({ "status": "updated", "config": &*cfg }))
}

async fn get_bench_overlay_status(State(state): State<AppState>) -> Json<Value> {
    let cfg = state.bench_overlay.config.read().await;
    Json(json!({
        "status": "ok",
        "enabled": cfg.enabled,
        "isShowing": state.bench_overlay.is_showing.load(std::sync::atomic::Ordering::SeqCst),
        "clickToSwap": cfg.click_to_swap,
        "badgePosition": cfg.badge_position,
        "verticalOffset": cfg.vertical_offset,
        "horizontalOffset": cfg.horizontal_offset
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BenchOverlayConfigReq {
    enabled: Option<bool>,
    click_to_swap: Option<bool>,
    badge_position: Option<String>,
    vertical_offset: Option<i32>,
    horizontal_offset: Option<i32>,
}

async fn update_bench_overlay_config(
    State(state): State<AppState>,
    Json(req): Json<BenchOverlayConfigReq>,
) -> Json<Value> {
    let mut cfg = state.bench_overlay.config.write().await;
    if let Some(v) = req.enabled {
        cfg.enabled = v;
    }
    if let Some(v) = req.click_to_swap {
        cfg.click_to_swap = v;
    }
    if let Some(v) = req.badge_position {
        cfg.badge_position = v;
    }
    if let Some(v) = req.vertical_offset {
        cfg.vertical_offset = v;
    }
    if let Some(v) = req.horizontal_offset {
        cfg.horizontal_offset = v;
    }
    Json(json!({ "status": "ok", "config": &*cfg }))
}

// ---------------- Item Overlay Handlers ----------------

async fn get_item_overlay_status(State(state): State<AppState>) -> Json<Value> {
    let rec = state.item_overlay.last_recommendation.read().await.clone();
    let cfg = state.item_overlay.config.read().await;
    Json(json!({
        "status": "ok",
        "enabled": cfg.enabled,
        "voiceEnabled": cfg.voice_enabled,
        "hotkey": cfg.hotkey,
        "isShowing": state.item_overlay.is_showing.load(std::sync::atomic::Ordering::SeqCst),
        "lastRecommendation": rec
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ItemOverlayConfigReq {
    enabled: Option<bool>,
    voice_enabled: Option<bool>,
    hotkey: Option<String>,
}

async fn update_item_overlay_config(
    State(state): State<AppState>,
    Json(req): Json<ItemOverlayConfigReq>,
) -> Json<Value> {
    let mut cfg = state.item_overlay.config.write().await;
    if let Some(v) = req.enabled {
        cfg.enabled = v;
    }
    if let Some(v) = req.voice_enabled {
        cfg.voice_enabled = v;
    }
    if let Some(v) = req.hotkey {
        cfg.hotkey = v;
    }
    Json(json!({ "status": "updated", "config": &*cfg }))
}

async fn trigger_item_overlay(State(state): State<AppState>) -> Json<Value> {
    state.item_overlay.trigger_manual_scan();
    Json(json!({ "status": "triggered", "message": "装备推荐 HUD 已触发！" }))
}

async fn get_item_preferences(State(state): State<AppState>) -> Json<Value> {
    let cfg = state.config_manager.get().await;
    Json(json!({
        "status": "ok",
        "preferences": cfg,
        "presets": get_preset_definitions()
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ItemPreferencesReq {
    resolve_conflicts: Option<bool>,
    hex_synergy_weight: Option<f64>,
    archetype_strictness: Option<f64>,
    counter_intensity: Option<f64>,
    gold_calc_mode: Option<String>,
    trio_mode: Option<String>,
    preset: Option<String>,
}

async fn update_item_preferences(
    State(state): State<AppState>,
    Json(req): Json<ItemPreferencesReq>,
) -> Json<Value> {
    let updated = state.config_manager.update(
        req.resolve_conflicts,
        req.hex_synergy_weight,
        req.archetype_strictness,
        req.counter_intensity,
        req.gold_calc_mode,
        req.trio_mode,
        req.preset,
    ).await;
    Json(json!({
        "status": "updated",
        "preferences": updated
    }))
}

#[derive(Deserialize)]
struct ItemRecQuery {
    augments: Option<String>,
    items: Option<String>,
    #[serde(default)]
    gold: f64,
    #[serde(default = "default_level")]
    level: i32,
}

fn default_level() -> i32 {
    6
}

async fn get_hero_item_recommendations(
    State(state): State<AppState>,
    AxPath(hero_id): AxPath<String>,
    Query(q): Query<ItemRecQuery>,
) -> Json<Value> {
    let augs: Vec<String> = q.augments.map(|s| s.split(',').map(|x| x.trim().to_string()).collect()).unwrap_or_default();
    let current_items: Vec<String> = q.items.map(|s| s.split(',').map(|x| x.trim().to_string()).collect()).unwrap_or_default();
    let cfg = state.config_manager.get().await;

    let res = DecisionEngine::get_item_recommendation(
        &state.db,
        &hero_id,
        &current_items,
        q.gold,
        q.level,
        &augs,
        &[],
        &cfg,
    );
    Json(res)
}

// ---------------- Ranked Handlers ----------------

async fn get_ranked_state(State(state): State<AppState>) -> Json<Value> {
    let draft = state.ranked_draft.read().await;
    Json(json!(draft.state))
}

async fn apply_ranked_runes(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let draft = state.ranked_draft.read().await;
    let runes = draft.state.decision_card.get("recommendedRunes").cloned().unwrap_or(Value::Null);
    if runes.is_null() {
        return Err(StatusCode::BAD_REQUEST);
    }
    if state.lcu_client.apply_perk_page(&runes).await {
        Ok(Json(json!({ "status": "ok", "message": "符文已成功写入客户端" })))
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

async fn apply_ranked_spells(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let draft = state.ranked_draft.read().await;
    if let Some(spells) = draft.state.decision_card.get("recommendedSpells") {
        let s1 = spells.get("spell1Id").and_then(|v| v.as_i64()).unwrap_or(4);
        let s2 = spells.get("spell2Id").and_then(|v| v.as_i64()).unwrap_or(14);
        if state.lcu_client.apply_summoner_spells(s1, s2).await {
            return Ok(Json(json!({ "status": "ok", "message": "召唤师技能已成功应用" })));
        }
    }
    Err(StatusCode::BAD_REQUEST)
}

async fn update_ranked_config(
    State(_state): State<AppState>,
    Json(_req): Json<Value>,
) -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn get_ranked_tiers(State(state): State<AppState>) -> Json<Value> {
    Json(state.opgg_client.get_ranked_tier_list(false).await)
}

#[derive(Deserialize)]
struct MatchupQuery {
    my_champ: String,
    enemy_champ: String,
    #[serde(default = "default_pos")]
    position: String,
}

fn default_pos() -> String {
    "TOP".to_string()
}

async fn get_ranked_matchup(
    State(state): State<AppState>,
    Query(q): Query<MatchupQuery>,
) -> Result<Json<Value>, StatusCode> {
    match state.opgg_client.get_matchup_win_rate(&q.my_champ, &q.enemy_champ, &q.position).await {
        Some(val) => Ok(Json(val)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Deserialize)]
struct CounterQuery {
    enemy_champ: String,
    #[serde(default = "default_pos")]
    position: String,
    top_n: Option<usize>,
}

async fn get_ranked_counters(
    State(state): State<AppState>,
    Query(q): Query<CounterQuery>,
) -> Json<Value> {
    Json(state.opgg_client.get_counter_picks_against(&q.enemy_champ, &q.position, q.top_n).await)
}

#[derive(Deserialize)]
struct SynergyQuery {
    champion: String,
    #[serde(default = "default_pos")]
    position: String,
}

async fn get_ranked_synergies(
    State(state): State<AppState>,
    Query(q): Query<SynergyQuery>,
) -> Json<Value> {
    Json(state.opgg_client.get_champion_synergies(&q.champion, &q.position).await)
}

async fn get_draft_decision(State(state): State<AppState>) -> Json<Value> {
    let draft = state.ranked_draft.read().await;
    Json(json!({
        "status": "ok",
        "is_draft_active": draft.state.is_draft_active,
        "decision_card": draft.state.decision_card
    }))
}

// ---------------- Data Export Handlers ----------------

async fn export_heroes_csv(State(state): State<AppState>) -> Result<Response, StatusCode> {
    match Exporter::export_heroes_summary_csv_bytes(&state.db) {
        Ok(bytes) => {
            let headers = [
                (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
                (header::CONTENT_DISPOSITION, "attachment; filename=\"heroes_summary.csv\""),
            ];
            Ok((headers, bytes).into_response())
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn export_augments_csv(State(state): State<AppState>) -> Result<Response, StatusCode> {
    match Exporter::export_augments_summary_csv_bytes(&state.db) {
        Ok(bytes) => {
            let headers = [
                (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
                (header::CONTENT_DISPOSITION, "attachment; filename=\"augments_summary.csv\""),
            ];
            Ok((headers, bytes).into_response())
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// ---------------- Overlay Test & Debug Handlers ----------------

async fn test_ranked_overlay(State(state): State<AppState>) -> Json<Value> {
    state.ranked_overlay.show_ranked_draft(crate::overlay::RankedOverlayData {
        my_team: vec![
            crate::overlay::RankedSlotView {
                position_cn: "上单".to_string(),
                champion_name: "德玛西亚之力".to_string(),
                win_rate: 0.528,
                tier: 1,
                is_local_player: true,
            },
            crate::overlay::RankedSlotView {
                position_cn: "打野".to_string(),
                champion_name: "盲僧".to_string(),
                win_rate: 0.512,
                tier: 2,
                is_local_player: false,
            },
            crate::overlay::RankedSlotView {
                position_cn: "中单".to_string(),
                champion_name: "九尾妖狐".to_string(),
                win_rate: 0.531,
                tier: 1,
                is_local_player: false,
            },
            crate::overlay::RankedSlotView {
                position_cn: "下路".to_string(),
                champion_name: "探险家".to_string(),
                win_rate: 0.495,
                tier: 3,
                is_local_player: false,
            },
            crate::overlay::RankedSlotView {
                position_cn: "辅助".to_string(),
                champion_name: "魂锁典狱长".to_string(),
                win_rate: 0.518,
                tier: 2,
                is_local_player: false,
            },
        ],
        their_team: vec![
            crate::overlay::RankedSlotView {
                position_cn: "上单".to_string(),
                champion_name: "诺克萨斯之手".to_string(),
                win_rate: 0.515,
                tier: 2,
                is_local_player: false,
            },
            crate::overlay::RankedSlotView {
                position_cn: "打野".to_string(),
                champion_name: "德邦总管".to_string(),
                win_rate: 0.508,
                tier: 2,
                is_local_player: false,
            },
            crate::overlay::RankedSlotView {
                position_cn: "中单".to_string(),
                champion_name: "影流之主".to_string(),
                win_rate: 0.498,
                tier: 3,
                is_local_player: false,
            },
            crate::overlay::RankedSlotView {
                position_cn: "下路".to_string(),
                champion_name: "虚空之女".to_string(),
                win_rate: 0.521,
                tier: 1,
                is_local_player: false,
            },
            crate::overlay::RankedSlotView {
                position_cn: "辅助".to_string(),
                champion_name: "曙光女神".to_string(),
                win_rate: 0.510,
                tier: 2,
                is_local_player: false,
            },
        ],
        recommendation: "👑 测试态 · 对位[诺克萨斯之手]: 胜率 52.8% · 推荐符文已就绪 · 技能: 闪现 + 点燃".to_string(),
    });

    Json(serde_json::json!({
        "success": true,
        "message": "已触发排位对位选人吸附悬浮窗 (持续30秒)"
    }))
}

async fn get_overlay_debug(State(state): State<AppState>) -> Json<Value> {
    let lol_rect = crate::vision::ScreenCapture::get_lol_window_rect();
    let is_league_active = crate::overlay::WindowTracker::is_league_active();
    let is_game_active = crate::overlay::WindowTracker::is_game_active();
    let phase = state.lcu_client.get_gameflow_phase().await;
    let is_showing = state.ranked_overlay.is_showing.load(std::sync::atomic::Ordering::SeqCst);
    let league_pid = crate::vision::capture::get_league_pid();

    Json(serde_json::json!({
        "lolWindowRect": lol_rect.map(|r| serde_json::json!({
            "x": r.x,
            "y": r.y,
            "width": r.width,
            "height": r.height
        })),
        "isLeagueActive": is_league_active,
        "isGameActive": is_game_active,
        "gameflowPhase": phase,
        "isRankedOverlayShowing": is_showing,
        "leaguePid": league_pid
    }))
}

