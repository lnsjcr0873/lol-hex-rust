mod config;
mod crawler;
mod data;
mod engine;
mod lcu;
mod overlay;
mod server;
mod vision;

use config::ConfigManager;
use crawler::OpggClient;
use data::loader::Database;
use engine::ranked_draft::RankedDraftManager;
use lcu::{LcuClient, LiveClientDataReader};
use overlay::{BenchOverlay, HotkeyManager, InGameOverlay, ItemOverlay, RankedOverlay, TextToSpeech};
use server::{create_router, AppState, WsBroadcaster};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    println!("============================================================");
    println!("⚡ HexBrain - 英雄联盟海克斯大乱斗决策系统 (Rust 原生版) ⚡");
    println!("基于 1157万+ 局真实对局大数据的海克斯大乱斗实时决策辅助引擎");
    println!("============================================================");

    // 1. Load In-Memory Database
    let db = Arc::new(Database::load_from_dir("data"));

    // 2. Initialize Subsystems
    let config_manager = Arc::new(ConfigManager::new("server/config/recommendation_config.json"));
    let lcu_client = Arc::new(LcuClient::new());
    let live_client = Arc::new(LiveClientDataReader::new());
    let ranked_draft = Arc::new(RwLock::new(RankedDraftManager::new()));
    let bench_overlay = Arc::new(BenchOverlay::new());
    let ranked_overlay = Arc::new(RankedOverlay::new());
    let ingame_overlay = Arc::new(InGameOverlay::new());
    let item_overlay = Arc::new(ItemOverlay::new());
    let opgg_client = Arc::new(OpggClient::new());
    let ws_broadcaster = Arc::new(WsBroadcaster::new());

    // 3. Start Overlays
    bench_overlay.start();
    ranked_overlay.start();
    ingame_overlay.start();
    item_overlay.start();

    // 4. Start Global Hotkeys
    let ingame_clone = ingame_overlay.clone();
    let item_clone = item_overlay.clone();
    let hotkeys = HotkeyManager::new();
    hotkeys.start(
        move || {
            info!("Hotkey '[' pressed: Triggering InGame Augment Scan");
            ingame_clone.trigger_scan();
            TextToSpeech::speak("海克斯强化推演已激活");
        },
        move || {
            info!("Hotkey ']' pressed: Triggering Item Recommendation HUD");
            item_clone.trigger_manual_scan();
        },
    );

    // 5. Background LCU & Game Polling Task
    let lcu_bg = lcu_client.clone();
    let draft_bg = ranked_draft.clone();
    let ws_bg = ws_broadcaster.clone();

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            let phase = lcu_bg.get_gameflow_phase().await;
            if phase == "ChampSelect" {
                let session = lcu_bg.get_champ_select_session().await;
                if !session.is_null() {
                    let mut d = draft_bg.write().await;
                    d.parse_session(&session);

                    ws_bg.broadcast(&serde_json::json!({
                        "event": "LIVE_SYNC",
                        "lcuConnected": true,
                        "gamePhase": "ChampSelect",
                        "session": session
                    }));
                }
            }
        }
    });

    // 6. Start Axum Web Server
    let state = AppState {
        db,
        config_manager,
        lcu_client,
        live_client,
        ranked_draft,
        bench_overlay,
        ranked_overlay,
        ingame_overlay,
        item_overlay,
        opgg_client,
        ws_broadcaster,
    };

    let app = create_router(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("🚀 HexBrain 服务已在 http://{} 启动！", addr);
    println!("📖 请在浏览器访问 http://127.0.0.1:8000 打开电竞看板");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

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
            assert!(db.heroes_detail.contains_key(&id), "Hero detail for Annie should be indexed by canonical id");
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
}
