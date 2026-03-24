use std::sync::Mutex;
use tauri::Manager;
pub mod commands;
pub mod llm_provider;
pub mod models;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .setup(|app| {
            use tauri_plugin_store::StoreExt;
            let app_handle = app.handle().clone();
            
            // Sync models
            let model_manager = models::ModelManager::new(&app_handle);
            let _ = model_manager.sync_registry();
            
            let provider_manager = app_handle.state::<llm_provider::ProviderManager>();
            provider_manager.setup_restart_handler(&app_handle);

            // Load settings and initialize provider
            let app_handle_for_setup = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(store) = app_handle_for_setup.store("settings.json") {
                    if let Some(config_val) = store.get("active_provider") {
                        if let Ok(config) = serde_json::from_value::<llm_provider::ProviderConfig>(config_val) {
                            let provider_manager = app_handle_for_setup.state::<llm_provider::ProviderManager>();
                            let _ = provider_manager.switch_provider(config, &app_handle_for_setup).await;
                        }
                    }
                }
            });

            Ok(())
        })
        .manage(llm_provider::ProviderManager::new())
        .manage(commands::system::LlamaProcess(Mutex::new(None)))
        .invoke_handler(tauri::generate_handler![
            commands::system::get_system_status,
            commands::system::start_llamacpp,
            commands::system::stop_llamacpp,
            commands::system::restart_llamacpp,
            commands::chat::chat_stream,
            commands::chat::save_conversations,
            commands::chat::load_conversations,
            commands::collection::get_collection,
            commands::collection::clear_collection,
            commands::collection::upsert_trainee,
            commands::collection::upsert_card,
            commands::db::init_db,
            commands::llm::get_provider_state,
            commands::llm::set_provider,
            commands::llm::start_provider,
            commands::llm::stop_provider,
            commands::llm::resolve_llama_binary,
            commands::models::list_models,
            commands::models::sync_models,
            commands::models::delete_model,
            commands::models::import_model,
            commands::models::search_hf_models,
            commands::models::download_model,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| match event {
            tauri::RunEvent::ExitRequested { .. } => {
                let provider_manager = app_handle.state::<llm_provider::ProviderManager>();
                let app_handle_clone = app_handle.clone();
                // Use block_on for cleanup
                tauri::async_runtime::block_on(async move {
                    let _ = provider_manager.stop_active(&app_handle_clone).await;
                });
            }
            _ => {}
        });
}
