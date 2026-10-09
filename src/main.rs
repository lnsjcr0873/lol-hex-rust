mod config;
mod crawler;
mod data;
mod engine;
mod lcu;
mod overlay;
mod vision;

use config::ConfigManager;
use crawler::{HexdataCrawler, OpggClient};
use data::loader::Database;
use engine::ranked_draft::RankedDraftManager;
use lcu::{LcuClient, LiveClientDataReader};
use overlay::{BenchOverlay, HotkeyManager, InGameOverlay, ItemOverlay, RankedOverlay, TextToSpeech};
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

pub static GLOBAL_TOKIO_HANDLE: OnceLock<tokio::runtime::Handle> = OnceLock::new();

pub fn spawn_task<F>(future: F)
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    if let Some(handle) = GLOBAL_TOKIO_HANDLE.get() {
        handle.spawn(future);
    } else if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.spawn(future);
    } else {
        tracing::error!("Cannot spawn task: Tokio runtime handle unavailable");
    }
}


async fn fetch_slot_counters_and_synergies(
    opgg: &crate::crawler::OpggClient,
    db: &crate::data::loader::Database,
    cid_str: &str,
    pos: &str,
) -> (Vec<crate::overlay::CounterHeroItem>, Vec<crate::overlay::CounterHeroItem>) {
    if cid_str.is_empty() || cid_str == "0" {
        return (Vec::new(), Vec::new());
    }

    let (c_data, syn_data) = tokio::join!(
        opgg.get_counter_picks_against(cid_str, pos, Some(30)),
        opgg.get_champion_synergies(cid_str, pos)
    );

    let mut counters = Vec::new();
    if let Some(arr) = c_data.get("counters").and_then(|v| v.as_array()) {
        for c in arr {
            let opp_cid = c.get("championId").and_then(|v| v.as_str()).unwrap_or("");
            let opp_name = db.resolve_hero_name(opp_cid).unwrap_or_else(|| format!("Hero-{}", opp_cid));
            let c_wr = c.get("counterWinRate").and_then(|v| v.as_f64()).unwrap_or(0.5);
            let games = c.get("games").and_then(|v| v.as_i64()).unwrap_or(100);
            let tier = if c_wr >= 0.53 { 1 } else if c_wr >= 0.51 { 2 } else { 3 };
            counters.push(crate::overlay::CounterHeroItem {
                champion_id: opp_cid.parse().unwrap_or(0),
                name: opp_name,
                tier,
                counter_win_rate: c_wr,
                overall_win_rate: c_wr,
                games,
            });
        }
    }

    let mut synergies = Vec::new();
    if let Some(arr) = syn_data.get("synergies").and_then(|v| v.as_array()) {
        for s in arr {
            let syn_cid = s.get("championId").and_then(|v| v.as_str()).unwrap_or("");
            let syn_name = db.resolve_hero_name(syn_cid).unwrap_or_else(|| format!("Hero-{}", syn_cid));
            let wr = s.get("winRate").and_then(|v| v.as_f64()).unwrap_or(0.5);
            let games = s.get("games").and_then(|v| v.as_i64()).unwrap_or(100);
            let tier = s.get("tier").and_then(|v| v.as_i64()).unwrap_or(2) as i32;
            synergies.push(crate::overlay::CounterHeroItem {
                champion_id: syn_cid.parse().unwrap_or(0),
                name: syn_name,
                tier,
                counter_win_rate: wr,
                overall_win_rate: wr,
                games,
            });
        }
    }

    (counters, synergies)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    // 1. Cache Tokio runtime handle so non-Tokio worker threads (Win32 Hook, GDI loop) can safely spawn async tasks
    let _ = GLOBAL_TOKIO_HANDLE.set(tokio::runtime::Handle::current());

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
    let _config_manager = Arc::new(ConfigManager::new("recommendation_config.json"));
    let lcu_client = Arc::new(LcuClient::new());
    let live_client = Arc::new(LiveClientDataReader::new());
    let ranked_draft = Arc::new(RwLock::new(RankedDraftManager::new()));
    let bench_overlay = Arc::new(BenchOverlay::new());
    let ranked_overlay = Arc::new(RankedOverlay::new());
    let ingame_overlay = Arc::new(InGameOverlay::new());
    let item_overlay = Arc::new(ItemOverlay::new());
    let opgg_client = Arc::new(OpggClient::new());

    // 3. Track current game mode (ARAM vs Summoner's Rift) for hotkeys & overlays
    let is_aram_active = Arc::new(std::sync::atomic::AtomicBool::new(false));

    // 4. Start Overlays
    bench_overlay.start();
    ranked_overlay.start();
    ingame_overlay.start();
    ingame_overlay.start_auto_sensing_loop(
        db.clone(),
        live_client.clone(),
        lcu_client.clone(),
        is_aram_active.clone(),
    );
    item_overlay.start();

    // 5. Start Global Hotkeys
    let ingame_clone = ingame_overlay.clone();
    let item_clone = item_overlay.clone();
    let db_for_hotkey = db.clone();
    let is_aram_for_hotkey = is_aram_active.clone();
    let hotkeys = HotkeyManager::new();
    hotkeys.start(
        move || {
            let is_aram = is_aram_for_hotkey.load(std::sync::atomic::Ordering::SeqCst);
            info!("Hotkey '[' pressed: is_aram = {}", is_aram);
            if is_aram {
                info!("Triggering InGame Augment Scan (ARAM)");
                ingame_clone.trigger_scan_with_db(&db_for_hotkey);
                TextToSpeech::speak("海克斯强化推演已激活");
            } else {
                let live_reader = LiveClientDataReader::new();
                let ingame_fallback = ingame_clone.clone();
                let db_fallback = db_for_hotkey.clone();
                let is_aram_flag = is_aram_for_hotkey.clone();
                crate::spawn_task(async move {
                    let stats = live_reader.get_live_stats().await;
                    if stats.is_in_game && stats.is_aram {
                        is_aram_flag.store(true, std::sync::atomic::Ordering::SeqCst);
                        info!("Fast live probe confirmed ARAM! Triggering scan...");
                        ingame_fallback.trigger_scan_with_db(&db_fallback);
                        TextToSpeech::speak("海克斯强化推演已激活");
                    } else if stats.is_in_game {
                        info!("Hotkey '[' pressed: Summoner's Rift active, skipping Hextech scan");
                        TextToSpeech::speak("当前为召唤师峡谷模式，海克斯强化仅在狂暴大乱斗模式生效");
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
    if let Ok(mut gh) = crate::overlay::win32_hud::GLOBAL_HISTORY.write() {
        *gh = Some(player_history.clone());
    }
    if let Ok(mut gl) = crate::overlay::win32_hud::GLOBAL_LCU.write() {
        *gl = Some(lcu_client.clone());
    }

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

    // Warm up OP.GG tier list in background
    let opgg_warmup = opgg_client.clone();
    tokio::spawn(async move {
        let _ = opgg_warmup.get_ranked_tier_list(false).await;
    });

    tokio::spawn(async move {
        let mut last_level = 0i32;
        let mut last_draft_sig = String::new();
        let mut cached_ranked_data: Option<crate::overlay::RankedOverlayData> = None;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            let phase = lcu_bg.get_gameflow_phase().await;
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
                        }
                    } else if !d.state.my_team.is_empty() {
                        if bench_bg.is_showing.load(std::sync::atomic::Ordering::SeqCst) {
                            bench_bg.stop();
                        }

                        // Compute draft session fingerprint: if unchanged, reuse cached HUD data to avoid spamming OP.GG API
                        let mut current_sig = String::new();
                        for s in &d.state.my_team {
                            current_sig.push_str(&format!("M:{}:{}:{};", s.cell_id, s.champion_id, s.position));
                        }
                        for s in &d.state.their_team {
                            current_sig.push_str(&format!("T:{}:{}:{};", s.cell_id, s.champion_id, s.position));
                        }

                        if current_sig == last_draft_sig {
                            if let Some(ref cached) = cached_ranked_data {
                                ranked_bg.show_ranked_draft(cached.clone());
                                continue;
                            }
                        }

                        // Resolve teammate profiles via LCU chat participants
                        let profiles = history_bg.resolve_teammates(&session, &lcu_bg).await;

                        let local_slot = d.state.my_team.iter().find(|s| s.is_local_player).cloned();
                        let local_pos = local_slot.as_ref().map(|s| s.position.clone()).unwrap_or_default();

                        // 2. 5v5 Ranked / Normal Draft -> RankedOverlay HUD
                        let mut my_slots = Vec::new();
                        for (idx, s) in d.state.my_team.iter().enumerate() {
                            let prof = profiles.get(idx).cloned();
                            let cid_num = s.champion_id.parse::<i32>().unwrap_or(0);
                            let opp_slot = d.state.their_team.iter().find(|t| t.position == s.position);
                            let m_wr = if s.is_local_player && !s.champion_id.is_empty() {
                                if let Some(opp) = opp_slot {
                                    if !opp.champion_id.is_empty() {
                                        opgg_bg.get_matchup_win_rate(&s.champion_id, &opp.champion_id, &s.position).await
                                            .and_then(|v| v.get("winRate").and_then(|w| w.as_f64()))
                                    } else { None }
                                } else { None }
                            } else { None };

                            let (c_heroes, s_heroes) = if s.is_local_player {
                                fetch_slot_counters_and_synergies(&opgg_bg, &db_bg, &s.champion_id, &s.position).await
                            } else {
                                (Vec::new(), Vec::new())
                            };

                            my_slots.push(crate::overlay::RankedSlotView {
                                slot_index: idx,
                                position: s.position.clone(),
                                position_cn: s.position_cn.clone(),
                                champion_name: s.champion_name.clone(),
                                champion_id: cid_num,
                                win_rate: s.win_rate,
                                tier: s.tier,
                                is_local_player: s.is_local_player,
                                is_locked: !s.champion_id.is_empty(),
                                is_manual_pos: false,
                                matchup_win_rate: m_wr,
                                profile: prof,
                                counter_heroes: c_heroes,
                                synergy_heroes: s_heroes,
                            });
                        }

                        let mut their_slots = Vec::new();
                        for (idx, s) in d.state.their_team.iter().enumerate() {
                            let cid_num = s.champion_id.parse::<i32>().unwrap_or(0);
                            let is_opp = !local_pos.is_empty() && s.position == local_pos;
                            let my_opp = d.state.my_team.iter().find(|m| m.position == s.position);
                            let m_wr = if is_opp && !s.champion_id.is_empty() {
                                if let Some(opp) = my_opp {
                                    if !opp.champion_id.is_empty() {
                                        opgg_bg.get_matchup_win_rate(&opp.champion_id, &s.champion_id, &s.position).await
                                            .and_then(|v| v.get("winRate").and_then(|w| w.as_f64()))
                                    } else { None }
                                } else { None }
                            } else { None };

                            let (c_heroes, s_heroes) = if is_opp {
                                fetch_slot_counters_and_synergies(&opgg_bg, &db_bg, &s.champion_id, &s.position).await
                            } else {
                                (Vec::new(), Vec::new())
                            };

                            their_slots.push(crate::overlay::RankedSlotView {
                                slot_index: idx,
                                position: s.position.clone(),
                                position_cn: s.position_cn.clone(),
                                champion_name: s.champion_name.clone(),
                                champion_id: cid_num,
                                win_rate: s.win_rate,
                                tier: s.tier,
                                is_local_player: false,
                                is_locked: !s.champion_id.is_empty(),
                                is_manual_pos: false,
                                matchup_win_rate: m_wr,
                                profile: None,
                                counter_heroes: c_heroes,
                                synergy_heroes: s_heroes,
                            });
                        }

                        let mut rune_guide = None;
                        let mut lane_counters = Vec::new();

                        if let Some(ref local) = local_slot {
                            if !local.champion_id.is_empty() {
                                if let Some(detail) = opgg_bg.get_champion_detail(&local.champion_id, &local.position).await {
                                    if let Some(runes) = detail.get("recommendedRunes") {
                                        if !runes.is_null() {
                                            rune_guide = Some(crate::overlay::RuneGuideView {
                                                champion_name: local.champion_name.clone(),
                                                position_cn: local.position_cn.clone(),
                                                skill_order: "Q > E > W".to_string(),
                                                rune_data: runes.clone(),
                                            });
                                        }
                                    }
                                }

                                let enemy_slot = d.state.their_team.iter().find(|s| s.position == local.position);
                                if let Some(enemy) = enemy_slot {
                                    if !enemy.champion_id.is_empty() {
                                        let c_data = opgg_bg.get_counter_picks_against(&enemy.champion_id, &enemy.position, Some(8)).await;
                                        if let Some(c_arr) = c_data.get("counters").and_then(|v| v.as_array()) {
                                            for c in c_arr {
                                                let opp_cid = c.get("championId").and_then(|v| v.as_str()).unwrap_or("");
                                                let opp_name = db_bg.resolve_hero_name(opp_cid).unwrap_or_else(|| format!("Hero-{}", opp_cid));
                                                let c_wr = c.get("counterWinRate").and_then(|v| v.as_f64()).unwrap_or(0.5);
                                                let games = c.get("games").and_then(|v| v.as_i64()).unwrap_or(100);
                                                let tier = if c_wr >= 0.53 { 1 } else if c_wr >= 0.51 { 2 } else { 3 };
                                                lane_counters.push(crate::overlay::CounterHeroItem {
                                                    champion_id: opp_cid.parse().unwrap_or(0),
                                                    name: opp_name,
                                                    tier,
                                                    counter_win_rate: c_wr,
                                                    overall_win_rate: c_wr,
                                                    games,
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        let rec_text = if let Some(ref local) = local_slot {
                            if !local.champion_id.is_empty() {
                                let enemy_slot = d.state.their_team.iter().find(|s| s.position == local.position);
                                if let Some(enemy) = enemy_slot {
                                    if !enemy.champion_id.is_empty() {
                                        format!(
                                            "对位[{}]: 胜率 {:.1}% · 推荐符文已就绪 · 技能: 闪现 + 点燃",
                                            enemy.champion_name,
                                            local.win_rate * 100.0
                                        )
                                    } else {
                                        format!("已选[{}]: 等待敌方对位锁定 · 符文已就绪", local.champion_name)
                                    }
                                } else {
                                    format!("已选[{}]: 推荐符文已就绪 · 召唤师技能: 闪现 + 点燃", local.champion_name)
                                }
                            } else {
                                "等待选择英雄 · 锁定后将自动推送一键符文与对位建议".to_string()
                            }
                        } else {
                            "排位选人辅助已激活 · 实时分析对位胜率".to_string()
                        };

                        let decision_card = crate::overlay::RankedDecisionCard {
                            mode: "counter".to_string(),
                            my_locked: local_slot.as_ref().map_or(false, |l| !l.champion_id.is_empty()),
                            default_tab: if rune_guide.is_some() { "runes".to_string() } else { "lane_counter".to_string() },
                            rune_guide,
                            lane_counters,
                            multi_counters: Vec::new(),
                            synergy_picks: Vec::new(),
                        };

                        let overlay_data = crate::overlay::RankedOverlayData {
                            my_team: my_slots,
                            their_team: their_slots,
                            decision_card,
                            recommendation: rec_text,
                        };

                        ranked_bg.show_ranked_draft(overlay_data.clone());
                        cached_ranked_data = Some(overlay_data);
                        last_draft_sig = current_sig;
                    }

                }
            } else {
                last_draft_sig.clear();
                cached_ranked_data = None;
                if bench_bg.is_showing.load(std::sync::atomic::Ordering::SeqCst) {
                    bench_bg.stop();
                }
                if ranked_bg.is_showing.load(std::sync::atomic::Ordering::SeqCst) && !ranked_bg.is_in_test_mode() {
                    ranked_bg.stop();
                }

                // In-Game Live 2999 Port Monitoring Workflow (Active whenever in game or phase == InProgress)
                let mut live_stats = live_bg.get_live_stats().await;
                if phase == "InProgress" || live_stats.is_in_game {
                    if live_stats.is_in_game {
                        // 补齐并保证 hero_id 的准确性 (通过英文/中文英雄名在本地字典反查)
                        if live_stats.hero_id.is_empty() && !live_stats.hero_name.is_empty() {
                            if let Some(resolved) = db_bg.resolve_hero_id(&live_stats.hero_name) {
                                live_stats.hero_id = resolved;
                            }
                        }

                        is_aram_bg.store(live_stats.is_aram, std::sync::atomic::Ordering::SeqCst);
                        if live_stats.is_aram {
                            // ARAM only: on-demand fetch hero detail with cookie handshake
                            if !live_stats.hero_id.is_empty() {
                                let crawler = hex_bg.clone();
                                let db_clone = db_bg.clone();
                                let hero_id = live_stats.hero_id.clone();
                                tokio::spawn(async move {
                                    let _ = db_clone.fetch_and_cache_hero_detail(&hero_id, &crawler).await;
                                });
                            }

                            // Level Milestone Perception (ARAM 3/7/11/15 only)
                            let current_lvl = live_stats.level;
                            let is_milestone = if last_level == 0 {
                                matches!(current_lvl, 3 | 4 | 7 | 11 | 15)
                            } else if current_lvl > 0 && current_lvl != last_level {
                                matches!(current_lvl, 3 | 7 | 11 | 15)
                            } else {
                                false
                            };

                            if is_milestone {
                                info!("ARAM level milestone reached: Level {}! Triggering augment readiness", current_lvl);
                                TextToSpeech::speak(&format!("已升至{}级，海克斯强化节点已就绪", current_lvl));
                                let ingame_auto = ingame_bg.clone();
                                let db_auto = db_bg.clone();
                                tokio::spawn(async move {
                                    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                                    ingame_auto.trigger_scan_with_db(&db_auto);
                                });
                            }
                            last_level = current_lvl;

                            // Continuously push real-time gold, owned components, and enemy items to ItemOverlay
                            item_bg.update_recommendations_from_live(
                                &db_bg,
                                &live_stats.hero_id,
                                &live_stats.hero_name,
                                live_stats.gold,
                                &live_stats.current_items,
                                &live_stats.enemy_team,
                            );
                        } else {
                            // Summoner's Rift: Strictly NO hextech augment milestones or speech!
                            last_level = live_stats.level;
                            item_bg.update_recommendations_from_live(
                                &db_bg,
                                &live_stats.hero_id,
                                &live_stats.hero_name,
                                live_stats.gold,
                                &live_stats.current_items,
                                &live_stats.enemy_team,
                            );
                        }
                    }
                } else if last_level > 0 {
                    // Match has terminated
                    last_level = 0;
                    is_aram_bg.store(false, std::sync::atomic::Ordering::SeqCst);
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
        assert!((s1.rate - 54.2).abs() < 1e-4);
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
        let updated = mgr.update(None, Some(f64::NAN), Some(f64::NAN), Some(f64::INFINITY), None, None, None).await;
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
        let matchup = opgg.get_matchup_win_rate("1", "86", "MID").await;
        assert!(matchup.is_some());
        let val = matchup.unwrap();
        assert!(val.get("winRate").is_some());
        assert!(val.get("label").is_some());
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
}

