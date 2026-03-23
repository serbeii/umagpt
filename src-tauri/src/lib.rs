pub mod commands;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            commands::system::get_system_status,
            commands::chat::chat_stream,
            commands::chat::save_conversations,
            commands::chat::load_conversations,
            commands::collection::get_collection,
            commands::collection::upsert_trainee,
            commands::collection::upsert_card,
            commands::db::init_db,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
