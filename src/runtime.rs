use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::info;

use crate::config::ConfigManager;
use crate::crawler::{HexdataCrawler, OpggClient};
use crate::data::loader::Database;
use crate::engine::ranked_draft::RankedDraftManager;
use crate::engine::PlayerHistoryManager;
use crate::lcu::{LcuClient, LiveClientDataReader};
use crate::overlay::{
    win32_hud, BenchOverlay, HotkeyManager, InGameOverlay, ItemOverlay, RankedOverlay, TextToSpeech,
};
use crate::sgp::SgpClient;
use crate::web::WsBroadcaster;

#[derive(Clone, Default)]
pub struct BackgroundEngineOptions {
    pub enable_hotkeys: bool,
    pub ws_broadcaster: Option<Arc<WsBroadcaster>>,
}

pub struct BackgroundEngine {
    pub db: Arc<Database>,
    pub lcu: Arc<LcuClient>,
    pub history: Arc<PlayerHistoryManager>,
    pub config: Arc<ConfigManager>,
    pub opgg: Arc<OpggClient>,
    pub live: Arc<LiveClientDataReader>,
    pub draft_mgr: Arc<RwLock<RankedDraftManager>>,
    pub bench_overlay: Arc<BenchOverlay>,
    pub ranked_overlay: Arc<RankedOverlay>,
    pub ingame_overlay: Arc<InGameOverlay>,
    pub item_overlay: Arc<ItemOverlay>,
    pub hexdata_client: Arc<HexdataCrawler>,
    pub hotkeys: Option<Arc<HotkeyManager>>,
    pub ws_broadcaster: Option<Arc<WsBroadcaster>>,
    pub is_aram_active: Arc<AtomicBool>,
    pub is_hextech_active: Arc<AtomicBool>,
}

