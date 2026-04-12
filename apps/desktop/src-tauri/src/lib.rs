mod commands;
mod error;
mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::create_campaign,
            commands::open_campaign,
            commands::list_characters,
            commands::create_character,
            commands::get_character,
            commands::apply_event,
            commands::get_ledger,
            commands::get_racial_limits,
            commands::validate_draft,
            commands::save_character_base,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
