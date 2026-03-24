use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::fs;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub id: String,
    pub name: String,
    pub file_path: String, // relative to AppData/models/
    pub file_size: u64,
    pub quantization: Option<String>,
    pub context_length_hint: Option<u32>,
    pub date_added: String,
}

pub struct ModelManager {
    pub app_data_dir: PathBuf,
}

impl ModelManager {
    pub fn new(app_handle: &AppHandle) -> Self {
        let app_data_dir = app_handle.path().app_data_dir().expect("Failed to get app data dir");
        let models_dir = app_data_dir.join("models");
        if !models_dir.exists() {
            fs::create_dir_all(&models_dir).expect("Failed to create models directory");
        }
        Self { app_data_dir }
    }

    pub fn get_models_dir(&self) -> PathBuf {
        self.app_data_dir.join("models")
    }

    pub fn get_registry_path(&self) -> PathBuf {
        self.app_data_dir.join("models.json")
    }

    pub fn list_models(&self) -> Vec<ModelEntry> {
        let registry_path = self.get_registry_path();
        if !registry_path.exists() {
            return Vec::new();
        }

        let content = fs::read_to_string(registry_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    }

    pub fn save_models(&self, models: Vec<ModelEntry>) -> Result<(), String> {
        let registry_path = self.get_registry_path();
        let content = serde_json::to_string_pretty(&models).map_err(|e| e.to_string())?;
        fs::write(registry_path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn add_model(&self, entry: ModelEntry) -> Result<(), String> {
        let mut models = self.list_models();
        models.push(entry);
        self.save_models(models)
    }

    pub fn delete_model(&self, id: &str) -> Result<(), String> {
        let mut models = self.list_models();
        if let Some(index) = models.iter().position(|m| m.id == id) {
            let entry = models.remove(index);
            let file_path = self.get_models_dir().join(&entry.file_path);
            if file_path.exists() {
                fs::remove_file(file_path).map_err(|e| e.to_string())?;
            }
            self.save_models(models)?;
        }
        Ok(())
    }

    pub fn sync_registry(&self) -> Result<Vec<ModelEntry>, String> {
        let models_dir = self.get_models_dir();
        let mut models = self.list_models();
        let mut changed = false;

        // 1. Remove entries where file is missing
        let initial_count = models.len();
        models.retain(|m| models_dir.join(&m.file_path).exists());
        if models.len() != initial_count {
            changed = true;
        }

        // 2. Scan for untracked .gguf files
        if let Ok(entries) = fs::read_dir(&models_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("gguf") {
                    let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
                    if !models.iter().any(|m| m.file_path == file_name) {
                        // Found untracked file
                        let metadata = entry.metadata().map_err(|e| e.to_string())?;
                        use chrono::Utc;
                        use uuid::Uuid;
                        
                        models.push(ModelEntry {
                            id: Uuid::new_v4().to_string(),
                            name: file_name.clone(),
                            file_path: file_name,
                            file_size: metadata.len(),
                            quantization: None,
                            context_length_hint: None,
                            date_added: Utc::now().to_rfc3339(),
                        });
                        changed = true;
                    }
                }
            }
        }

        if changed {
            self.save_models(models.clone())?;
        }

        Ok(models)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_sync_registry_scans_new_file() {
        let dir = tempdir().unwrap();
        let app_data_dir = dir.path().to_path_buf();
        let models_dir = app_data_dir.join("models");
        fs::create_dir_all(&models_dir).unwrap();

        // Create a fake gguf file
        let model_file = models_dir.join("test.gguf");
        fs::write(&model_file, "fake data").unwrap();

        let manager = ModelManager { app_data_dir };
        let models = manager.sync_registry().unwrap();

        assert_eq!(models.len(), 1);
        assert_eq!(models[0].name, "test.gguf");
    }
}
