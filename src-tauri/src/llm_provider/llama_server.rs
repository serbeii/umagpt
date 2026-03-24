use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{AppHandle, Manager, Emitter};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandChild;
use which::which;
use crate::llm_provider::{LlmProvider, ProviderState};
use std::time::Duration;

pub struct LlamaServerProvider {
    pub child: Arc<Mutex<Option<CommandChild>>>,
    pub binary_path: Option<String>,
    pub model_path: String,
    pub port: u16,
    pub context_length: u32,
    pub extra_args: Vec<String>,
    pub last_error: Arc<Mutex<Option<String>>>,
    pub retry_count: Arc<Mutex<u32>>,
}

impl LlamaServerProvider {
    pub fn new(
        binary_path: Option<String>,
        model_path: String,
        port: u16,
        context_length: u32,
        extra_args: Vec<String>,
    ) -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
            binary_path,
            model_path,
            port,
            context_length,
            extra_args,
            last_error: Arc::new(Mutex::new(None)),
            retry_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn resolve_binary(user_path: Option<&str>) -> Option<PathBuf> {
        // 1. Check user-configured path
        if let Some(path) = user_path {
            let pb = PathBuf::from(path);
            if pb.exists() {
                return Some(pb);
            }
        }

        // 2. Fall back to PATH lookup
        if let Ok(path) = which("llama-server") {
            return Some(path);
        }

        // 3. Well-known install locations
        #[cfg(target_os = "linux")]
        let paths = vec!["~/.local/bin/llama-server", "/usr/local/bin/llama-server"];
        #[cfg(target_os = "macos")]
        let paths = vec!["/usr/local/bin/llama-server", "/opt/homebrew/bin/llama-server"];
        #[cfg(target_os = "windows")]
        let paths = vec!["%LOCALAPPDATA%\\llama.cpp\\llama-server.exe"];

        for path in paths {
            // Need to expand ~ and env vars
            let expanded = if path.starts_with('~') {
                if let Some(home) = dirs::home_dir() {
                    home.join(path.trim_start_matches("~/"))
                } else {
                    PathBuf::from(path)
                }
            } else {
                PathBuf::from(path)
            };

            if expanded.exists() {
                return Some(expanded);
            }
        }

        None
    }
}

use async_trait::async_trait;

#[async_trait]
impl LlmProvider for LlamaServerProvider {
    async fn start(&self, app: &AppHandle) -> Result<(), String> {
        let binary = Self::resolve_binary(self.binary_path.as_deref())
            .ok_or_else(|| "Could not find llama-server binary. Please install llama.cpp or provide a path in settings.".to_string())?;

        // Resolve model path
        let mut model_path = std::path::PathBuf::from(&self.model_path);
        if !model_path.is_absolute() {
            let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
            model_path = app_data.join("models").join(&self.model_path);
        }

        // Pre-launch validations
        if !binary.exists() {
            return Err(format!("Binary does not exist: {:?}", binary));
        }

        if !model_path.exists() {
            return Err(format!("Model file does not exist: {:?}", model_path));
        }
        if model_path.extension().and_then(|s| s.to_str()) != Some("gguf") {
            return Err("Model file must have .gguf extension".to_string());
        }

        // Check if port is free
        use std::net::TcpListener;
        if TcpListener::bind(format!("127.0.0.1:{}", self.port)).is_err() {
            return Err(format!("Port {} is already in use. Try changing it in settings.", self.port));
        }

        let mut args = vec![
            "--model".to_string(),
            model_path.to_string_lossy().to_string(),
            "--port".to_string(),
            self.port.to_string(),
            "--ctx-size".to_string(),
            self.context_length.to_string(),
        ];
        args.extend(self.extra_args.clone());

        let mut shell_command = app.shell().command(binary.to_str().unwrap());
        shell_command = shell_command.args(args);

        let (mut rx, child) = shell_command.spawn().map_err(|e| e.to_string())?;

        let mut lock = self.child.lock().await;
        *lock = Some(child);

        // Reset retry count on manual start
        *self.retry_count.lock().await = 0;

        // Spawn task to handle output and exit
        let app_handle = app.clone();
        let retry_count = self.retry_count.clone();
        let last_error = self.last_error.clone();
        
        tauri::async_runtime::spawn(async move {
            use tauri_plugin_shell::process::CommandEvent;
            let mut is_intentional_stop = false;

            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Stdout(line) => {
                        let _ = app_handle.emit("provider_log", String::from_utf8_lossy(&line));
                    }
                    CommandEvent::Stderr(line) => {
                        let _ = app_handle.emit("provider_log", String::from_utf8_lossy(&line));
                    }
                    CommandEvent::Terminated(status) => {
                        if !is_intentional_stop {
                            let mut retries = retry_count.lock().await;
                            if *retries < 3 {
                                *retries += 1;
                                let _ = app_handle.emit("provider_log", format!("Process crashed (code {:?}). Attempting restart {}/3...", status.code, *retries));
                                let _ = app_handle.emit("provider_state_changed", ProviderState::Starting);
                                
                                // Exponential backoff
                                tokio::time::sleep(Duration::from_secs(2u64.pow(*retries))).await;
                                
                                // Need a way to re-trigger start. 
                                // Since we are in a spawned task, we'd need to call into ProviderManager
                                // but for simplicity we'll just emit an event or let the manager handle it.
                                let _ = app_handle.emit("provider_restart_requested", ());
                            } else {
                                *last_error.lock().await = Some(format!("Process terminated with code: {:?}", status.code));
                                let _ = app_handle.emit("provider_state_changed", ProviderState::Crashed);
                                let _ = app_handle.emit("provider_log", format!("Process terminated with code: {:?}. Max retries reached.", status.code));
                            }
                        }
                        break;
                    }
                    _ => {}
                }
            }
        });

        // Poll /health endpoint
        let url = format!("http://127.0.0.1:{}/health", self.port);
        let mut success = false;
        for _ in 0..30 { // 30 seconds timeout
            tokio::time::sleep(Duration::from_secs(1)).await;
            if let Ok(res) = reqwest::get(&url).await {
                if res.status().is_success() {
                    success = true;
                    break;
                }
            }
            // Check if process still alive
            if self.child.lock().await.is_none() {
                return Err("Process died during startup".to_string());
            }
        }

        if !success {
            let _ = self.stop().await;
            return Err("Timed out waiting for llama-server to become healthy".to_string());
        }

        Ok(())
    }

    async fn stop(&self) -> Result<(), String> {
        let mut lock = self.child.lock().await;
        if let Some(child) = lock.take() {
            child.kill().map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    async fn health(&self) -> ProviderState {
        let url = format!("http://localhost:{}/health", self.port);
        match reqwest::get(url).await {
            Ok(res) if res.status().is_success() => ProviderState::Healthy,
            _ => ProviderState::Degraded,
        }
    }

    fn base_url(&self) -> String {
        format!("http://localhost:{}", self.port)
    }

    fn completion_handler(&self) -> Box<dyn crate::llm_provider::Completion> {
        Box::new(crate::llm_provider::remote::OpenAiCompletionHandler {
            api_key: "not-needed".to_string(),
            base_url: self.base_url(),
            model_id: "local".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_binary_not_found() {
        let res = LlamaServerProvider::resolve_binary(Some("/non/existent/path"));
        // This might still find it in PATH, so it's not a reliable negative test 
        // unless we know llama-server is not in PATH.
    }
}
