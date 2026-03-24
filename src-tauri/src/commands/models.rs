use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Emitter};
use crate::models::{ModelManager, ModelEntry};
use std::path::PathBuf;
use std::fs;
use uuid::Uuid;
use chrono::Utc;

#[tauri::command]
pub async fn sync_models(app: AppHandle) -> Result<Vec<ModelEntry>, String> {
    let manager = ModelManager::new(&app);
    manager.sync_registry()
}

#[tauri::command]
pub async fn list_models(app: AppHandle) -> Result<Vec<ModelEntry>, String> {
    let manager = ModelManager::new(&app);
    Ok(manager.list_models())
}

#[tauri::command]
pub async fn delete_model(app: AppHandle, id: String) -> Result<(), String> {
    let manager = ModelManager::new(&app);
    manager.delete_model(&id)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HfModel {
    pub id: String,
    pub downloads: u32,
    pub likes: u32,
}

#[tauri::command]
pub async fn search_hf_models(query: String) -> Result<Vec<HfModel>, String> {
    let url = format!(
        "https://huggingface.co/api/models?search={}&filter=gguf&sort=downloads&direction=-1&limit=20",
        query
    );
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header("User-Agent", "umagpt")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let models: Vec<HfModel> = res.json().await.map_err(|e| e.to_string())?;
    Ok(models)
}

#[tauri::command]
pub async fn download_model(
    app: AppHandle,
    repo_id: String,
    file_name: String,
) -> Result<ModelEntry, String> {
    let manager = ModelManager::new(&app);
    let dest_path = manager.get_models_dir().join(&file_name);

    if dest_path.exists() {
        return Err("File already exists".to_string());
    }

    let url = format!(
        "https://huggingface.co/{}/resolve/main/{}",
        repo_id, file_name
    );

    let client = reqwest::Client::new();
    let res = client.get(url).send().await.map_err(|e| e.to_string())?;

    let total_size = res
        .content_length()
        .ok_or("Failed to get content length")?;

    use futures_util::StreamExt;
    let mut stream = res.bytes_stream();
    let mut file = fs::File::create(&dest_path).map_err(|e| e.to_string())?;
    let mut downloaded: u64 = 0;

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| e.to_string())?;
        use std::io::Write;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        // Emit progress
        let progress = (downloaded as f64 / total_size as f64) * 100.0;
        let _ = app.emit("download_progress", (file_name.clone(), progress));
    }

    let entry = ModelEntry {
        id: Uuid::new_v4().to_string(),
        name: file_name.clone(),
        file_path: file_name,
        file_size: total_size,
        quantization: None,
        context_length_hint: None,
        date_added: Utc::now().to_rfc3339(),
    };

    manager.add_model(entry.clone())?;
    Ok(entry)
}

#[tauri::command]
pub async fn import_model(app: AppHandle, path: String) -> Result<ModelEntry, String> {
    let manager = ModelManager::new(&app);
    let src_path = PathBuf::from(&path);
    if !src_path.exists() {
        return Err("Source file does not exist".to_string());
    }

    let file_name = src_path.file_name().ok_or("Invalid filename")?.to_str().ok_or("Invalid filename")?;
    let dest_path = manager.get_models_dir().join(file_name);

    // Copy file
    fs::copy(&src_path, &dest_path).map_err(|e| e.to_string())?;

    let metadata = fs::metadata(&dest_path).map_err(|e| e.to_string())?;
    
    let entry = ModelEntry {
        id: Uuid::new_v4().to_string(),
        name: file_name.to_string(),
        file_path: file_name.to_string(),
        file_size: metadata.len(),
        quantization: None, // TODO: parse from name
        context_length_hint: None,
        date_added: Utc::now().to_rfc3339(),
    };

    manager.add_model(entry.clone())?;
    Ok(entry)
}
