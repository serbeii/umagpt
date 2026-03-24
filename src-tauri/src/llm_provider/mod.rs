pub mod llama_server;
pub mod remote;

use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tauri::{AppHandle, Manager, Emitter, Listener};
use async_trait::async_trait;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProviderState {
    Unconfigured,
    Starting,
    Healthy,
    Degraded,
    Crashed,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProviderConfig {
    LlamaServer {
        binary_path: Option<String>,
        model_path: String,
        port: u16,
        context_length: u32,
        extra_args: Vec<String>,
    },
    OpenAI {
        api_key: String,
        base_url: Option<String>,
        model_id: String,
    },
    Anthropic {
        api_key: String,
        model_id: String,
    },
    Gemini {
        api_key: String,
        model_id: String,
    },
    Groq {
        api_key: String,
        model_id: String,
    },
    DeepSeek {
        api_key: String,
        model_id: String,
    },
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn start(&self, app: &AppHandle) -> Result<(), String>;
    async fn stop(&self) -> Result<(), String>;
    async fn health(&self) -> ProviderState;
    fn base_url(&self) -> String;
    fn completion_handler(&self) -> Box<dyn Completion>;
}

#[async_trait]
pub trait Completion: Send + Sync {
    async fn chat_stream(
        &self,
        app: &AppHandle,
        messages: Vec<crate::commands::chat::Message>,
    ) -> Result<(), String>;
}

pub struct ProviderManager {
    pub state: Arc<RwLock<ProviderState>>,
    pub config: Arc<RwLock<Option<ProviderConfig>>>,
    pub active_provider: Arc<RwLock<Option<Box<dyn LlmProvider>>>>,
}

impl ProviderManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(ProviderState::Unconfigured)),
            config: Arc::new(RwLock::new(None)),
            active_provider: Arc::new(RwLock::new(None)),
        }
    }

    pub fn setup_restart_handler(&self, app: &AppHandle) {
        let app_handle = app.clone();
        app.listen("provider_restart_requested", move |_| {
            let app_clone = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                let manager = app_clone.state::<ProviderManager>();
                let _ = manager.start_active(&app_clone).await;
            });
        });
    }

    pub async fn get_state(&self) -> ProviderState {
        self.state.read().await.clone()
    }

    pub async fn set_config(&self, config: ProviderConfig) {
        *self.config.write().await = Some(config);
    }

    pub async fn update_state(&self, new_state: ProviderState, app: &AppHandle) {
        let mut state = self.state.write().await;
        *state = new_state.clone();
        let _ = app.emit("provider_state_changed", new_state);
    }

    pub async fn switch_provider(&self, config: ProviderConfig, app: &AppHandle) -> Result<(), String> {
        // Stop current provider
        if let Some(provider) = self.active_provider.write().await.take() {
            provider.stop().await?;
        }
        self.update_state(ProviderState::Stopped, app).await;

        let provider: Box<dyn LlmProvider> = match config.clone() {
            ProviderConfig::LlamaServer { binary_path, model_path, port, context_length, extra_args } => {
                Box::new(llama_server::LlamaServerProvider::new(binary_path, model_path, port, context_length, extra_args))
            }
            ProviderConfig::OpenAI { api_key, base_url, model_id } => {
                Box::new(remote::OpenAIProvider::new(api_key, base_url, model_id))
            }
            ProviderConfig::Anthropic { api_key, model_id } => {
                Box::new(remote::AnthropicProvider::new(api_key, model_id))
            }
            ProviderConfig::Gemini { api_key, model_id } => {
                Box::new(remote::GeminiProvider::new(api_key, model_id))
            }
            ProviderConfig::Groq { api_key, model_id } => {
                Box::new(remote::OpenAIProvider::new_groq(api_key, model_id))
            }
            ProviderConfig::DeepSeek { api_key, model_id } => {
                Box::new(remote::OpenAIProvider::new_deepseek(api_key, model_id))
            }
        };

        self.set_config(config.clone()).await;
        *self.active_provider.write().await = Some(provider);
        
        // For remote providers, they don't need "starting", they are healthy if config is OK
        match config {
            ProviderConfig::LlamaServer { .. } => {
                self.update_state(ProviderState::Stopped, app).await;
            },
            _ => {
                self.update_state(ProviderState::Healthy, app).await;
            }
        }

        // Persist to store
        use tauri_plugin_store::StoreExt;
        if let Ok(store) = app.store("settings.json") {
            store.set("active_provider", serde_json::to_value(config).unwrap());
            let _ = store.save();
        }

        Ok(())
    }

    pub async fn start_active(&self, app: &AppHandle) -> Result<(), String> {
        let provider_opt = self.active_provider.read().await;
        if let Some(provider) = provider_opt.as_ref() {
            self.update_state(ProviderState::Starting, app).await;
            match provider.start(app).await {
                Ok(_) => {
                    self.update_state(ProviderState::Healthy, app).await;
                    Ok(())
                },
                Err(e) => {
                    self.update_state(ProviderState::Crashed, app).await;
                    Err(e)
                }
            }
        } else {
            return Err("No active provider".to_string());
        }
    }

    pub async fn stop_active(&self, app: &AppHandle) -> Result<(), String> {
        if let Some(provider) = self.active_provider.read().await.as_ref() {
            provider.stop().await?;
            self.update_state(ProviderState::Stopped, app).await;
        }
        Ok(())
    }
}

