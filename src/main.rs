#![recursion_limit = "512"]

mod config;
mod crawler;
mod data;
mod engine;
mod lcu;
mod overlay;
mod sgp;
mod vision;
mod web;

use config::ConfigManager;
use crawler::{HexdataCrawler, OpggClient};
use data::loader::Database;
use engine::ranked_draft::RankedDraftManager;
use lcu::{LcuClient, LiveClientDataReader};
use overlay::{BenchOverlay, HotkeyManager, InGameOverlay, ItemOverlay, RankedOverlay, TextToSpeech};
use serde_json::Value;
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

pub use lol_hex_rust::{get_tokio_handle, init_tokio_handle, spawn_task, GLOBAL_TOKIO_HANDLE};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    // 1. Cache Tokio runtime handle so non-Tokio worker threads (Win32 Hook, GDI loop) can safely spawn async tasks
    init_tokio_handle(tokio::runtime::Handle::current());

    // 2. Global panic hook to log panics gracefully without silent crashes
    std::panic::set_hook(Box::new(|info| {
        eprintln!("💥 HexBrain Caught Panic: {}", info);
        tracing::error!("💥 HexBrain Caught Panic: {}", info);
    }));

    println!("============================================================");
    println!("⚡ HexBrain - 英雄联盟海克斯大乱斗决策系统 (Rust 原生版) ⚡");
    println!("基于 1157万+ 局真实对局大数据的海克斯大乱斗实时决策辅助引擎");
    println!("💡 局内感知提示: 请确保英雄联盟客户端视频设置设为【无边框窗口化】");
    println!("💡 热键说明: 局内按【或 [ 随时推演海克斯强化，按 】或 ] 查看动态出装");
    println!("============================================================");

    // Set Per-Monitor V2 DPI Awareness to ensure pixel-perfect HUD alignment on 2K/4K scaling
    #[cfg(windows)]
    unsafe {
        #[link(name = "user32")]
        extern "system" {
            fn SetProcessDpiAwarenessContext(value: isize) -> i32;
        }
        let _ = SetProcessDpiAwarenessContext(-4);
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
    let config_manager = Arc::new(ConfigManager::new("recommendation_config.json"));
    let lcu_client = Arc::new(LcuClient::new());
    let live_client = Arc::new(LiveClientDataReader::new());
    let ranked_draft = Arc::new(RwLock::new(RankedDraftManager::new()));
    let bench_overlay = Arc::new(BenchOverlay::new());
    let ranked_overlay = Arc::new(RankedOverlay::new());
    let ingame_overlay = Arc::new(InGameOverlay::new());
    let opgg_client = Arc::new(OpggClient::new());
    let item_overlay = Arc::new(ItemOverlay::new());
    item_overlay.set_database(db.clone());
    item_overlay.set_opgg(opgg_client.clone());
    if let Ok(mut gio) = overlay::win32_hud::GLOBAL_ITEM_OVERLAY.write() {
        *gio = Some(item_overlay.clone());
    }

    // 3. Track current game mode (ARAM vs Summoner's Rift vs Hextech) for hotkeys & overlays
    let is_aram_active = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let is_hextech_active = Arc::new(std::sync::atomic::AtomicBool::new(false));

    // 4. Start Overlays
    bench_overlay.start();
    ranked_overlay.start();
    ingame_overlay.start();
    ingame_overlay.start_auto_sensing_loop(
        db.clone(),
        live_client.clone(),
        lcu_client.clone(),
        is_hextech_active.clone(),
    );
    item_overlay.start();

    // 5. Start Global Hotkeys
    let ingame_clone = ingame_overlay.clone();
    let item_clone = item_overlay.clone();
    let db_for_hotkey = db.clone();
    let is_hextech_for_hotkey = is_hextech_active.clone();
    let hotkeys = HotkeyManager::new();
    hotkeys.start(
        move || {
            let is_hextech = is_hextech_for_hotkey.load(std::sync::atomic::Ordering::SeqCst);
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
                        is_hex_flag.store(true, std::sync::atomic::Ordering::SeqCst);
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

    // 6. Background LCU & Game Polling Task
    let player_history = Arc::new(crate::engine::PlayerHistoryManager::new());
    player_history.set_database(db.clone()).await;
    let sgp_client = Arc::new(crate::sgp::SgpClient::new(lcu_client.clone()));
    player_history.set_sgp_client(sgp_client.clone()).await;
    lcu_client.set_opgg(opgg_client.clone()).await;
    if let Ok(mut gh) = crate::overlay::win32_hud::GLOBAL_HISTORY.write() {
        *gh = Some(player_history.clone());
    }
    if let Ok(mut gl) = crate::overlay::win32_hud::GLOBAL_LCU.write() {
        *gl = Some(lcu_client.clone());
    }

    // 6.1 Initialize and Launch Embedded Web UI (League-Akari Style Realtime Match + History + Settings)
    let ws_broadcaster = Arc::new(crate::web::WsBroadcaster::new());
    let web_state = crate::web::AppState {
        db: db.clone(),
        lcu: lcu_client.clone(),
        history: player_history.clone(),
        config: config_manager.clone(),
        item_overlay: item_overlay.clone(),
        ingame_overlay: ingame_overlay.clone(),
        bench_overlay: bench_overlay.clone(),
        broadcaster: ws_broadcaster.clone(),
    };
    tokio::spawn(async move {
        if let Err(e) = crate::web::start_web_server(web_state, 8765).await {
            tracing::error!("Embedded Web Server failed: {}", e);
        }
    });

    let lcu_bg = lcu_client.clone();
    let live_bg = live_client.clone();
    let item_bg = item_overlay.clone();
    let draft_bg = ranked_draft.clone();
    let db_bg = db.clone();
    let bench_bg = bench_overlay.clone();
    let ranked_bg = ranked_overlay.clone();
    let opgg_bg = opgg_client.clone();
    let history_bg = player_history.clone();
    let ingame_bg = ingame_overlay.clone();
    let hex_bg = hexdata_client.clone();
    let is_aram_bg = is_aram_active.clone();
    let is_hextech_bg = is_hextech_active.clone();
    let config_bg = config_manager.clone();
    let ws_bg = ws_broadcaster.clone();

    // Warm up OP.GG tier list in background
    let opgg_warmup = opgg_client.clone();
    tokio::spawn(async move {
        let _ = opgg_warmup.get_ranked_tier_list(false).await;
    });

    tokio::spawn(async move {
        let mut last_level = 0i32;
        let mut last_draft_sig = String::new();
        let mut cached_ranked_data: Option<crate::overlay::RankedOverlayData> = None;
        let mut last_applied_runes_cid = 0i64;
        let mut swapped_bench_cids: std::collections::HashSet<String> = std::collections::HashSet::new();
        let ready_check_accepting = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            let phase = lcu_bg.get_gameflow_phase().await;

            // 0. Auto-Accept Matchmaking Ready Check (decoupled to avoid blocking main loop)
            if phase == "ReadyCheck" {
                let cfg = config_bg.get().await;
                if cfg.auto_accept_match {
                    if let Some(rc) = lcu_bg.get_ready_check().await {
                        let player_resp = rc.get("playerResponse").and_then(|v| v.as_str()).unwrap_or("");
                        let state = rc.get("state").and_then(|v| v.as_str()).unwrap_or("");
                        if player_resp != "Accepted" && player_resp != "Declined" && state == "InProgress" {
                            let delay = cfg.auto_accept_delay_seconds.clamp(0, 10);
                            if !ready_check_accepting.swap(true, std::sync::atomic::Ordering::SeqCst) {
                                let lcu_c = lcu_bg.clone();
                                let ws_c = ws_bg.clone();
                                let flag = ready_check_accepting.clone();
                                tokio::spawn(async move {
                                    if delay > 0 {
                                        tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
                                    }
                                    let cur_phase = lcu_c.get_gameflow_phase().await;
                                    if cur_phase == "ReadyCheck" {
                                        let success = lcu_c.accept_ready_check().await;
                                        if success {
                                            tracing::info!("⚡ [HexBrain LCU] 对局已就绪！已自动为您确认接受对局 (延迟 {} 秒)", delay);
                                            ws_c.send_event("auto_accept", serde_json::json!({
                                                "status": "accepted",
                                                "delaySeconds": delay
                                            }));
                                        }
                                    }
                                    flag.store(false, std::sync::atomic::Ordering::SeqCst);
                                });
                            }
                        }
                    }
                }
            } else {
                ready_check_accepting.store(false, std::sync::atomic::Ordering::SeqCst);
            }

            if phase != "ChampSelect" {
                last_applied_runes_cid = 0;
                swapped_bench_cids.clear();
                if bench_bg.is_showing.load(std::sync::atomic::Ordering::SeqCst) {
                    bench_bg.stop();
                }
            }

            if phase == "ChampSelect" {
                let session = lcu_bg.get_champ_select_session().await;
                if !session.is_null() {
                    let mut d = draft_bg.write().await;
                    d.parse_session_with_db_and_opgg(&session, Some(&db_bg), Some(&opgg_bg));

                    // 1. If benchEnabled or non-empty bench -> ARAM mode -> BenchOverlay HUD
                    let is_aram = session.get("benchEnabled").and_then(|v| v.as_bool()).unwrap_or(false)
                        || session.get("benchChampions").and_then(|b| b.as_array()).map_or(false, |b| !b.is_empty());
                    is_aram_bg.store(is_aram, std::sync::atomic::Ordering::SeqCst);

                    if is_aram {
                        last_draft_sig.clear();
                        cached_ranked_data = None;
                        if ranked_bg.is_showing.load(std::sync::atomic::Ordering::SeqCst) && !ranked_bg.is_in_test_mode() {
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

                            // ARAM Auto-Swap Bench Fast Grab
                            let cfg = config_bg.get().await;
                            if cfg.auto_swap_bench && !cfg.bench_preferred_champions.is_empty() {
                                let local_cell_id = session.get("localPlayerCellId").and_then(|v| v.as_i64()).unwrap_or(-1);
                                let my_cid_str = session.get("myTeam")
                                    .and_then(|t| t.as_array())
                                    .and_then(|arr| {
                                        arr.iter()
                                            .find(|m| m.get("cellId").and_then(|c| c.as_i64()) == Some(local_cell_id))
                                            .and_then(|m| m.get("championId").or_else(|| m.get("championPickIntent")))
                                            .map(|v| match v {
                                                Value::String(s) => s.clone(),
                                                Value::Number(n) => n.to_string(),
                                                _ => String::new(),
                                            })
                                    })
                                    .unwrap_or_default();

                                for pref_raw in &cfg.bench_preferred_champions {
                                    let pref_id_str = db_bg.resolve_hero_id(pref_raw).unwrap_or_else(|| pref_raw.clone());
                                    if !my_cid_str.is_empty() && my_cid_str == pref_id_str {
                                        break;
                                    }
                                    if swapped_bench_cids.contains(&pref_id_str) {
                                        continue;
                                    }
                                    let on_bench = bench.iter().any(|b| {
                                        b.get("championId")
                                            .map(|v| match v {
                                                Value::String(s) => s == &pref_id_str,
                                                Value::Number(n) => n.to_string() == pref_id_str,
                                                _ => false,
                                            })
                                            .unwrap_or(false)
                                    });
                                    if on_bench {
                                        if lcu_bg.swap_bench_champion(&pref_id_str).await {
                                            swapped_bench_cids.insert(pref_id_str.clone());
                                            let pref_name = db_bg.resolve_hero_name(&pref_id_str).unwrap_or_else(|| format!("英雄{}", pref_id_str));
                                            tracing::info!("⚡ [HexBrain ARAM] 心愿英雄已出现在待选席！已极速为您秒抢：{}", pref_name);
                                            ws_bg.send_event("auto_swap_bench", serde_json::json!({
                                                "status": "swapped",
                                                "championId": pref_id_str,
                                                "championName": pref_name
                                            }));
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    } else if !d.state.my_team.is_empty() {
                        if bench_bg.is_showing.load(std::sync::atomic::Ordering::SeqCst) {
                            bench_bg.stop();
                        }

                        // Auto-apply recommended runes for locked champion
                        let cfg = config_bg.get().await;
                        if cfg.auto_apply_runes {
                            let local_cell_id = session.get("localPlayerCellId").and_then(|v| v.as_i64()).unwrap_or(-1);
                            let is_locked = session.get("actions").and_then(|v| v.as_array()).map(|action_rounds| {
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
                            }).unwrap_or(false);

                            if is_locked {
                                let local_slot = d.state.my_team.iter().find(|s| s.is_local_player);
                                if let Some(local) = local_slot {
                                    if !local.champion_id.is_empty() {
                                        if let Ok(cid_i64) = local.champion_id.parse::<i64>() {
                                            if cid_i64 > 0 && cid_i64 != last_applied_runes_cid {
                                                last_applied_runes_cid = cid_i64;
                                                let (success, msg) = lcu_bg.do_apply_runes_and_spells(&db_bg, cid_i64, &local.position).await;
                                                if success {
                                                    tracing::info!("⚡ [HexBrain LCU] 已自动为【{}】配置版本推荐符文天赋与召唤师技能", local.champion_name);
                                                    ws_bg.send_event("auto_apply_runes", serde_json::json!({
                                                        "status": "applied",
                                                        "championId": cid_i64,
                                                        "championName": local.champion_name,
                                                        "message": msg
                                                    }));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
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
                    }

                }
            } else if phase == "GameStart" {
                if bench_bg.is_showing.load(std::sync::atomic::Ordering::SeqCst) {
                    bench_bg.stop();
                }

                let is_aram = is_aram_bg.load(std::sync::atomic::Ordering::SeqCst);
                if is_aram {
                    // ARAM Mode: Cleanly dismiss draft overlays immediately upon entering loading screen!
                    if ranked_bg.is_showing.load(std::sync::atomic::Ordering::SeqCst) && !ranked_bg.is_in_test_mode() {
                        ranked_bg.stop();
                    }
                    crate::overlay::win32_hud::GLOBAL_HUD.hide();
                    continue;
                }

                // If 10 players are already resolved and in cache, keep displaying without spamming LCU
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
                    // Loading screen before game 2999 port is active: dismiss draft HUDs immediately
                    let is_aram = is_aram_bg.load(std::sync::atomic::Ordering::SeqCst);
                    if is_aram {
                        if ranked_bg.is_showing.load(std::sync::atomic::Ordering::SeqCst) && !ranked_bg.is_in_test_mode() {
                            ranked_bg.stop();
                        }
                        crate::overlay::win32_hud::GLOBAL_HUD.hide();
                    }
                } else {
                    if live_stats.hero_id.is_empty() && !live_stats.hero_name.is_empty() {
                        if let Some(resolved) = db_bg.resolve_hero_id(&live_stats.hero_name) {
                            live_stats.hero_id = resolved;
                        }
                    }

                    is_aram_bg.store(live_stats.is_aram, std::sync::atomic::Ordering::SeqCst);
                    let is_hextech_mode = live_stats.has_hextech;
                    is_hextech_bg.store(is_hextech_mode, std::sync::atomic::Ordering::SeqCst);

                    if live_stats.is_aram {
                        // ARAM mode: transition from ranked overlay to augments
                        if ranked_bg.is_showing.load(std::sync::atomic::Ordering::SeqCst) && !ranked_bg.is_in_test_mode() {
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
                                tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                                ingame_auto.trigger_scan_with_db(&db_auto);
                            });
                        }
                        last_level = current_lvl;

                    } else {
                        // Summoner's Rift: keep ranked overlay for loading/early game
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
                last_draft_sig.clear();
                cached_ranked_data = None;
                if last_level > 0 {
                    last_level = 0;
                    is_aram_bg.store(false, std::sync::atomic::Ordering::SeqCst);
                    is_hextech_bg.store(false, std::sync::atomic::Ordering::SeqCst);
                }
                if bench_bg.is_showing.load(std::sync::atomic::Ordering::SeqCst) {
                    bench_bg.stop();
                }
                if ranked_bg.is_showing.load(std::sync::atomic::Ordering::SeqCst) && !ranked_bg.is_in_test_mode() {
                    ranked_bg.stop();
                }
            }
        }
    });

    println!("🚀 HexBrain 原生电竞守护引擎已就绪！");
    println!("💡 免切屏置顶 HUD: [ 键三选一推演 · ] 键自适应出装推荐");
    println!("🎮 选人阶段自动激活大乱斗备战席与排位对位克制 · 按 Ctrl+C 退出程序");

    tokio::signal::ctrl_c().await?;
    println!("⚡ HexBrain 正在安全退出...");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::recipe_data::calculate_item_net_cost;
    use crate::engine::decision::calc_wilson_lower_bound;
    use crate::vision::OcrScanner;

    #[test]
    fn test_wilson_lower_bound() {
        let wr = 0.54;
        let games = 1000;
        let lower = calc_wilson_lower_bound(wr, games, 1.96);
        assert!(lower > 0.50);
        assert!(lower < wr);
    }

    #[test]
    fn test_net_cost_calculation() {
        // Target: Rabadon's Deathcap (3089), total 3600g
        // Owned: Needlessly Large Rod (1058), price 1250g
        let owned = vec!["1058".to_string()];
        let res = calculate_item_net_cost("3089", &owned, 1500.0, "NET_DEDUCT");
        assert_eq!(res.total_cost, 3600);
        assert_eq!(res.deducted_gold, 1250);
        assert_eq!(res.net_cost, 2350);
        assert!(!res.is_affordable); // 1500 < 2350
    }

    #[test]
    fn test_clean_chinese_text() {
        let raw = "沃 格 勒 特 的 巫 师 帽";
        let cleaned = OcrScanner::clean_chinese_text(raw);
        assert_eq!(cleaned, "沃格勒特的巫师帽");
    }

    #[test]
    fn test_database_loads_embedded() {
        let db = Database::load_from_dir("data");
        assert!(!db.heroes_summary.is_empty(), "Heroes summary should not be empty");
        assert!(!db.augments_summary.is_empty(), "Augments summary should not be empty");
        assert!(db.resolve_hero_id("金克丝").is_some());
        let annie_id = db.resolve_hero_id("黑暗之女");
        assert!(annie_id.is_some());
        if let Some(id) = annie_id {
            assert_eq!(db.resolve_hero_name(&id), Some("黑暗之女".to_string()));
        }
    }

    #[test]
    fn test_lenient_number_deserialization() {
        #[derive(serde::Deserialize)]
        struct Sample {
            #[serde(deserialize_with = "crate::data::models::deserialize_f64_lenient")]
            rate: f64,
            #[serde(deserialize_with = "crate::data::models::deserialize_i64_lenient")]
            count: i64,
        }

        let json1 = r#"{"rate": "54.2%", "count": "1,250"}"#;
        let s1: Sample = serde_json::from_str(json1).unwrap();
        assert!((s1.rate - 0.542).abs() < 1e-4);
        assert_eq!(s1.count, 1250);

        let json2 = r#"{"rate": null, "count": "15.0"}"#;
        let s2: Sample = serde_json::from_str(json2).unwrap();
        assert_eq!(s2.rate, 0.0);
        assert_eq!(s2.count, 15);
    }

    #[test]
    fn test_nested_recipe_deduction() {
        // Target: 6657 requires 3802 (Lost Chapter) + 3108 (Fiendish Codex)
        // If player has 1052 (Amplifying Tome, 400g) which is a sub-component of 3802
        let owned = vec!["1052".to_string()];
        let res = calculate_item_net_cost("6657", &owned, 1000.0, "NET_DEDUCT");
        assert!(res.deducted_gold >= 400);
        assert_eq!(res.matched_components.len(), 1);
        assert_eq!(res.matched_components[0].slot, "nested");
    }

    #[tokio::test]
    async fn test_config_nan_safety() {
        let temp_dir = std::env::temp_dir().join("hexbrain_test_cfg");
        let cfg_path = temp_dir.join("config.json");
        let mgr = crate::config::ConfigManager::new(cfg_path.to_str().unwrap());
        // Pass f64::NAN
        let updated = mgr.update(None, Some(f64::NAN), Some(f64::NAN), Some(f64::INFINITY), None, None, None, None, None).await;
        assert!(!updated.hex_synergy_weight.is_nan());
        assert!(!updated.archetype_strictness.is_nan());
        assert!(!updated.counter_intensity.is_infinite());
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_wilson_nan_and_extremes() {
        assert_eq!(calc_wilson_lower_bound(f64::NAN, 100, 1.96), 0.5);
        assert_eq!(calc_wilson_lower_bound(0.6, 0, 1.96), 0.6);
        let low_zero = calc_wilson_lower_bound(0.0, 100, 1.96);
        assert!(low_zero >= 0.0);
        let high_one = calc_wilson_lower_bound(1.0, 100, 1.96);
        assert!(high_one <= 1.0);
    }

    #[test]
    fn test_counter_item_threat_detection() {
        use serde_json::json;
        // Test with Dr. Mundo, Soraka, and Leona
        let enemies = vec![
            json!({ "championName": "DrMundo" }),
            json!({ "championName": "众星之子" }),
        ];
        let threat = crate::engine::counter::CounterItemEngine::analyze_enemy_threats(&enemies);
        assert!(threat.has_high_heal, "DrMundo and Soraka should trigger has_high_heal");
        assert!(threat.has_super_tank, "DrMundo should trigger has_super_tank");
        let arch = crate::engine::archetype::ArchetypeInfo::default();
        let urgent = crate::engine::counter::CounterItemEngine::get_urgent_counter_ids(&threat, &arch);
        assert!(urgent.contains("3165") || urgent.contains("3033"), "Should recommend anti-heal");
    }

    #[test]
    fn test_synergy_nan_safety() {
        use serde_json::json;
        let candidates = vec![
            json!({ "itemId": "3089", "tacticalScore": 60.0 }), // Deathcap (AP)
        ];
        let augments = vec![
            json!({ "name": "珠光护手", "description": "法术伤害可以暴击" }),
        ];
        let owned = std::collections::HashSet::new();
        let reranked = crate::engine::synergy::HexItemSynergyEngine::rerank_items_with_synergy(
            &candidates,
            &augments,
            &owned,
            "1",
            f64::NAN, // test NaN weight
            true,
        );
        assert_eq!(reranked.len(), 1);
        let score = reranked[0]["tacticalScore"].as_f64().unwrap();
        assert!(!score.is_nan());
        assert!(score >= 60.0);
    }

    #[test]
    fn test_ocr_substring_and_anvil() {
        let db = Database::load_from_dir("data");
        // Test anvil detection
        let anvil_lines = vec!["选择一个属性铁砧".to_string(), "+10 法术强度".to_string()];
        assert!(OcrScanner::is_stat_anvil_cards(&anvil_lines));

        // Test augment match with extra punctuation/whitespace
        let lines = vec!["《珠 光 护 手》".to_string()];
        let matches = OcrScanner::match_augments_from_texts(&db, &lines);
        assert!(!matches.is_empty(), "Should match 珠光护手 despite guillemets and spaces");
    }

    #[test]
    fn test_ultrawide_viewport_math() {
        use crate::vision::ScreenCapture;
        let vp = ScreenCapture::get_augment_title_band_rect();
        assert!(vp.width > 0);
        assert!(vp.height > 0);
    }

    #[test]
    fn test_lcu_credentials_parsing() {
        use crate::lcu::LcuConnector;
        // Test lockfile parsing
        let lockfile_sample = "LeagueClient:1234:56789:my_secret_token_123:https";
        let creds = LcuConnector::parse_lockfile(lockfile_sample);
        assert!(creds.is_some());
        let c = creds.unwrap();
        assert_eq!(c.port, 56789);
        assert_eq!(c.auth_token, "my_secret_token_123");

        // Test commandline parsing
        let cmdline_sample = "\"C:\\Riot\\LeagueClientUx.exe\" \"--app-port=51234\" \"--remoting-auth-token=token_abc_xyz\" \"--riotclient-auth-port=1234\"";
        let cmd_creds = LcuConnector::parse_commandline(cmdline_sample);
        assert!(cmd_creds.is_some());
        let cc = cmd_creds.unwrap();
        assert_eq!(cc.port, 51234);
        assert_eq!(cc.auth_token, "token_abc_xyz");
    }

    #[tokio::test]
    async fn test_overlay_state_and_config_mutation() {
        let overlay = InGameOverlay::new();
        assert!(!overlay.is_showing.load(std::sync::atomic::Ordering::SeqCst));

        // Test mutating config via RwLock
        {
            let mut cfg = overlay.config.write().await;
            cfg.voice_enabled = false;
            cfg.hotkey = "F1".to_string();
        }
        {
            let cfg = overlay.config.read().await;
            assert!(!cfg.voice_enabled);
            assert_eq!(cfg.hotkey, "F1");
        }

        // Test triggering scan
        overlay.trigger_scan();
        assert!(overlay.is_showing.load(std::sync::atomic::Ordering::SeqCst));
        overlay.stop();
        assert!(!overlay.is_showing.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn test_win32_native_hud_state_transitions() {
        use crate::overlay::win32_hud::{AugmentCardView, AugmentOverlayData, NativeHud};
        use std::time::Duration;

        let hud = NativeHud::new();
        let cards = vec![AugmentCardView {
            name: "珠光护手".to_string(),
            tier: "S".to_string(),
            win_rate: 0.584,
            delta_win_rate: 0.036,
            wilson_lower: 0.552,
            games: 15000,
            recommendation: "首选核心输出".to_string(),
        }];

        let data = AugmentOverlayData {
            hero_name: "黑暗之女".to_string(),
            cards,
        };

        hud.show_augments(data, Duration::from_secs(5));
        assert!(hud.is_visible());

        hud.hide();
        assert!(!hud.is_visible());
    }

    #[test]
    fn test_database_resolve_hero_name() {
        let db = Database::load_from_dir("data");
        let name = db.resolve_hero_name("1");
        assert!(name.is_some(), "Should resolve hero id 1");
        assert_eq!(name.unwrap(), "黑暗之女");
    }

    #[test]
    fn test_exporter_bom_and_bytes() {
        let db = Database::load_from_dir("data");
        let bytes = crate::crawler::Exporter::export_heroes_summary_csv_bytes(&db).unwrap();
        // Check UTF-8 BOM
        assert!(bytes.len() >= 3);
        assert_eq!(&bytes[0..3], b"\xEF\xBB\xBF", "CSV export must prepend UTF-8 BOM for Windows Excel compatibility");
        let text = String::from_utf8(bytes[3..].to_vec()).unwrap();
        assert!(text.contains("英雄ID"), "Header should be in CSV");
        assert!(text.contains("黑暗之女"), "Data should contain Chinese hero names");

        let aug_bytes = crate::crawler::Exporter::export_augments_summary_csv_bytes(&db).unwrap();
        assert_eq!(&aug_bytes[0..3], b"\xEF\xBB\xBF");
        let aug_text = String::from_utf8(aug_bytes[3..].to_vec()).unwrap();
        assert!(aug_text.contains("强化ID"));
    }

    #[test]
    fn test_terminal_trio_alignment() {
        use serde_json::json;
        use std::collections::HashSet;

        let trios = vec![
            json!({
                "itemIds": ["3089", "3157", "3020"],
                "itemNames": ["灭世者的死亡之帽", "中娅沙漏", "法师之靴"],
                "winRate": 0.58,
                "games": 500
            }),
            json!({
                "itemIds": ["3031", "3094", "3006"],
                "itemNames": ["无尽之刃", "疾射火炮", "狂战士胫甲"],
                "winRate": 0.59,
                "games": 400
            }),
        ];

        let mut owned = HashSet::new();
        owned.insert("3089".to_string()); // Player already owns Deathcap

        let arch = crate::engine::archetype::ArchetypeInfo {
            locked_archetype_id: "BURST_MAGE".to_string(),
            favored_tags: vec!["ABILITY_POWER".to_string()],
            is_locked: true,
            ..Default::default()
        };

        let aligned = crate::engine::archetype::ArchetypeEngine::align_terminal_trio_with_archetype(
            &trios,
            &arch,
            &owned,
            &[],
            true,
            "DYNAMIC_COUNTER",
        );

        assert!(aligned.is_some());
        let res = aligned.unwrap();
        let item_ids = res["itemIds"].as_array().unwrap();
        assert_eq!(item_ids[0].as_str().unwrap(), "3089", "Should pick the AP trio matching owned Deathcap");
        assert_eq!(res["ownedMatches"].as_i64().unwrap(), 1);
    }

    #[test]
    fn test_compare_augments_trio_synergy() {
        let db = Database::load_from_dir("data");
        // Compare augments for Annie (1) with an existing augment
        // If Annie has augments, pass existing_augments and check comparison
        let res = crate::engine::decision::DecisionEngine::compare_augments(
            &db,
            "1",
            &["1238".to_string(), "1068".to_string(), "1124".to_string()],
            Some(1),
            &["珠光护手".to_string()],
        );

        assert!(res.get("candidates").is_some());
        let candidates = res["candidates"].as_array().unwrap();
        assert!(!candidates.is_empty());
        // Verify each candidate has hasTrioSynergy field
        for c in candidates {
            assert!(c.get("hasTrioSynergy").is_some());
        }
    }

    #[test]
    fn test_ranked_draft_session_parsing_with_db() {
        use serde_json::json;
        let db = Database::load_from_dir("data");
        let mut mgr = crate::engine::ranked_draft::RankedDraftManager::new();

        let session = json!({
            "myTeam": [
                { "cellId": 0, "championId": 1, "assignedPosition": "middle" },
                { "cellId": 1, "championId": 0, "assignedPosition": "top" }
            ],
            "theirTeam": [
                { "cellId": 5, "championId": "1", "assignedPosition": "utility" }
            ],
            "localPlayerCellId": 0
        });

        mgr.parse_session_with_db(&session, Some(&db));
        assert!(mgr.state.is_draft_active);
        assert_eq!(mgr.state.my_team.len(), 2);
        assert_eq!(mgr.state.my_team[0].champion_name, "黑暗之女");
        assert!(mgr.state.my_team[0].is_local_player);
        assert_eq!(mgr.state.my_team[1].champion_name, "队友2");
        assert_eq!(mgr.state.their_team[0].champion_name, "黑暗之女");
    }

    #[test]
    fn test_ranked_overlay_hud_state_transitions() {
        use crate::overlay::win32_hud::{NativeHud, RankedOverlayData, RankedSlotView};
        use std::time::Duration;

        let hud = NativeHud::new();
        let data = RankedOverlayData {
            my_team: vec![
                RankedSlotView {
                    position_cn: "中单".to_string(),
                    champion_name: "黑暗之女".to_string(),
                    win_rate: 0.53,
                    tier: 1,
                    is_local_player: true,
                    ..Default::default()
                },
            ],
            their_team: vec![
                RankedSlotView {
                    position_cn: "中单".to_string(),
                    champion_name: "劫".to_string(),
                    win_rate: 0.49,
                    tier: 3,
                    is_local_player: false,
                    ..Default::default()
                },
            ],
            decision_card: Default::default(),
            recommendation: "对线刺客，注意保留控制".to_string(),
        };

        hud.show_ranked(data, Duration::from_secs(5));
        assert!(hud.is_visible());
        hud.hide();
        assert!(!hud.is_visible());
    }

    #[test]
    fn test_database_arc_hero_detail() {
        let db = Database::new();
        let detail = std::sync::Arc::new(crate::data::models::HeroDetail {
            hero_id: serde_json::Value::String("1".to_string()),
            hero_name: "黑暗之女".to_string(),
            ..Default::default()
        });
        if let Ok(mut map) = db.heroes_detail.write() {
            map.insert("1".to_string(), std::sync::Arc::clone(&detail));
            map.insert("黑暗之女".to_string(), std::sync::Arc::clone(&detail));
        }

        let detail_by_id = db.get_hero_detail("1");
        assert!(detail_by_id.is_some(), "Hero detail for Annie (1) should exist");
        let detail_by_name = db.get_hero_detail("黑暗之女");
        assert!(detail_by_name.is_some(), "Hero detail by Chinese name should exist");

        // Verify Arc pointer sharing (zero data copy across index keys)
        assert!(
            std::sync::Arc::ptr_eq(&detail_by_id.unwrap(), &detail_by_name.unwrap()),
            "Database should share the exact same Arc heap instance across keys"
        );
    }

    #[tokio::test]
    async fn test_opgg_client_fallback_and_cache() {
        let opgg = crate::crawler::OpggClient::new();
        assert_eq!(opgg.get_primary_position("24").as_deref(), Some("TOP"));
        assert_eq!(opgg.get_primary_position("64").as_deref(), Some("JUNGLE"));
        assert!(opgg.get_matchup_win_rate("", "", "MID").await.is_none());
    }

    #[tokio::test]
    async fn test_hexdata_crawler_and_lazy_load() {
        let mut db = Database::load_from_dir("data");
        if db.heroes_summary.is_empty() {
            db = Database::load_from_dir("../data");
        }
        let crawler = crate::crawler::HexdataCrawler::new();

        let initial_len = db.heroes_detail.read().unwrap().len();
        if let Some(detail) = db.fetch_and_cache_hero_detail("1", &crawler).await {
            assert_eq!(detail.hero_name, "黑暗之女");
            let after_len = db.heroes_detail.read().unwrap().len();
            assert!(after_len > initial_len);
        }
    }

    #[tokio::test]
    async fn test_akari_match_history_and_radar() {
        let db = Database::new();
        let history = crate::engine::PlayerHistoryManager::new();

        let mock_match = history.get_mock_ongoing_match(&db);
        assert_eq!(mock_match.blue_team.len(), 5);
        assert_eq!(mock_match.red_team.len(), 5);
        assert!(mock_match.is_active);

        // Check duo premade on blue team
        assert_eq!(mock_match.blue_team[1].premade_tag.as_deref(), Some("双排车队"));
        assert_eq!(mock_match.blue_team[3].premade_tag.as_deref(), Some("双排车队"));

        // Check radar computation
        let player = &mock_match.blue_team[0];
        let radar = &player.tactical_analysis.radar;
        assert!(radar.contains_key("damage"));
        assert!(radar.contains_key("kda"));
        assert!(radar.contains_key("kp"));
        assert!(radar.contains_key("survival"));
        assert!(radar.contains_key("economy"));
        assert!(radar.contains_key("vision"));

        for (k, v) in radar {
            assert!(*v >= 25 && *v <= 99, "Radar {} value {} should be within [25, 99]", k, v);
        }
    }

    #[test]
    fn test_premade_team_deduction() {
        let mut players = vec![
            crate::engine::player_history::OngoingPlayerCard {
                slot_index: 0,
                puuid: "p1".to_string(),
                matches: vec![
                    crate::engine::player_history::ParsedMatchRecord {
                        teammate_puuids: vec!["p2".to_string()],
                        ..Default::default()
                    },
                    crate::engine::player_history::ParsedMatchRecord {
                        teammate_puuids: vec!["p2".to_string()],
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            crate::engine::player_history::OngoingPlayerCard {
                slot_index: 1,
                puuid: "p2".to_string(),
                matches: vec![
                    crate::engine::player_history::ParsedMatchRecord {
                        teammate_puuids: vec!["p1".to_string()],
                        ..Default::default()
                    },
                    crate::engine::player_history::ParsedMatchRecord {
                        teammate_puuids: vec!["p1".to_string()],
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            crate::engine::player_history::OngoingPlayerCard {
                slot_index: 2,
                puuid: "p3".to_string(),
                matches: vec![],
                ..Default::default()
            },
        ];

        crate::engine::player_history::detect_premade_teams(&mut players);
        assert_eq!(players[0].premade_tag.as_deref(), Some("双排车队"));
        assert_eq!(players[1].premade_tag.as_deref(), Some("双排车队"));
        assert_eq!(players[2].premade_tag, None);
    }

    #[test]
    fn test_static_asset_path_normalization() {
        let uri_path = "/static/css/style.css";
        let mut path = uri_path.trim_start_matches('/');
        if let Some(stripped) = path.strip_prefix("static/") {
            path = stripped;
        }
        assert_eq!(path, "css/style.css");
        assert!(std::path::Path::new(path).extension().is_some());

        // Test SPA route without extension
        let spa_path = "/ongoingMatch";
        let mut spath = spa_path.trim_start_matches('/');
        if let Some(stripped) = spath.strip_prefix("static/") {
            spath = stripped;
        }
        assert_eq!(spath, "ongoingMatch");
        assert!(std::path::Path::new(spath).extension().is_none());
    }

    #[tokio::test]
    async fn test_player_history_chinese_hero_name_resolution() {
        let db = Arc::new(crate::data::loader::Database::load_from_dir("data"));
        let mgr = crate::engine::player_history::PlayerHistoryManager::new();
        mgr.set_database(db.clone()).await;

        let res_db = mgr.get_database().await;
        assert!(res_db.is_some());
        let hero_name = res_db.unwrap().resolve_hero_name("266");
        assert_eq!(hero_name.as_deref(), Some("暗裔剑魔"));
    }

    #[test]
    fn test_compare_augments_engine_fallback() {
        let db = crate::data::loader::Database::load_from_dir("data");
        let cands = vec!["1113".to_string(), "1053".to_string(), "2095".to_string()];
        let res = crate::engine::decision::DecisionEngine::compare_augments(&db, "266", &cands, Some(1), &[]);
        assert!(res.get("candidates").is_some());
        assert!(res.get("recommendationRationale").is_some());
    }

    #[tokio::test]
    async fn test_auto_accept_config_toggle() {
        let temp_dir = std::env::temp_dir().join("hexbrain_test_auto_accept_cfg");
        let cfg_path = temp_dir.join("config.json");
        let mgr = crate::config::ConfigManager::new(cfg_path.to_str().unwrap());
        assert!(!mgr.get().await.auto_accept_match);
        let updated = mgr.set_auto_accept(true, Some(3)).await;
        assert!(updated.auto_accept_match);
        assert_eq!(updated.auto_accept_delay_seconds, 3);
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}

