use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};
use crate::llm_provider::ProviderManager;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub scenario: String,
    pub messages: Vec<Message>,
    pub created_at: String,
    pub updated_at: String,
}

fn get_storage_path(app: &AppHandle) -> PathBuf {
    let mut path = app.path().app_data_dir().expect("failed to get app data dir");
    if !path.exists() {
        fs::create_dir_all(&path).expect("failed to create app data dir");
    }
    path.push("history.json");
    path
}

#[tauri::command]
pub async fn save_conversations(app: AppHandle, conversations: Vec<Conversation>) -> Result<(), String> {
    let path = get_storage_path(&app);
    let json = serde_json::to_string_pretty(&conversations).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn load_conversations(app: AppHandle) -> Result<Vec<Conversation>, String> {
    let path = get_storage_path(&app);
    if !path.exists() {
        return Ok(vec![]);
    }
    let json = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let conversations = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(conversations)
}

#[tauri::command]
pub async fn chat_stream(
    app: AppHandle,
    state: State<'_, ProviderManager>,
    messages: Vec<Message>,
) -> Result<(), String> {
    let handler = {
        let active_lock = state.active_provider.read().await;
        let active = active_lock.as_ref().ok_or("No active provider")?;
        active.completion_handler()
    };

    handler.chat_stream(&app, messages).await
}
