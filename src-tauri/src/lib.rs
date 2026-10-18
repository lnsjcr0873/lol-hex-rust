#![recursion_limit = "512"]

pub mod cmd;

use std::sync::Arc;
use lol_hex_rust::config::ConfigManager;
use lol_hex_rust::crawler::OpggClient;
use lol_hex_rust::data::loader::Database;
use lol_hex_rust::engine::{PlayerHistoryManager, RankedDraftManager};
use lol_hex_rust::lcu::LcuClient;
use lol_hex_rust::overlay::ranked_overlay::update_ranked_draft_overlay;
use lol_hex_rust::overlay::{win32_hud, BenchOverlay, RankedOverlay};
use lol_hex_rust::sgp::SgpClient;
use tokio::sync::RwLock;

pub struct TauriAppState {
    pub lcu: Arc<LcuClient>,
    pub history: Arc<PlayerHistoryManager>,
    pub db: Arc<Database>,
    pub config: Arc<ConfigManager>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = tracing_subscriber::fmt::try_init();

    // Note: Tauri and WRY configure DPI awareness natively; manual SetProcessDpiAwarenessContext causes WebView2 E_INVALIDARG (0x80070057)

    // 1. Initialize DB and Subsystems
    let mut initial_db = Database::load_from_dir("data");
    if initial_db.heroes_summary.is_empty() {
        initial_db = Database::load_from_dir("../data");
    }
    let db = Arc::new(initial_db);
    let lcu = Arc::new(LcuClient::new());
    let history = Arc::new(PlayerHistoryManager::new());
    let opgg = Arc::new(OpggClient::new());
    let sgp = Arc::new(SgpClient::new(lcu.clone()));
    let config = Arc::new(ConfigManager::new("recommendation_config.json"));
    let ranked_overlay = Arc::new(RankedOverlay::new());
    let bench_overlay = Arc::new(BenchOverlay::new());
    let draft_mgr = Arc::new(RwLock::new(RankedDraftManager::new()));

    // Warm up OP.GG tier list in background
    let opgg_warmup = opgg.clone();
    tauri::async_runtime::spawn(async move {
        let _ = opgg_warmup.get_ranked_tier_list(false).await;
    });

    // Start Win32 HUD Overlay engine & register single source of truth references
    ranked_overlay.start();
    bench_overlay.start();

    if let Ok(mut gh) = win32_hud::GLOBAL_HISTORY.write() {
        *gh = Some(history.clone());
    }
    if let Ok(mut gl) = win32_hud::GLOBAL_LCU.write() {
        *gl = Some(lcu.clone());
    }

    let history_db = history.clone();
    let db_clone = db.clone();
    let sgp_clone = sgp.clone();
    let lcu_clone = lcu.clone();
    let opgg_clone = opgg.clone();
    tauri::async_runtime::spawn(async move {
        lol_hex_rust::init_tokio_handle(tokio::runtime::Handle::current());
        history_db.set_database(db_clone).await;
        history_db.set_sgp_client(sgp_clone).await;
        lcu_clone.set_opgg(opgg_clone).await;
    });

    // 2. Background Automation Monitor (ReadyCheck, Overlay HUD, Auto-Bench-Swap, Auto-Apply-Runes)
    let lcu_bg = lcu.clone();
    let config_bg = config.clone();
    let db_bg = db.clone();
    let history_bg = history.clone();
    let opgg_bg = opgg.clone();
    let ranked_overlay_bg = ranked_overlay.clone();
    let bench_overlay_bg = bench_overlay.clone();
    let draft_bg = draft_mgr.clone();

