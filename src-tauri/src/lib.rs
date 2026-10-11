mod cmd;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = tracing_subscriber::fmt::try_init();

    tauri::Builder::default()
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
