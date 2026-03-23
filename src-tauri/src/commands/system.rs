use serde::{Serialize, Deserialize};
use std::process::Command;
use std::net::TcpStream;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemStatus {
    pub llamacpp_installed: bool,
    pub llamacpp_running: bool,
    pub database_ready: bool,
    pub path_to_llamacpp: Option<String>,
}

#[tauri::command]
pub async fn get_system_status() -> SystemStatus {
    // 1. Check if llama-server is in PATH
    let (installed, path) = match Command::new("which")
        .arg("llama-server")
        .output() {
            Ok(output) if output.status.success() => {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                (true, Some(path_str))
            },
            _ => (false, None),
        };

    // 2. Check if a server is already listening on 8080
    let running = TcpStream::connect_timeout(
        &"127.0.0.1:8080".parse().unwrap(), 
        Duration::from_millis(100)
    ).is_ok();

    SystemStatus {
        llamacpp_installed: installed,
        llamacpp_running: running,
        database_ready: true, // Placeholder for SQLite check
        path_to_llamacpp: path,
    }
}
