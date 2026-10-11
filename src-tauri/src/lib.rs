#![recursion_limit = "512"]

pub mod cmd;

use std::sync::Arc;
use lol_hex_rust::config::ConfigManager;
use lol_hex_rust::data::loader::Database;
use lol_hex_rust::engine::PlayerHistoryManager;
use lol_hex_rust::lcu::LcuClient;

pub struct TauriAppState {
    pub lcu: Arc<LcuClient>,
    pub history: Arc<PlayerHistoryManager>,
    pub db: Arc<Database>,
    pub config: Arc<ConfigManager>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = tracing_subscriber::fmt::try_init();

    // 1. Initialize DB and Subsystems
    let mut initial_db = Database::load_from_dir("data");
    if initial_db.heroes_summary.is_empty() {
        initial_db = Database::load_from_dir("../data");
    }
    let db = Arc::new(initial_db);
    let lcu = Arc::new(LcuClient::new());
    let history = Arc::new(PlayerHistoryManager::new());
    let config = Arc::new(ConfigManager::new("recommendation_config.json"));

    let history_db = history.clone();
    let db_clone = db.clone();
    tauri::async_runtime::spawn(async move {
        history_db.set_database(db_clone).await;
    });

    // 2. Background ReadyCheck Auto-Accept Monitor
    let lcu_bg = lcu.clone();
    let config_bg = config.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            let phase = lcu_bg.get_gameflow_phase().await;
            if phase == "ReadyCheck" {
                let cfg = config_bg.get().await;
                if cfg.auto_accept_match {
                    if let Some(rc) = lcu_bg.get_ready_check().await {
                        let player_resp = rc.get("playerResponse").and_then(|v| v.as_str()).unwrap_or("");
                        let state = rc.get("state").and_then(|v| v.as_str()).unwrap_or("");
                        if player_resp != "Accepted" && player_resp != "Declined" && state == "InProgress" {
                            let delay = cfg.auto_accept_delay_seconds;
                            if delay > 0 {
                                tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
                            }
                            if lcu_bg.get_gameflow_phase().await == "ReadyCheck" {
                                let ok = lcu_bg.accept_ready_check().await;
                                if ok {
                                    tracing::info!("⚡ [HexBrain Tauri] 自动接受对局成功 (延迟 {}s)", delay);
                                }
                            }
                        }
                    }
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
            cmd::get_match_detail,
            cmd::get_automation_config,
            cmd::update_automation_config,
            cmd::apply_runes_and_spells
        ])
        .run(tauri::generate_context!())
        .expect("error while running hexbrain tauri application");
}
