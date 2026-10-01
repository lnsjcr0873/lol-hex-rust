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
    }
}
