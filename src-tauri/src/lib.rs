#![recursion_limit = "512"]

pub mod cmd;

use std::sync::Arc;
use lol_hex_rust::config::ConfigManager;
use lol_hex_rust::data::loader::Database;
use lol_hex_rust::engine::PlayerHistoryManager;
use lol_hex_rust::lcu::LcuClient;
use lol_hex_rust::runtime::{start_background_engine, BackgroundEngineOptions};

use tauri::Manager;

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

    // Initialize unified HexBrain background engine (LCU, Overlays, Hotkeys, Gameflow Monitor)
    let engine = tauri::async_runtime::block_on(async {
        start_background_engine(BackgroundEngineOptions {
            enable_hotkeys: true,
            ws_broadcaster: None,
        }).await
    });

    tauri::Builder::default()
        .manage(TauriAppState {
            lcu: engine.lcu.clone(),
            history: engine.history.clone(),
            db: engine.db.clone(),
            config: engine.config.clone(),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            cmd::get_ongoing_match,
            cmd::send_chat_records,
            cmd::get_lcu_status,
            cmd::get_summoner_profile,
            cmd::get_match_history,
            cmd::get_raw_match_history,
            cmd::get_champion_mastery,
            cmd::get_player_challenges,
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
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    if window.label() == "ongoing-overlay" {
                        api.prevent_close();
                        let _ = window.hide();
                    } else if window.label() == "main" {
                        window.app_handle().exit(0);
                    }
                }
                tauri::WindowEvent::Destroyed => {
                    if window.label() == "main" {
                        window.app_handle().exit(0);
                    }
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running hexbrain tauri application");
}