    tauri::async_runtime::spawn(async move {
        let mut last_runes_champion_id: i64 = 0;
        let mut last_swapped_champion_id: i64 = 0;
        let mut last_swap_instant = std::time::Instant::now();
        let mut last_draft_sig = String::new();
        let mut cached_ranked_data: Option<lol_hex_rust::overlay::RankedOverlayData> = None;
        let mut accepting_ready_check = false;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
            let phase = lcu_bg.get_gameflow_phase().await;

            // 2.1 ReadyCheck: Auto-Accept Match (non-blocking async task to never stall main loop)
            if phase == "ReadyCheck" {
                let cfg = config_bg.get().await;
                if cfg.auto_accept_match && !accepting_ready_check {
                    if let Some(rc) = lcu_bg.get_ready_check().await {
                        let player_resp = rc.get("playerResponse").and_then(|v| v.as_str()).unwrap_or("");
                        let state = rc.get("state").and_then(|v| v.as_str()).unwrap_or("");
                        if player_resp != "Accepted" && player_resp != "Declined" && state == "InProgress" {
                            accepting_ready_check = true;
                            let lcu_accept = lcu_bg.clone();
                            let delay = cfg.auto_accept_delay_seconds.clamp(0, lol_hex_rust::config::MAX_AUTO_ACCEPT_DELAY_SECONDS);
                            tauri::async_runtime::spawn(async move {
                                if delay > 0 {
                                    tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
                                }
                                if let Some(rc_after) = lcu_accept.get_ready_check().await {
                                    let resp_after = rc_after.get("playerResponse").and_then(|v| v.as_str()).unwrap_or("");
                                    let state_after = rc_after.get("state").and_then(|v| v.as_str()).unwrap_or("");
                                    if resp_after != "Accepted" && resp_after != "Declined" && state_after == "InProgress" {
                                        let ok = lcu_accept.accept_ready_check().await;
                                        if ok {
                                            tracing::info!("⚡ [HexBrain Tauri] 自动接受对局成功 (延迟 {}s)", delay);
                                        }
                                    }
                                }
                            });
                        }
                    }
                }
            } else {
                accepting_ready_check = false;
            }

            // 2.2 ChampSelect: Win32 Overlay HUD, Auto-Swap Bench & Auto-Apply Runes
            if phase == "ChampSelect" {
                let cfg = config_bg.get().await;
                let session = lcu_bg.get_champ_select_session().await;
                if !session.is_null() {
                    let is_aram = session.get("benchEnabled").and_then(|v| v.as_bool()).unwrap_or(false)
                        || session.get("benchChampions").and_then(|b| b.as_array()).map_or(false, |b| !b.is_empty());

                    if is_aram {
                        last_draft_sig.clear();
                        cached_ranked_data = None;
                        if ranked_overlay_bg.is_showing.load(std::sync::atomic::Ordering::SeqCst) && !ranked_overlay_bg.is_in_test_mode() {
                            ranked_overlay_bg.stop();
                        }
                        if let Some(bench) = session.get("benchChampions").and_then(|b| b.as_array()) {
                            let mut views = Vec::new();
                            for (idx, champ) in bench.iter().enumerate() {
                                if let Some(cid) = champ.get("championId").and_then(|c| c.as_i64()) {
                                    let id_str = cid.to_string();
                                    let hero_name = db_bg.resolve_hero_name(&id_str).unwrap_or_else(|| format!("Hero-{}", cid));
                                    let wr = db_bg.heroes_map.get(&id_str).map(|s| s.win_rate).unwrap_or(0.5);
                                    let tier = if wr >= 0.52 { "S" } else if wr >= 0.50 { "A" } else if wr >= 0.48 { "B" } else { "C" };
                                    views.push(win32_hud::BenchChampView {
                                        slot_index: idx,
                                        champion_id: cid as i32,
                                        champion_name: hero_name,
                                        tier: tier.to_string(),
                                        win_rate: wr,
                                    });
                                }
                            }
                            if !views.is_empty() {
                                bench_overlay_bg.show_bench_heroes(views);
                            }
                        }
                    } else {
                        // 5v5 Ranked / Normal Draft -> Update Win32 HUD Overlay
                        if bench_overlay_bg.is_showing.load(std::sync::atomic::Ordering::SeqCst) {
                            bench_overlay_bg.stop();
                        }
                        update_ranked_draft_overlay(
                            &session,
                            &draft_bg,
                            &ranked_overlay_bg,
                            &lcu_bg,
                            &history_bg,
                            &opgg_bg,
                            &db_bg,
                            &mut last_draft_sig,
                            &mut cached_ranked_data,
                        ).await;
                    }

                    let local_cell_id = session.get("localPlayerCellId").and_then(|v| v.as_i64()).unwrap_or(-1);
                    let my_team = session.get("myTeam").and_then(|v| v.as_array());

                    let mut my_champion_id: i64 = 0;
                    if let Some(team) = my_team {
                        for member in team {
                            let cell = member.get("cellId").and_then(|v| v.as_i64()).unwrap_or(-99);
                            if cell == local_cell_id {
                                my_champion_id = member.get("championId")
                                    .or_else(|| member.get("championPickIntent"))
                                    .and_then(|v| v.as_i64())
                                    .unwrap_or(0);
                                break;
                            }
                        }
                    }

                    // A. ARAM Bench Auto Swap
                    if is_aram && cfg.auto_swap_bench && !cfg.bench_preferred_champions.is_empty() {
                        if let Some(bench) = session.get("benchChampions").and_then(|v| v.as_array()) {
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
                                    let allow_swap = pref_id != last_swapped_champion_id || last_swap_instant.elapsed() > std::time::Duration::from_secs(3);
                                    if bench_ids.contains(&pref_id) && allow_swap {
                                        let swapped = lcu_bg.swap_bench_champion(&pref_id.to_string()).await;
                                        if swapped {
                                            tracing::info!("⚡ [HexBrain Tauri] 自动秒抢备战席心愿英雄: {}", pref_id);
                                            last_swapped_champion_id = pref_id;
                                            last_swap_instant = std::time::Instant::now();
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // B. Auto Apply Runes & Summoner Spells (Requires Explicit User Opt-In)
                    if cfg.auto_apply_runes && my_champion_id > 0 && my_champion_id != last_runes_champion_id {
                        let is_bench_enabled = session.get("benchEnabled").and_then(|v| v.as_bool()).unwrap_or(false);
                        let is_locked = if is_bench_enabled {
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
                            let (ok, msg) = cmd::do_apply_runes_and_spells(&lcu_bg, &db_bg, my_champion_id).await;
                            if ok {
                                tracing::info!("⚡ [HexBrain Tauri] 自动配置权威胜率符文与技能成功: {} ({})", my_champion_id, msg);
                                last_runes_champion_id = my_champion_id;
                            }
                        }
                    }
                }
            } else {
                if last_runes_champion_id != 0 {
                    last_runes_champion_id = 0;
                }
                if last_swapped_champion_id != 0 {
                    last_swapped_champion_id = 0;
                }
                last_draft_sig.clear();
                cached_ranked_data = None;
                if bench_overlay_bg.is_showing.load(std::sync::atomic::Ordering::SeqCst) {
                    bench_overlay_bg.stop();
                }
                if ranked_overlay_bg.is_showing.load(std::sync::atomic::Ordering::SeqCst) && !ranked_overlay_bg.is_in_test_mode() {
                    ranked_overlay_bg.stop();
                }
            }
        }
    });

    tauri::Builder::default()
        .manage(TauriAppState {
            lcu,
            history,
            db,
            config,
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            cmd::get_ongoing_match,
            cmd::send_chat_records,
            cmd::get_summoner_profile,
            cmd::get_match_history,
            cmd::get_raw_match_history,
            cmd::get_champion_mastery,
            cmd::get_match_detail,
            cmd::get_raw_match_detail,
            cmd::get_match_timeline,
            cmd::get_raw_match_timeline,
            cmd::get_automation_config,
            cmd::update_automation_config,
            cmd::apply_runes_and_spells,
            cmd::search_summoner,
            cmd::toggle_ongoing_overlay,
            cmd::set_ongoing_overlay_visible,
            cmd::set_ongoing_overlay_always_on_top,
            cmd::is_ongoing_overlay_visible
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "ongoing-overlay" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running hexbrain tauri application");
}
