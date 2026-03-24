use tauri::{AppHandle, State};
use crate::llm_provider::{ProviderManager, ProviderConfig, ProviderState, llama_server};

#[tauri::command]
pub async fn get_provider_state(state: State<'_, ProviderManager>) -> Result<ProviderState, String> {
    Ok(state.get_state().await)
}

#[tauri::command]
pub async fn set_provider(
    app: AppHandle,
    state: State<'_, ProviderManager>,
    config: ProviderConfig,
) -> Result<(), String> {
    state.switch_provider(config, &app).await
}

#[tauri::command]
pub async fn start_provider(
    app: AppHandle,
    state: State<'_, ProviderManager>,
) -> Result<(), String> {
    state.start_active(&app).await
}

#[tauri::command]
pub async fn stop_provider(
    app: AppHandle,
    state: State<'_, ProviderManager>,
) -> Result<(), String> {
    state.stop_active(&app).await
}

#[tauri::command]
pub async fn resolve_llama_binary(user_path: Option<String>) -> Result<Option<String>, String> {
    Ok(llama_server::LlamaServerProvider::resolve_binary(user_path.as_deref())
        .map(|p| p.to_string_lossy().to_string()))
}

#[tauri::command]
pub async fn get_logs(state: tauri::State<'_, ProviderManager>) -> Result<Vec<String>, String> {
    // This is optional if we only rely on events, but useful for initial load
    Ok(vec![])
}
