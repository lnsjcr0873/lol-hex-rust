#![recursion_limit = "512"]

mod config;
mod crawler;
mod data;
mod engine;
mod lcu;
mod overlay;
mod runtime;
mod sgp;
mod storage;
mod vision;
mod web;

use std::sync::Arc;
use tracing::Level;
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

    // 1. Initialize and start unified HexBrain background engine
    let ws_broadcaster = Arc::new(crate::web::WsBroadcaster::new());
    let engine = crate::runtime::start_background_engine(crate::runtime::BackgroundEngineOptions {
        enable_hotkeys: true,
        ws_broadcaster: Some(ws_broadcaster.clone()),
    }).await;

    // 2. Initialize and Launch Embedded Web UI (League-Akari Style Realtime Match + History + Settings)
    let web_state = crate::web::AppState {
        db: engine.db.clone(),
        lcu: engine.lcu.clone(),
        history: engine.history.clone(),
        config: engine.config.clone(),
        item_overlay: engine.item_overlay.clone(),
        ingame_overlay: engine.ingame_overlay.clone(),
        bench_overlay: engine.bench_overlay.clone(),
        broadcaster: ws_broadcaster.clone(),
    };
    tokio::spawn(async move {
        if let Err(e) = crate::web::start_web_server(web_state, 8765).await {
            tracing::error!("Embedded Web Server failed: {}", e);
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
    use crate::data::Database;
    use crate::overlay::InGameOverlay;
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

