use tauri::{AppHandle, Emitter};
use crate::llm_provider::{LlmProvider, ProviderState, Completion};
use async_trait::async_trait;
use serde_json::json;
use futures_util::StreamExt;

pub struct OpenAiCompletionHandler {
    pub api_key: String,
    pub base_url: String,
    pub model_id: String,
}

#[async_trait]
impl Completion for OpenAiCompletionHandler {
    async fn chat_stream(
        &self,
        app: &AppHandle,
        messages: Vec<crate::commands::chat::Message>,
    ) -> Result<(), String> {
        let client = reqwest::Client::new();
        let url = if self.base_url.ends_with("/v1") {
            format!("{}/chat/completions", self.base_url)
        } else {
            format!("{}/v1/chat/completions", self.base_url)
        };

        let body = json!({
            "model": self.model_id,
            "messages": messages,
            "stream": true,
        });

        let mut request = client.post(url).json(&body);
        if !self.api_key.is_empty() && self.api_key != "not-needed" {
            request = request.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let res = request.send().await.map_err(|e| e.to_string())?;
        let mut stream = res.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| e.to_string())?;
            let text = String::from_utf8_lossy(&chunk);
            
            for line in text.lines() {
                if line.starts_with("data: ") {
                    let data = line.trim_start_matches("data: ");
                    if data == "[DONE]" { break; }
                    
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                            let _ = app.emit("chat_delta", content);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub struct OpenAIProvider {
    pub api_key: String,
    pub base_url: Option<String>,
    pub model_id: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String, base_url: Option<String>, model_id: String) -> Self {
        Self { api_key, base_url, model_id }
    }
    pub fn new_groq(api_key: String, model_id: String) -> Self {
        Self { api_key, base_url: Some("https://api.groq.com/openai/v1".to_string()), model_id }
    }
    pub fn new_deepseek(api_key: String, model_id: String) -> Self {
        Self { api_key, base_url: Some("https://api.deepseek.com".to_string()), model_id }
    }
}

#[async_trait]
impl LlmProvider for OpenAIProvider {
    async fn start(&self, _app: &AppHandle) -> Result<(), String> {
        if self.api_key.is_empty() { return Err("API key is required".to_string()); }
        Ok(())
    }
    async fn stop(&self) -> Result<(), String> { Ok(()) }
    async fn health(&self) -> ProviderState {
        if self.api_key.is_empty() { return ProviderState::Unconfigured; }
        ProviderState::Healthy
    }
    fn base_url(&self) -> String {
        self.base_url.clone().unwrap_or_else(|| "https://api.openai.com/v1".to_string())
    }
    fn completion_handler(&self) -> Box<dyn Completion> {
        Box::new(OpenAiCompletionHandler {
            api_key: self.api_key.clone(),
            base_url: self.base_url(),
            model_id: self.model_id.clone(),
        })
    }
}

pub struct AnthropicProvider {
    pub api_key: String,
    pub model_id: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model_id: String) -> Self {
        Self { api_key, model_id }
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn start(&self, _app: &AppHandle) -> Result<(), String> {
        if self.api_key.is_empty() { return Err("API key is required".to_string()); }
        Ok(())
    }
    async fn stop(&self) -> Result<(), String> { Ok(()) }
    async fn health(&self) -> ProviderState {
        if self.api_key.is_empty() { return ProviderState::Unconfigured; }
        ProviderState::Healthy
    }
    fn base_url(&self) -> String { "https://api.anthropic.com/v1".to_string() }
    fn completion_handler(&self) -> Box<dyn Completion> {
        Box::new(AnthropicCompletionHandler {
            api_key: self.api_key.clone(),
            model_id: self.model_id.clone(),
        })
    }
}

pub struct AnthropicCompletionHandler {
    pub api_key: String,
    pub model_id: String,
}

#[async_trait]
impl Completion for AnthropicCompletionHandler {
    async fn chat_stream(
        &self,
        app: &AppHandle,
        messages: Vec<crate::commands::chat::Message>,
    ) -> Result<(), String> {
        let client = reqwest::Client::new();
        let body = json!({
            "model": self.model_id,
            "messages": messages,
            "max_tokens": 4096,
            "stream": true,
        });

        let res = client.post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let mut stream = res.bytes_stream();
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| e.to_string())?;
            let text = String::from_utf8_lossy(&chunk);
            for line in text.lines() {
                if line.starts_with("data: ") {
                    let data = line.trim_start_matches("data: ");
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(content) = json["delta"]["text"].as_str() {
                            let _ = app.emit("chat_delta", content);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

pub struct GeminiProvider {
    pub api_key: String,
    pub model_id: String,
}

impl GeminiProvider {
    pub fn new(api_key: String, model_id: String) -> Self {
        Self { api_key, model_id }
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn start(&self, _app: &AppHandle) -> Result<(), String> {
        if self.api_key.is_empty() { return Err("API key is required".to_string()); }
        Ok(())
    }
    async fn stop(&self) -> Result<(), String> { Ok(()) }
    async fn health(&self) -> ProviderState {
        if self.api_key.is_empty() { return ProviderState::Unconfigured; }
        ProviderState::Healthy
    }
    fn base_url(&self) -> String { "https://generativelanguage.googleapis.com".to_string() }
    fn completion_handler(&self) -> Box<dyn Completion> {
        Box::new(GeminiCompletionHandler {
            api_key: self.api_key.clone(),
            model_id: self.model_id.clone(),
        })
    }
}

pub struct GeminiCompletionHandler {
    pub api_key: String,
    pub model_id: String,
}

#[async_trait]
impl Completion for GeminiCompletionHandler {
    async fn chat_stream(
        &self,
        app: &AppHandle,
        messages: Vec<crate::commands::chat::Message>,
    ) -> Result<(), String> {
        let client = reqwest::Client::new();
        let contents: Vec<serde_json::Value> = messages.iter().map(|m| {
            json!({
                "role": if m.role == "assistant" { "model" } else { "user" },
                "parts": [{ "text": m.content }]
            })
        }).collect();

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
            self.model_id, self.api_key
        );

        let body = json!({ "contents": contents });
        let res = client.post(url).json(&body).send().await.map_err(|e| e.to_string())?;
        let mut stream = res.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| e.to_string())?;
            let text = String::from_utf8_lossy(&chunk);
            for line in text.lines() {
                if line.starts_with("data: ") {
                    let data = line.trim_start_matches("data: ");
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(content) = json["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                            let _ = app.emit("chat_delta", content);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
