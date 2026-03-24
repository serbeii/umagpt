use serde::{Serialize, Deserialize};
use std::net::TcpStream;
use std::time::Duration;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandChild;
use std::sync::Mutex;

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemStatus {
    pub llamacpp_installed: bool,
    pub llamacpp_running: bool,
    pub database_ready: bool,
    pub path_to_llamacpp: Option<String>,
}

pub struct LlamaProcess(pub Mutex<Option<CommandChild>>);

#[tauri::command]
pub async fn get_system_status() -> SystemStatus {
    // 1. Check if llama-server sidecar is present
    // For now we'll just check if a server is already listening on 8080
    let running = TcpStream::connect_timeout(
        &"127.0.0.1:8080".parse().unwrap(), 
        Duration::from_millis(100)
    ).is_ok();

    SystemStatus {
        llamacpp_installed: true, // We assume sidecar exists if configured
        llamacpp_running: running,
        database_ready: true,
        path_to_llamacpp: Some("Sidecar: llama-server".to_string()),
    }
}

#[tauri::command]
pub async fn start_llamacpp(app: AppHandle, state: State<'_, LlamaProcess>) -> Result<(), String> {
    let mut lock = state.0.lock().unwrap();
    if lock.is_some() {
        return Ok(()); // Already started
    }

    // Get model path from app data dir
    let mut model_path = app.path().app_data_dir().unwrap();
    model_path.push("models/chat.gguf");

    // Ensure models dir exists
    let mut models_dir = app.path().app_data_dir().unwrap();
    models_dir.push("models");
    if !models_dir.exists() {
        std::fs::create_dir_all(&models_dir).map_err(|e| e.to_string())?;
    }

    let sidecar = app.shell().sidecar("llama-server")
        .map_err(|e| e.to_string())?
        .args(["--port", "8080", "--model", model_path.to_str().unwrap()]);

    let (mut _rx, child) = sidecar.spawn().map_err(|e| e.to_string())?;
    
    *lock = Some(child);
    Ok(())
}

#[tauri::command]
pub async fn stop_llamacpp(state: State<'_, LlamaProcess>) -> Result<(), String> {
    let mut lock = state.0.lock().unwrap();
    if let Some(child) = lock.take() {
        child.kill().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn restart_llamacpp(app: AppHandle, state: State<'_, LlamaProcess>) -> Result<(), String> {
    stop_llamacpp(state.clone()).await?;
    start_llamacpp(app, state).await?;
    Ok(())
}
