use std::collections::HashSet;
use std::fs;
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::{Value as JsonValue, json};
use tauri::{AppHandle, Manager, State};

use crate::core::audio_capture::AudioCaptureState;
use crate::core::path_safety::{
    ensure_scoped_existing_path, ensure_scoped_path, sanitize_file_name, sanitize_path_component,
};

const OPENAI_PORT: u16 = 11434;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationLevel {
    Ok,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStatus {
    pub level: ValidationLevel,
    pub messages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelFormat {
    Gguf,
    Safetensors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GGUFKeyValue {
    pub key: String,
    pub value: JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GGUFMetadata {
    pub format_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub alignment: u64,
    pub tensor_count: usize,
    pub metadata_kv_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_length: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_length: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attention_head_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kv_head_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rope_dimension: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokenizer_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bos_token_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eos_token_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokenizer_tokens: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokenizer_scores: Option<Vec<f32>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_metadata: Vec<GGUFKeyValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub path: String,
    pub name: String,
    pub file_size: u64,
    pub format: ModelFormat,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detected_architecture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_length: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_count: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokenizer_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vocab_size: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_repo_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_repo_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_quantization: Option<String>,
    pub candle_compatible: bool,
    pub validation_status: ValidationStatus,
    pub created_at: String,
    pub metadata: GGUFMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteGGUFFile {
    pub filename: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantization: Option<String>,
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HFModelInfo {
    pub repo_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    pub downloads: u64,
    pub likes: u64,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub architectures: Vec<String>,
    #[serde(default)]
    pub quantizations: Vec<String>,
    #[serde(default)]
    pub gguf_files: Vec<RemoteGGUFFile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_count: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_length: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoteModelFilters {
    pub architecture: Option<String>,
    pub license: Option<String>,
    pub quantization: Option<String>,
    pub max_file_size: Option<u64>,
    pub min_downloads: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SttModelSource {
    Bundled,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttSettings {
    pub source: SttModelSource,
    pub custom_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttDownloadRequest {
    pub repo_id: String,
    pub revision: Option<String>,
    pub model_filename: String,
    pub tokenizer_filename: String,
    pub config_filename: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttDownloadResponse {
    pub model_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefixCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub entries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefixCacheInfo {
    pub enabled: bool,
    pub max_entries: usize,
    pub stats: PrefixCacheStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCardRepo {
    pub repo_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelCardSources {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gguf: Option<ModelCardRepo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safetensors: Option<ModelCardRepo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCardSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    pub tags: Vec<String>,
    pub hf_repo_id: String,
    pub supported_formats: Vec<String>,
    pub has_gguf: bool,
    pub has_safetensors: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<ModelCardSources>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gguf_quantizations: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCardsResponse {
    pub version: u32,
    pub cards: Vec<ModelCardSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCardDownloadResult {
    pub card_id: String,
    pub format: String,
    pub destination_dir: String,
    pub downloaded_files: Vec<String>,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelCardDownloadArgs {
    pub card_id: String,
    pub format: String,
    pub models_root: String,
    pub quantization: Option<String>,
}

static PREFIX_CACHE_STATE: Lazy<Mutex<PrefixCacheInfo>> = Lazy::new(|| {
    Mutex::new(PrefixCacheInfo {
        enabled: true,
        max_entries: 32,
        stats: PrefixCacheStats {
            hits: 0,
            misses: 0,
            entries: 0,
        },
    })
});

static EXPERIMENTAL_FEATURES_ENABLED: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
static STT_SETTINGS_STATE: Lazy<Mutex<SttSettings>> = Lazy::new(|| {
    Mutex::new(SttSettings {
        source: SttModelSource::Bundled,
        custom_dir: None,
    })
});

#[tauri::command]
pub fn get_server_config() -> ServerConfig {
    let addr = SocketAddr::from(([127, 0, 0, 1], OPENAI_PORT));
    let running = TcpStream::connect_timeout(&addr, Duration::from_millis(120)).is_ok();
    ServerConfig {
        port: OPENAI_PORT,
        running,
    }
}

#[tauri::command]
pub fn get_experimental_features_enabled() -> Result<bool, String> {
    EXPERIMENTAL_FEATURES_ENABLED
        .lock()
        .map(|v| *v)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_experimental_features_enabled(enabled: bool) -> Result<(), String> {
    let mut guard = EXPERIMENTAL_FEATURES_ENABLED
        .lock()
        .map_err(|e| e.to_string())?;
    *guard = enabled;
    Ok(())
}

#[tauri::command]
pub fn get_prefix_cache_info() -> Result<PrefixCacheInfo, String> {
    PREFIX_CACHE_STATE
        .lock()
        .map(|v| v.clone())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_prefix_cache_enabled(enabled: bool, max_entries: usize) -> Result<(), String> {
    let mut guard = PREFIX_CACHE_STATE.lock().map_err(|e| e.to_string())?;
    guard.enabled = enabled;
    guard.max_entries = max_entries.max(1);
    Ok(())
}

#[tauri::command]
pub fn clear_prefix_cache() -> Result<(), String> {
    let mut guard = PREFIX_CACHE_STATE.lock().map_err(|e| e.to_string())?;
    guard.stats = PrefixCacheStats {
        hits: 0,
        misses: 0,
        entries: 0,
    };
    Ok(())
}

#[tauri::command]
pub fn start_voice_recording(
    app: AppHandle,
    state: State<'_, AudioCaptureState>,
) -> Result<(), String> {
    state.start(app)
}

#[tauri::command]
pub fn stop_voice_recording_and_transcribe(
    state: State<'_, AudioCaptureState>,
    _language: Option<String>,
) -> Result<String, String> {
    let (samples, sample_rate) = state.stop()?;
    if samples.is_empty() {
        return Ok(String::new());
    }

    let duration = if sample_rate == 0 {
        0.0
    } else {
        samples.len() as f32 / sample_rate as f32
    };
    Ok(format!("[voice {:.1}s]", duration))
}

#[tauri::command]
pub fn cancel_voice_recording(state: State<'_, AudioCaptureState>) -> Result<(), String> {
    state.cancel()
}

#[tauri::command]
pub fn get_stt_settings() -> Result<SttSettings, String> {
    STT_SETTINGS_STATE
        .lock()
        .map(|v| v.clone())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_stt_settings(settings: SttSettings) -> Result<(), String> {
    let mut guard = STT_SETTINGS_STATE.lock().map_err(|e| e.to_string())?;
    *guard = settings;
    Ok(())
}

#[tauri::command]
pub fn download_stt_model(
    app: AppHandle,
    req: SttDownloadRequest,
) -> Result<SttDownloadResponse, String> {
    let base = app
        .path()
        .app_local_data_dir()
        .map_err(|e| e.to_string())?
        .join("oxide-lab")
        .join("stt");
    let safe_repo = sanitize_path_component(
        &req.repo_id.replace(['/', '\\'], "_"),
        "STT repository identifier",
    )?;
    let target_dir = base.join(safe_repo);
    fs::create_dir_all(&target_dir).map_err(|e| e.to_string())?;

    for file_name in [
        &req.model_filename,
        &req.tokenizer_filename,
        &req.config_filename,
    ] {
        let safe_file_name = sanitize_file_name(file_name)?;
        let path = target_dir.join(safe_file_name);
        if !path.exists() {
            fs::write(&path, b"").map_err(|e| e.to_string())?;
        }
    }

    let target_dir_string = target_dir.to_string_lossy().to_string();
    let mut guard = STT_SETTINGS_STATE.lock().map_err(|e| e.to_string())?;
    *guard = SttSettings {
        source: SttModelSource::Custom,
        custom_dir: Some(target_dir_string.clone()),
    };

    Ok(SttDownloadResponse {
        model_dir: target_dir_string,
    })
}

#[tauri::command]
pub fn parse_gguf_metadata(app: AppHandle, file_path: String) -> Result<GGUFMetadata, String> {
    let path = ensure_scoped_existing_path(&app, &file_path)?;
    Ok(default_gguf_metadata_for_path(&path))
}

#[tauri::command]
pub fn scan_models_folder(app: AppHandle, folder_path: String) -> Result<Vec<ModelInfo>, String> {
    let root = ensure_scoped_existing_path(&app, &folder_path)?;
    if !root.exists() || !root.is_dir() {
        return Err(format!("Folder not found: {}", root.display()));
    }

    let mut files = Vec::new();
    collect_model_files(&root, &mut files)?;
    let mut models = files
        .into_iter()
        .filter_map(|path| model_info_from_path(&path).ok())
        .collect::<Vec<_>>();
    models.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(models)
}

#[tauri::command]
pub fn delete_local_model(app: AppHandle, model_path: String) -> Result<(), String> {
    let path = ensure_scoped_path(&app, &model_path, false)?;
    if !path.exists() {
        return Ok(());
    }
    if path.is_file() {
        fs::remove_file(path).map_err(|e| e.to_string())
    } else if path.is_dir() {
        fs::remove_dir_all(path).map_err(|e| e.to_string())
    } else {
        Ok(())
    }
}

#[tauri::command]
pub fn update_model_manifest(
    app: AppHandle,
    model_path: String,
    repo_name: Option<String>,
    publisher: Option<String>,
) -> Result<(), String> {
    let path = ensure_scoped_existing_path(&app, &model_path)?;
    if !path.exists() {
        return Err(format!("Model path not found: {}", path.display()));
    }
    let manifest_path = PathBuf::from(format!("{}.oxide-manifest.json", path.to_string_lossy()));
    let payload = json!({
        "repo_name": repo_name,
        "publisher": publisher,
        "updated_at": Utc::now().to_rfc3339(),
    });
    fs::write(
        manifest_path,
        serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_huggingface_gguf(
    query: String,
    filters: Option<RemoteModelFilters>,
) -> Result<Vec<HFModelInfo>, String> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let limit = filters.and_then(|f| f.limit).unwrap_or(20).clamp(1, 100);
    let client = reqwest::Client::new();
    let response = client
        .get("https://huggingface.co/api/models")
        .query(&[
            ("search", trimmed),
            ("filter", "gguf"),
            ("limit", &limit.to_string()),
            ("full", "true"),
            ("config", "true"),
        ])
        .send()
        .await
        .map_err(|e| format!("Failed to query Hugging Face: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Hugging Face request failed: {e}"))?;

    let raw: Vec<JsonValue> = response
        .json()
        .await
        .map_err(|e| format!("Invalid Hugging Face response: {e}"))?;

    let mut result = Vec::new();
    for item in raw {
        let Some(repo_id) = item.get("id").and_then(JsonValue::as_str) else {
            continue;
        };

        let mut gguf_files = Vec::new();
        if let Some(siblings) = item.get("siblings").and_then(JsonValue::as_array) {
            for sibling in siblings {
                let Some(filename) = sibling.get("rfilename").and_then(JsonValue::as_str) else {
                    continue;
                };
                if !filename.to_lowercase().ends_with(".gguf") {
                    continue;
                }
                let size = sibling
                    .get("size")
                    .and_then(JsonValue::as_u64)
                    .or_else(|| {
                        sibling
                            .get("lfs")
                            .and_then(|lfs| lfs.get("size"))
                            .and_then(JsonValue::as_u64)
                    })
                    .unwrap_or(0);
                let sha256 = sibling
                    .get("lfs")
                    .and_then(|lfs| lfs.get("sha256"))
                    .and_then(JsonValue::as_str)
                    .map(|s| s.to_string());
                gguf_files.push(RemoteGGUFFile {
                    filename: filename.to_string(),
                    size,
                    sha256,
                    quantization: extract_quantization(filename),
                    download_url: format!(
                        "https://huggingface.co/{repo_id}/resolve/main/{filename}"
                    ),
                });
            }
        }

        if gguf_files.is_empty() {
            continue;
        }

        let quantizations = gguf_files
            .iter()
            .filter_map(|f| f.quantization.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        let name = repo_id
            .split('/')
            .next_back()
            .unwrap_or(repo_id)
            .to_string();

        result.push(HFModelInfo {
            repo_id: repo_id.to_string(),
            name,
            author: item
                .get("author")
                .and_then(JsonValue::as_str)
                .map(|s| s.to_string()),
            description: item
                .get("cardData")
                .and_then(|c| c.get("description"))
                .and_then(JsonValue::as_str)
                .map(|s| s.to_string()),
            license: item
                .get("cardData")
                .and_then(|c| c.get("license"))
                .and_then(JsonValue::as_str)
                .map(|s| s.to_string()),
            downloads: item
                .get("downloads")
                .and_then(JsonValue::as_u64)
                .unwrap_or(0),
            likes: item.get("likes").and_then(JsonValue::as_u64).unwrap_or(0),
            tags: item
                .get("tags")
                .and_then(JsonValue::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(JsonValue::as_str)
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            architectures: item
                .get("config")
                .and_then(|c| c.get("architectures"))
                .and_then(JsonValue::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(JsonValue::as_str)
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            quantizations,
            gguf_files,
            last_modified: item
                .get("lastModified")
                .and_then(JsonValue::as_str)
                .map(|s| s.to_string()),
            created_at: item
                .get("createdAt")
                .and_then(JsonValue::as_str)
                .map(|s| s.to_string()),
            parameter_count: None,
            context_length: None,
        });
    }

    Ok(result)
}

#[tauri::command]
pub async fn get_model_readme(repo_id: String) -> Result<String, String> {
    let trimmed = repo_id.trim();
    if trimmed.is_empty() {
        return Err("Repository id is empty".to_string());
    }
    let url = format!("https://huggingface.co/{trimmed}/raw/main/README.md");
    let response = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to request README: {e}"))?;

    if response.status().as_u16() == 404 {
        return Ok("README.md не найден для этой модели.".to_string());
    }

    response
        .error_for_status()
        .map_err(|e| format!("Failed to fetch README: {e}"))?
        .text()
        .await
        .map_err(|e| format!("Failed to decode README: {e}"))
}

#[tauri::command]
pub fn get_model_cards() -> Result<ModelCardsResponse, String> {
    load_model_cards_from_path(&resolve_default_model_cards_path()?)
}

#[tauri::command]
pub fn import_model_cards(
    app: AppHandle,
    config_path: String,
) -> Result<ModelCardsResponse, String> {
    let path = ensure_scoped_existing_path(&app, &config_path)?;
    load_model_cards_from_path(&path)
}

#[tauri::command]
pub fn reset_model_cards() -> Result<ModelCardsResponse, String> {
    load_model_cards_from_path(&resolve_default_model_cards_path()?)
}

#[tauri::command]
pub async fn download_model_card_format(
    app: AppHandle,
    args: ModelCardDownloadArgs,
) -> Result<ModelCardDownloadResult, String> {
    if args.format != "gguf" && args.format != "safetensors" {
        return Err(format!("Unsupported format: {}", args.format));
    }

    let cards_path = resolve_default_model_cards_path()?;
    let raw = fs::read_to_string(cards_path).map_err(|e| e.to_string())?;
    let root: JsonValue = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let cards = root
        .get("cards")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| "Invalid model cards format".to_string())?;

    let card = cards
        .iter()
        .find(|c| c.get("id").and_then(JsonValue::as_str) == Some(args.card_id.as_str()))
        .ok_or_else(|| format!("Card not found: {}", args.card_id))?;

    let safe_card_id = sanitize_path_component(&args.card_id, "Card ID")?;
    let mut download_url = None;
    let mut filename = None;
    if args.format == "gguf"
        && let Some(files) = card.get("files").and_then(JsonValue::as_array)
    {
        if let Some(q) = args.quantization.as_ref()
            && let Some(entry) = files.iter().find(|f| {
                f.get("name")
                    .and_then(JsonValue::as_str)
                    .map(|name| name.eq_ignore_ascii_case(q))
                    .unwrap_or(false)
            })
        {
            download_url = entry
                .get("url")
                .and_then(JsonValue::as_str)
                .map(|s| s.to_string());
            filename = entry
                .get("url")
                .and_then(JsonValue::as_str)
                .and_then(|s| s.rsplit('/').next())
                .map(|s| s.to_string());
        }
        if download_url.is_none()
            && let Some(entry) = files.first()
        {
            download_url = entry
                .get("url")
                .and_then(JsonValue::as_str)
                .map(|s| s.to_string());
            filename = entry
                .get("url")
                .and_then(JsonValue::as_str)
                .and_then(|s| s.rsplit('/').next())
                .map(|s| s.to_string());
        }
    }

    let url = download_url.ok_or_else(|| "No downloadable file for selected format".to_string())?;
    let file_name = sanitize_file_name(&filename.unwrap_or_else(|| "model.bin".to_string()))?;
    let bytes = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Download failed: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Download failed: {e}"))?
        .bytes()
        .await
        .map_err(|e| format!("Failed to read download bytes: {e}"))?;

    let scoped_models_root = ensure_scoped_path(&app, &args.models_root, false)?;
    let destination_dir = scoped_models_root.join(&safe_card_id);
    fs::create_dir_all(&destination_dir).map_err(|e| e.to_string())?;
    let destination_path = destination_dir.join(&file_name);
    fs::write(&destination_path, &bytes).map_err(|e| e.to_string())?;

    Ok(ModelCardDownloadResult {
        card_id: args.card_id,
        format: args.format,
        destination_dir: destination_dir.to_string_lossy().to_string(),
        downloaded_files: vec![destination_path.to_string_lossy().to_string()],
        total_bytes: bytes.len() as u64,
    })
}

fn resolve_default_model_cards_path() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let candidates = [
        cwd.join("model_cards.json"),
        cwd.join("..").join("model_cards.json"),
        cwd.join("..").join("..").join("model_cards.json"),
    ];
    candidates
        .into_iter()
        .find(|p| p.exists())
        .ok_or_else(|| "model_cards.json not found".to_string())
}

fn load_model_cards_from_path(path: &Path) -> Result<ModelCardsResponse, String> {
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let root: JsonValue = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let version = root.get("version").and_then(JsonValue::as_u64).unwrap_or(1) as u32;
    let cards_raw = root
        .get("cards")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| "Invalid model cards format".to_string())?;

    let cards = cards_raw
        .iter()
        .filter_map(normalize_model_card)
        .collect::<Vec<_>>();

    Ok(ModelCardsResponse { version, cards })
}

fn normalize_model_card(card: &JsonValue) -> Option<ModelCardSummary> {
    let id = card.get("id")?.as_str()?.to_string();
    let name = card
        .get("name")
        .and_then(JsonValue::as_str)
        .unwrap_or(&id)
        .to_string();
    let description = card
        .get("description")
        .and_then(JsonValue::as_str)
        .unwrap_or("")
        .to_string();
    let hf_repo_id = card
        .get("hf_repo_id")
        .and_then(JsonValue::as_str)
        .unwrap_or("")
        .to_string();
    let tags = card
        .get("tags")
        .and_then(JsonValue::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(JsonValue::as_str)
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let gguf_quantizations = card
        .get("files")
        .and_then(JsonValue::as_array)
        .map(|files| {
            files
                .iter()
                .filter_map(|f| f.get("name").and_then(JsonValue::as_str))
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .filter(|v| !v.is_empty());

    let has_gguf = card.get("files").is_some() || card.get("hf_filename").is_some();
    let has_safetensors = card
        .get("sources")
        .and_then(|s| s.get("safetensors"))
        .is_some();

    let mut supported_formats = Vec::new();
    if has_gguf {
        supported_formats.push("gguf".to_string());
    }
    if has_safetensors {
        supported_formats.push("safetensors".to_string());
    }

    let sources = if hf_repo_id.is_empty() {
        None
    } else {
        Some(ModelCardSources {
            gguf: Some(ModelCardRepo {
                repo_id: hf_repo_id.clone(),
                revision: None,
            }),
            safetensors: None,
        })
    };

    Some(ModelCardSummary {
        id,
        name,
        description,
        family: card
            .get("family")
            .and_then(JsonValue::as_str)
            .map(|s| s.to_string()),
        tags,
        hf_repo_id,
        supported_formats,
        has_gguf,
        has_safetensors,
        sources,
        gguf_quantizations,
    })
}

fn collect_model_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir).map_err(|e| e.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|e| e.to_string())?;
        if file_type.is_dir() {
            collect_model_files(&path, out)?;
            continue;
        }
        if file_type.is_file() && is_supported_model_file(&path) {
            out.push(path);
        }
    }
    Ok(())
}

fn is_supported_model_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let ext_l = ext.to_ascii_lowercase();
            ext_l == "gguf" || ext_l == "safetensors"
        })
        .unwrap_or(false)
}

fn model_info_from_path(path: &Path) -> Result<ModelInfo, String> {
    let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
    let file_size = metadata.len();
    let created_at = file_timestamp_iso(&metadata);
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let format = if extension == "gguf" {
        ModelFormat::Gguf
    } else {
        ModelFormat::Safetensors
    };

    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("model")
        .to_string();
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("model")
        .to_string();

    let model_metadata = default_gguf_metadata_for_path(path);
    let quantization = extract_quantization(&file_name);

    Ok(ModelInfo {
        path: path.to_string_lossy().to_string(),
        name,
        file_size,
        format,
        architecture: None,
        detected_architecture: None,
        model_name: None,
        version: None,
        context_length: None,
        parameter_count: None,
        quantization,
        tokenizer_type: None,
        vocab_size: None,
        source_repo_id: None,
        source_repo_name: None,
        source_quantization: None,
        candle_compatible: true,
        validation_status: ValidationStatus {
            level: ValidationLevel::Ok,
            messages: Vec::new(),
        },
        created_at,
        metadata: model_metadata,
    })
}

fn default_gguf_metadata_for_path(path: &Path) -> GGUFMetadata {
    GGUFMetadata {
        format_version: 3,
        architecture: None,
        name: path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string()),
        version: None,
        author: None,
        alignment: 32,
        tensor_count: 0,
        metadata_kv_count: 0,
        parameter_count: None,
        size_label: None,
        context_length: None,
        embedding_length: None,
        block_count: None,
        attention_head_count: None,
        kv_head_count: None,
        rope_dimension: None,
        tokenizer_model: None,
        bos_token_id: None,
        eos_token_id: None,
        tokenizer_tokens: None,
        tokenizer_scores: None,
        custom_metadata: Vec::new(),
    }
}

fn file_timestamp_iso(metadata: &fs::Metadata) -> String {
    metadata
        .created()
        .or_else(|_| metadata.modified())
        .ok()
        .map(|ts| DateTime::<Utc>::from(ts).to_rfc3339())
        .unwrap_or_else(|| Utc::now().to_rfc3339())
}

fn extract_quantization(filename: &str) -> Option<String> {
    let upper = filename.to_uppercase();
    let patterns = [
        "Q2_K", "Q3_K", "Q4_0", "Q4_1", "Q4_K", "Q5_0", "Q5_1", "Q5_K", "Q6_K", "Q8_0", "Q8_K",
        "F16", "F32", "BF16",
    ];
    patterns
        .iter()
        .find(|p| upper.contains(**p))
        .map(|p| (*p).to_string())
}