/// Initializes and starts the unified HexBrain background engine:
/// - In-memory Database loading with packaged data and online crawler fallback
/// - Subsystems (LCU client, SGP client, OP.GG client, Player History, 2999 Live telemetry)
/// - Overlays (Bench HUD, Ranked 5v5 HUD, InGame Augment OCR HUD, Item Recommendation HUD)
/// - Global Hotkeys (`[` for Augments scan, `]` for Dynamic Item Shop recommendations)
/// - Background gameflow monitor loop (800ms) with atomic ReadyCheck auto-accept,
///   ARAM bench auto-swap (with 3s cooldown & priority break), auto runes, and in-game milestone TTS.
pub async fn start_background_engine(options: BackgroundEngineOptions) -> BackgroundEngine {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        crate::init_tokio_handle(handle);
    }

    // 1. Initialize In-Memory Database: Prioritize local packaged data (Instant 0ms, Zero WAF 403)
    let mut initial_db = Database::load_from_dir("data");
    if initial_db.heroes_summary.is_empty() {
        initial_db = Database::load_from_dir("../data");
    }
    let hexdata_client = Arc::new(HexdataCrawler::new());
    if initial_db.heroes_summary.is_empty() {
        // Fallback to online API only if local files are completely missing
        initial_db.sync_from_api(&hexdata_client).await;
    }
    let db = Arc::new(initial_db);

    // Warm up Hexdata session handshake in background
    let hex_warmup = hexdata_client.clone();
    tokio::spawn(async move {
        hex_warmup.ensure_handshake().await;
    });

    // 2. Initialize Subsystems
    let config = Arc::new(ConfigManager::new("recommendation_config.json"));
    let lcu = Arc::new(LcuClient::new());
    let live = Arc::new(LiveClientDataReader::new());
    let draft_mgr = Arc::new(RwLock::new(RankedDraftManager::new()));
    let bench_overlay = Arc::new(BenchOverlay::new());
    let ranked_overlay = Arc::new(RankedOverlay::new());
    let ingame_overlay = Arc::new(InGameOverlay::new());
    let opgg = Arc::new(OpggClient::new());
    let item_overlay = Arc::new(ItemOverlay::new());
    item_overlay.set_database(db.clone());
    item_overlay.set_opgg(opgg.clone());
    if let Ok(mut gio) = win32_hud::GLOBAL_ITEM_OVERLAY.write() {
        *gio = Some(item_overlay.clone());
    }

    // 3. Track game modes for hotkeys & overlays
    let is_aram_active = Arc::new(AtomicBool::new(false));
    let is_hextech_active = Arc::new(AtomicBool::new(false));

    // 4. Start Overlays
    bench_overlay.start();
    ranked_overlay.start();
    ingame_overlay.start();
    ingame_overlay.start_auto_sensing_loop(
        db.clone(),
        live.clone(),
        lcu.clone(),
        is_hextech_active.clone(),
    );
    item_overlay.start();

    // 5. Start Global Hotkeys
    let hotkeys = if options.enable_hotkeys {
        let ingame_clone = ingame_overlay.clone();
        let item_clone = item_overlay.clone();
        let db_for_hotkey = db.clone();
        let is_hextech_for_hotkey = is_hextech_active.clone();
        let hk = Arc::new(HotkeyManager::new());
        hk.start(
            move || {
                let is_hextech = is_hextech_for_hotkey.load(Ordering::SeqCst);
                info!("Hotkey '[' pressed: is_hextech = {}", is_hextech);
                if is_hextech {
                    info!("Triggering InGame Augment Scan (Hextech Mode)");
                    ingame_clone.trigger_scan_with_db(&db_for_hotkey);
                    TextToSpeech::speak("海克斯强化推演已激活");
                } else {
                    let live_reader = LiveClientDataReader::new();
                    let ingame_fallback = ingame_clone.clone();
                    let db_fallback = db_for_hotkey.clone();
                    let is_hex_flag = is_hextech_for_hotkey.clone();
                    crate::spawn_task(async move {
                        let stats = live_reader.get_live_stats().await;
                        if stats.is_in_game && stats.has_hextech {
                            is_hex_flag.store(true, Ordering::SeqCst);
                            info!("Fast live probe confirmed Hextech mode! Triggering scan...");
                            ingame_fallback.trigger_scan_with_db(&db_fallback);
                            TextToSpeech::speak("海克斯强化推演已激活");
                        } else if stats.is_in_game {
                            info!("Hotkey '[' pressed: Non-hextech mode active, skipping Hextech scan");
                            TextToSpeech::speak("当前对局非海克斯强化模式，海克斯推荐仅在斗魂竞技场或海克斯大乱斗生效");
                        } else {
                            info!("Hotkey '[' pressed outside match, attempting scan fallback");
                            ingame_fallback.trigger_scan_with_db(&db_fallback);
                            TextToSpeech::speak("正在尝试感知当前卡片画面");
                        }
                    });
                }
            },
            move || {
                info!("Hotkey ']' pressed: Triggering Item Recommendation HUD");
                item_clone.trigger_manual_scan();
            },
        );
        Some(hk)
    } else {
        None
    };

    // 6. Connect Player History and SGP
    let player_history = Arc::new(PlayerHistoryManager::new());
    player_history.set_database(db.clone()).await;
    let sgp_client = Arc::new(SgpClient::new(lcu.clone()));
    player_history.set_sgp_client(sgp_client.clone()).await;
    lcu.set_opgg(opgg.clone()).await;
    if let Ok(mut gh) = win32_hud::GLOBAL_HISTORY.write() {
        *gh = Some(player_history.clone());
    }
    if let Ok(mut gl) = win32_hud::GLOBAL_LCU.write() {
        *gl = Some(lcu.clone());
    }

    // Warm up OP.GG tier list in background
    let opgg_warmup = opgg.clone();
    tokio::spawn(async move {
        let _ = opgg_warmup.get_ranked_tier_list(false).await;
    });

    // 7. Background LCU & Game Polling Task
    let lcu_bg = lcu.clone();
    let live_bg = live.clone();
    let item_bg = item_overlay.clone();
    let draft_bg = draft_mgr.clone();
    let db_bg = db.clone();
    let bench_bg = bench_overlay.clone();
    let ranked_bg = ranked_overlay.clone();
    let opgg_bg = opgg.clone();
    let history_bg = player_history.clone();
    let ingame_bg = ingame_overlay.clone();
    let hex_bg = hexdata_client.clone();
    let is_aram_bg = is_aram_active.clone();
    let is_hextech_bg = is_hextech_active.clone();
    let config_bg = config.clone();
    let ws_bg = options.ws_broadcaster.clone();

    tokio::spawn(async move {
        let mut last_level = 0i32;
        let mut last_draft_sig = String::new();
        let mut cached_ranked_data: Option<crate::overlay::RankedOverlayData> = None;
        let mut last_applied_runes_cid = 0i64;
        let mut last_swapped_champion_id = 0i64;
        let mut last_swap_instant = Instant::now() - Duration::from_secs(10);
        let ready_check_accepting = Arc::new(AtomicBool::new(false));

        loop {
            tokio::time::sleep(Duration::from_millis(800)).await;
            let phase = lcu_bg.get_gameflow_phase().await;

            // 7.1 Auto-Accept Matchmaking Ready Check (decoupled to avoid blocking main loop)
            if phase == "ReadyCheck" {
                let cfg = config_bg.get().await;
                if cfg.auto_accept_match {
                    if let Some(rc) = lcu_bg.get_ready_check().await {
                        let player_resp = rc.get("playerResponse").and_then(|v| v.as_str()).unwrap_or("");
                        let state = rc.get("state").and_then(|v| v.as_str()).unwrap_or("");
                        if player_resp != "Accepted" && player_resp != "Declined" && state == "InProgress" {
                            let delay = cfg.auto_accept_delay_seconds.clamp(0, crate::config::MAX_AUTO_ACCEPT_DELAY_SECONDS);
                            if !ready_check_accepting.swap(true, Ordering::SeqCst) {
                                let lcu_c = lcu_bg.clone();
                                let ws_c = ws_bg.clone();
                                let flag = ready_check_accepting.clone();
                                tokio::spawn(async move {
                                    if delay > 0 {
                                        tokio::time::sleep(Duration::from_secs(delay)).await;
                                    }
                                    let cur_phase = lcu_c.get_gameflow_phase().await;
                                    if cur_phase == "ReadyCheck" {
                                        if let Some(rc_after) = lcu_c.get_ready_check().await {
                                            let resp_after = rc_after.get("playerResponse").and_then(|v| v.as_str()).unwrap_or("");
                                            let state_after = rc_after.get("state").and_then(|v| v.as_str()).unwrap_or("");
                                            if resp_after != "Accepted" && resp_after != "Declined" && state_after == "InProgress" {
                                                let success = lcu_c.accept_ready_check().await;
                                                if success {
                                                    info!("⚡ [HexBrain LCU] 对局已就绪！已自动为您确认接受对局 (延迟 {} 秒)", delay);
                                                    if let Some(ws) = &ws_c {
                                                        ws.send_event("auto_accept", serde_json::json!({
                                                            "status": "accepted",
                                                            "delaySeconds": delay
                                                        }));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    flag.store(false, Ordering::SeqCst);
                                });
                            }
                        }
                    }
                }
            } else {
                ready_check_accepting.store(false, Ordering::SeqCst);
            }

            if phase == "ChampSelect" {
                let session = lcu_bg.get_champ_select_session().await;
                if !session.is_null() {
                    let mut d = draft_bg.write().await;
                    d.parse_session_with_db_and_opgg(&session, Some(&db_bg), Some(&opgg_bg));

                    // Check ARAM mode vs Summoner's Rift
                    let is_aram = session.get("benchEnabled").and_then(|v| v.as_bool()).unwrap_or(false)
                        || session.get("benchChampions").and_then(|b| b.as_array()).map_or(false, |b| !b.is_empty());
                    is_aram_bg.store(is_aram, Ordering::SeqCst);

                    let local_cell_id = session.get("localPlayerCellId").and_then(|v| v.as_i64()).unwrap_or(-1);
                    let my_team = session.get("myTeam").and_then(|v| v.as_array());
                    let mut my_champion_id: i64 = 0;
                    let mut my_assigned_pos = "all".to_string();
                    if let Some(team) = my_team {
                        for member in team {
                            let cell = member.get("cellId").and_then(|v| v.as_i64()).unwrap_or(-99);
                            if cell == local_cell_id {
                                my_champion_id = member.get("championId")
                                    .or_else(|| member.get("championPickIntent"))
                                    .and_then(|v| v.as_i64())
                                    .unwrap_or(0);
                                if let Some(assigned) = member.get("assignedPosition").and_then(|v| v.as_str()) {
                                    if !assigned.is_empty() {
                                        my_assigned_pos = assigned.to_string();
                                    }
                                }
                                break;
                            }
                        }
                    }

                    if is_aram {
                        last_draft_sig.clear();
                        cached_ranked_data = None;
                        if ranked_bg.is_showing.load(Ordering::SeqCst) && !ranked_bg.is_in_test_mode() {
                            ranked_bg.stop();
                        }
                        if let Some(bench) = session.get("benchChampions").and_then(|b| b.as_array()) {
                            let mut views = Vec::new();
                            for (idx, champ) in bench.iter().enumerate() {
                                if let Some(cid) = champ.get("championId").and_then(|c| c.as_i64()) {
                                    let id_str = cid.to_string();
                                    let hero_name = db_bg.resolve_hero_name(&id_str).unwrap_or_else(|| format!("Hero-{}", cid));
                                    let wr = db_bg.heroes_map.get(&id_str).map(|s| s.win_rate).unwrap_or(0.5);
                                    let tier = if wr >= 0.52 { "S" } else if wr >= 0.50 { "A" } else if wr >= 0.48 { "B" } else { "C" };
                                    views.push(crate::overlay::BenchChampView {
                                        slot_index: idx,
                                        champion_id: cid as i32,
                                        champion_name: hero_name,
                                        tier: tier.to_string(),
                                        win_rate: wr,
                                    });
                                }
                            }
                            if !views.is_empty() {
                                bench_bg.show_bench_heroes(views);
                            }

                            // ARAM Auto-Swap Bench Fast Grab (with 3-sec cooldown & break when holding preferred champion)
                            let cfg = config_bg.get().await;
                            if cfg.auto_swap_bench && !cfg.bench_preferred_champions.is_empty() {
                                if last_swap_instant.elapsed() >= Duration::from_secs(3) {
                                    let bench_ids: Vec<i64> = bench.iter()
                                        .filter_map(|b| b.get("championId").and_then(|v| v.as_i64()))
                                        .collect();

                                    for pref_str in &cfg.bench_preferred_champions {
                                        let pref_id_opt = pref_str.parse::<i64>().ok().or_else(|| {
                                            db_bg.resolve_hero_id(pref_str).and_then(|s| s.parse::<i64>().ok())
                                        });
                                        if let Some(pref_id) = pref_id_opt {
                                            if my_champion_id == pref_id {
                                                break;
                                            }
                                            if bench_ids.contains(&pref_id) {
                                                let swapped = lcu_bg.swap_bench_champion(&pref_id.to_string()).await;
                                                if swapped {
                                                    let pref_name = db_bg.resolve_hero_name(&pref_id.to_string()).unwrap_or_else(|| format!("英雄{}", pref_id));
                                                    info!("⚡ [HexBrain ARAM] 自动秒抢备战席心愿英雄: {} ({})", pref_name, pref_id);
                                                    last_swapped_champion_id = pref_id;
                                                    last_swap_instant = Instant::now();
                                                    if let Some(ws) = &ws_bg {
                                                        ws.send_event("auto_swap_bench", serde_json::json!({
                                                            "status": "swapped",
                                                            "championId": pref_id.to_string(),
                                                            "championName": pref_name
                                                        }));
                                                    }
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else if !d.state.my_team.is_empty() {
                        if bench_bg.is_showing.load(Ordering::SeqCst) {
                            bench_bg.stop();
                        }
                        drop(d);
                        crate::overlay::ranked_overlay::update_ranked_draft_overlay(
                            &session,
                            &draft_bg,
                            &ranked_bg,
                            &lcu_bg,
                            &history_bg,
                            &opgg_bg,
                            &db_bg,
                            &mut last_draft_sig,
                            &mut cached_ranked_data,
                        ).await;
                    } else {
                        drop(d);
                    }

                    // Auto apply runes & summoner spells for locked champion
                    let cfg = config_bg.get().await;
                    if cfg.auto_apply_runes && my_champion_id > 0 && my_champion_id != last_applied_runes_cid {
                        let is_locked = if is_aram {
                            true
                        } else {
                            session.get("actions").and_then(|v| v.as_array()).map(|action_rounds| {
                                action_rounds.iter().any(|round| {
                                    round.as_array().map(|actions| {
                                        actions.iter().any(|a| {
                                            let actor = a.get("actorCellId").and_then(|v| v.as_i64()).unwrap_or(-1);
                                            let atype = a.get("type").and_then(|v| v.as_str()).unwrap_or("");
                                            let completed = a.get("completed").and_then(|v| v.as_bool()).unwrap_or(false);
                                            actor == local_cell_id && atype == "pick" && completed
                                        })
                                    }).unwrap_or(false)
                                })
                            }).unwrap_or(false)
                        };

                        if is_locked {
                            let (success, msg) = lcu_bg.do_apply_runes_and_spells(&db_bg, my_champion_id, &my_assigned_pos).await;
                            if success {
                                last_applied_runes_cid = my_champion_id;
                                let champ_name = db_bg.resolve_hero_name(&my_champion_id.to_string()).unwrap_or_else(|| format!("英雄{}", my_champion_id));
                                info!("⚡ [HexBrain LCU] 已自动为【{}】配置版本推荐符文天赋与技能: {}", champ_name, msg);
                                if let Some(ws) = &ws_bg {
                                    ws.send_event("auto_apply_runes", serde_json::json!({
                                        "status": "applied",
                                        "championId": my_champion_id,
                                        "championName": champ_name,
                                        "message": msg
                                    }));
                                }
                            }
                        }
                    }
                }
            } else if phase == "GameStart" {
                if bench_bg.is_showing.load(Ordering::SeqCst) {
                    bench_bg.stop();
                }

                let is_aram = is_aram_bg.load(Ordering::SeqCst);
                if is_aram {
                    if ranked_bg.is_showing.load(Ordering::SeqCst) && !ranked_bg.is_in_test_mode() {
                        ranked_bg.stop();
                    }
                    win32_hud::GLOBAL_HUD.hide();
                    continue;
                }

                if let Some(ref cached) = cached_ranked_data {
                    let total_ready = cached.my_team.iter().filter(|s| s.profile.is_some()).count()
                        + cached.their_team.iter().filter(|s| s.profile.is_some()).count();
                    if total_ready >= 8 {
                        ranked_bg.show_ranked_draft(cached.clone());
                        continue;
                    }
                }

                let gf_session = lcu_bg.get_gameflow_session().await;
                if !gf_session.is_null() {
                    if let Some((my_team_info, their_team_info)) = history_bg.resolve_gameflow_10_players(&gf_session, &lcu_bg).await {
                        let mut my_slots = Vec::new();
                        for (idx, s) in my_team_info.into_iter().enumerate() {
                            let cname = if s.champion_id > 0 {
                                db_bg.resolve_hero_name(&s.champion_id.to_string()).unwrap_or_else(|| s.summoner_name.clone())
                            } else if !s.summoner_name.is_empty() {
                                s.summoner_name.clone()
                            } else {
                                format!("队友{}", idx + 1)
                            };
                            let wr = s.profile.as_ref().map(|p| if p.cur_champ_win_rate_pct > 0 { p.cur_champ_win_rate_pct as f64 / 100.0 } else { p.win_rate_pct as f64 / 100.0 }).unwrap_or(0.5);
                            my_slots.push(crate::overlay::RankedSlotView {
                                slot_index: idx,
                                position: s.position,
                                position_cn: s.position_cn,
                                champion_name: cname,
                                champion_id: s.champion_id,
                                win_rate: wr,
                                tier: 3,
                                is_local_player: s.is_local_player,
                                is_locked: true,
                                is_manual_pos: false,
                                matchup_win_rate: None,
                                profile: s.profile,
                                counter_heroes: Vec::new(),
                                synergy_heroes: Vec::new(),
                            });
                        }

                        let mut their_slots = Vec::new();
                        for (idx, s) in their_team_info.into_iter().enumerate() {
                            let cname = if s.champion_id > 0 {
                                db_bg.resolve_hero_name(&s.champion_id.to_string()).unwrap_or_else(|| s.summoner_name.clone())
                            } else if !s.summoner_name.is_empty() {
                                s.summoner_name.clone()
                            } else {
                                format!("对手{}", idx + 1)
                            };
                            let wr = s.profile.as_ref().map(|p| if p.cur_champ_win_rate_pct > 0 { p.cur_champ_win_rate_pct as f64 / 100.0 } else { p.win_rate_pct as f64 / 100.0 }).unwrap_or(0.5);
                            their_slots.push(crate::overlay::RankedSlotView {
                                slot_index: idx,
                                position: s.position,
                                position_cn: s.position_cn,
                                champion_name: cname,
                                champion_id: s.champion_id,
                                win_rate: wr,
                                tier: 3,
                                is_local_player: false,
                                is_locked: true,
                                is_manual_pos: false,
                                matchup_win_rate: None,
                                profile: s.profile,
                                counter_heroes: Vec::new(),
                                synergy_heroes: Vec::new(),
                            });
                        }

                        let overlay_data = crate::overlay::RankedOverlayData {
                            my_team: my_slots,
                            their_team: their_slots,
                            decision_card: Default::default(),
                            recommendation: "对局加载中 · 双方10人实战画像已同步".to_string(),
                        };
                        ranked_bg.show_ranked_draft(overlay_data.clone());
                        cached_ranked_data = Some(overlay_data);
                    }
                }
            } else if phase == "InProgress" {
                // In-Game Live 2999 Port Monitoring Workflow
                let mut live_stats = live_bg.get_live_stats().await;
                if !live_stats.is_in_game {
                    let is_aram = is_aram_bg.load(Ordering::SeqCst);
                    if is_aram {
                        if ranked_bg.is_showing.load(Ordering::SeqCst) && !ranked_bg.is_in_test_mode() {
                            ranked_bg.stop();
                        }
                        win32_hud::GLOBAL_HUD.hide();
                    }
                } else {
                    if live_stats.hero_id.is_empty() && !live_stats.hero_name.is_empty() {
                        if let Some(resolved) = db_bg.resolve_hero_id(&live_stats.hero_name) {
                            live_stats.hero_id = resolved;
                        }
                    }

                    is_aram_bg.store(live_stats.is_aram, Ordering::SeqCst);
                    let is_hextech_mode = live_stats.has_hextech;
                    is_hextech_bg.store(is_hextech_mode, Ordering::SeqCst);

                    if live_stats.is_aram {
                        if ranked_bg.is_showing.load(Ordering::SeqCst) && !ranked_bg.is_in_test_mode() {
                            ranked_bg.stop();
                        }
                        if !live_stats.hero_id.is_empty() && db_bg.get_hero_detail(&live_stats.hero_id).is_none() {
                            let crawler = hex_bg.clone();
                            let db_clone = db_bg.clone();
                            let hero_id = live_stats.hero_id.clone();
                            tokio::spawn(async move {
                                let _ = db_clone.fetch_and_cache_hero_detail(&hero_id, &crawler).await;
                            });
                        }

                        let current_lvl = live_stats.level;
                        let is_milestone = if is_hextech_mode {
                            if last_level == 0 {
                                matches!(current_lvl, 3 | 4 | 7 | 11 | 15)
                            } else if current_lvl > last_level {
                                [3, 7, 11, 15].iter().any(|&m| last_level < m && current_lvl >= m)
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if is_milestone {
                            info!("Hextech level milestone reached: Level {}! Triggering augment readiness", current_lvl);
                            TextToSpeech::speak(&format!("已升至{}级，海克斯强化节点已就绪", current_lvl));
                            let ingame_auto = ingame_bg.clone();
                            let db_auto = db_bg.clone();
                            tokio::spawn(async move {
                                tokio::time::sleep(Duration::from_millis(1500)).await;
                                ingame_auto.trigger_scan_with_db(&db_auto);
                            });
                        }
                        last_level = current_lvl;
                    } else {
                        last_level = live_stats.level;
                    }

                    let assigned_pos = {
                        let d = draft_bg.read().await;
                        d.state.my_team.iter().find(|s| s.is_local_player).map(|s| s.position.clone()).unwrap_or_default()
                    };

                    item_bg.update_recommendations_from_live(
                        &db_bg,
                        &opgg_bg,
                        live_stats.is_aram,
                        &live_stats.hero_id,
                        &live_stats.hero_name,
                        &assigned_pos,
                        live_stats.gold,
                        &live_stats.current_items,
                        &live_stats.active_augments,
                        &live_stats.enemy_team,
                        live_stats.game_time,
                    ).await;
                }
            } else {
                // Out of match: Lobby, Matchmaking, EndOfGame, etc. -> Clean up!
                last_applied_runes_cid = 0;
                last_swapped_champion_id = 0;
                last_draft_sig.clear();
                cached_ranked_data = None;
                if last_level > 0 {
                    last_level = 0;
                    is_aram_bg.store(false, Ordering::SeqCst);
                    is_hextech_bg.store(false, Ordering::SeqCst);
                }
                if bench_bg.is_showing.load(Ordering::SeqCst) {
                    bench_bg.stop();
                }
                if ranked_bg.is_showing.load(Ordering::SeqCst) && !ranked_bg.is_in_test_mode() {
                    ranked_bg.stop();
                }
            }
        }
    });

    BackgroundEngine {
        db,
        lcu,
        history: player_history,
        config,
        opgg,
        live,
        draft_mgr,
        bench_overlay,
        ranked_overlay,
        ingame_overlay,
        item_overlay,
        hexdata_client,
        hotkeys,
        ws_broadcaster: options.ws_broadcaster,
        is_aram_active,
        is_hextech_active,
    }
}
