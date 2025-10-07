//! Model manager API for local GGUF files and Hugging Face integration.
//!
//! This module provides:
//! * Comprehensive GGUF metadata parsing (including tokenizer metadata and validation)
//! * Recursive scanning of local folders with Candle compatibility checks
//! * Hugging Face Hub search focused on GGUF artifacts with filtering
//! * Download helper with progress events bridged to the Svelte frontend

use crate::models::registry::{detect_arch, ArchKind};
use candle::quantized::gguf_file::{self, Content, Value as GgufValue, VersionedMagic};
use chrono::{DateTime, Utc};
use hf_hub::api::tokio::{ApiBuilder, Progress as HubProgress};
use once_cell::sync::OnceCell;
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tauri::{async_runtime, AppHandle, Emitter};

/// Validation severity level for GGUF files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationLevel {
    Ok,
    Warning,
    Error,
}

/// Validation outcome that frontend can render.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStatus {
    pub level: ValidationLevel,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<String>,
}

/// Key-value representation for additional GGUF metadata that is not mapped explicitly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GGUFKeyValue {
    pub key: String,
    pub value: JsonValue,
}

/// Parsed GGUF metadata payload sent to the UI.
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokenizer_tokens: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokenizer_scores: Option<Vec<f32>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_metadata: Vec<GGUFKeyValue>,
}

/// Local model description returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub path: PathBuf,
    pub file_size: u64,
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
    pub candle_compatible: bool,
    pub validation_status: ValidationStatus,
    pub created_at: DateTime<Utc>,
    pub metadata: GGUFMetadata,
}

/// Remote GGUF file descriptor from Hugging Face.
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

/// Remote model listing entry.
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub architectures: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub quantizations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
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

/// Sorting options for remote search.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelSortField {
    Downloads,
    Likes,
    Updated,
    FileSize,
}

/// Sort order for remote search.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Filters accepted by the Hugging Face search command.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_file_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_downloads: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<ModelSortField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<SortOrder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
}

/// Download outcome returned to the frontend after copying the cached artifact.
#[derive(Debug, Clone, Serialize)]
pub struct DownloadedFileInfo {
    pub repo_id: String,
    pub filename: String,
    pub local_path: PathBuf,
    pub size: u64,
}

/// Download stage for progress events.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadStage {
    Started,
    InProgress,
    Finished,
}

/// Event payload used to report download progress to the frontend.
#[derive(Debug, Clone, Serialize)]
pub struct DownloadProgressPayload {
    pub download_id: String,
    pub filename: String,
    pub current: u64,
    pub total: u64,
    pub stage: DownloadStage,
}

/// Wrapper around Hugging Face download progress reporting that emits Tauri events.
#[derive(Clone)]
struct HubProgressEmitter {
    app: AppHandle,
    download_id: String,
    filename: String,
    current: Arc<AtomicU64>,
    total: Arc<AtomicU64>,
}

impl HubProgressEmitter {
    fn new(app: AppHandle, download_id: String, filename: String) -> Self {
        Self {
            app,
            download_id,
            filename,
            current: Arc::new(AtomicU64::new(0)),
            total: Arc::new(AtomicU64::new(0)),
        }
    }

    fn emit(&self, stage: DownloadStage) {
        let payload = DownloadProgressPayload {
            download_id: self.download_id.clone(),
            filename: self.filename.clone(),
            current: self.current.load(Ordering::Relaxed),
            total: self.total.load(Ordering::Relaxed),
            stage,
        };
        let _ = self.app.emit("model-download-progress", &payload);
    }
}

impl HubProgress for HubProgressEmitter {
    async fn init(&mut self, size: usize, _filename: &str) {
        self.total.store(size as u64, Ordering::Relaxed);
        self.current.store(0, Ordering::Relaxed);
        self.emit(DownloadStage::Started);
    }

    async fn update(&mut self, size: usize) {
        self.current.fetch_add(size as u64, Ordering::Relaxed);
        self.emit(DownloadStage::InProgress);
    }

    async fn finish(&mut self) {
        let total = self.total.load(Ordering::Relaxed);
        self.current.store(total, Ordering::Relaxed);
        self.emit(DownloadStage::Finished);
    }
}

/// Command: parse a GGUF metadata section from a file.
#[tauri::command]
pub async fn parse_gguf_metadata(file_path: String) -> Result<GGUFMetadata, String> {
    let path = PathBuf::from(&file_path);
    async_runtime::spawn_blocking(move || {
        let envelope = read_gguf_metadata(&path, true)?;
        Ok(envelope.metadata)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Command: scan a folder recursively for GGUF models.
#[tauri::command]
pub async fn scan_models_folder(folder_path: String) -> Result<Vec<ModelInfo>, String> {
    let path = PathBuf::from(&folder_path);
    async_runtime::spawn_blocking(move || scan_directory(&path))
        .await
        .map_err(|e| e.to_string())?
}

/// Backwards-compatible alias for legacy frontend code.
#[tauri::command]
pub async fn scan_local_models_folder(folder_path: String) -> Result<Vec<ModelInfo>, String> {
    scan_models_folder(folder_path).await
}

/// Command: delete a local model file.
#[tauri::command]
pub async fn delete_local_model(model_path: String) -> Result<(), String> {
    let path = PathBuf::from(model_path);
    async_runtime::spawn_blocking(move || {
        if !path.exists() {
            return Err(format!("File does not exist: {}", path.display()));
        }
        if !path.is_file() {
            return Err(format!("Path is not a file: {}", path.display()));
        }
        fs::remove_file(&path).map_err(|e| format!("Failed to delete file: {e}"))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Command: search Hugging Face Hub for GGUF models.
#[tauri::command]
pub async fn search_huggingface_gguf(
    query: String,
    filters: ModelFilters,
) -> Result<Vec<HFModelInfo>, String> {
    let client = build_http_client()?;
    let limit = filters.limit.unwrap_or(20).clamp(1, 100);
    let offset = filters.offset.unwrap_or(0);

    let mut params: Vec<(&str, String)> = vec![
        ("limit", limit.to_string()),
        ("full", "true".to_string()),
        ("config", "true".to_string()),
        ("sort", "downloads".to_string()),
    ];

    if offset > 0 {
        params.push(("offset", offset.to_string()));
    }
    if !query.trim().is_empty() {
        params.push(("search", query.trim().to_string()));
    }

    let response = client
        .get("https://huggingface.co/api/models")
        .query(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to query Hugging Face: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Hugging Face request failed: {e}"))?;

    let items: Vec<HFSearchModel> = response
        .json()
        .await
        .map_err(|e| format!("Failed to decode Hugging Face response: {e}"))?;

    let mut results = Vec::new();
    for item in items {
        if item.private.unwrap_or(false) {
            continue;
        }
        let detail = fetch_model_detail(&client, &item.id).await?;
        if let Some(info) = convert_detail_to_info(detail, &filters)? {
            results.push(info);
        }
    }

    apply_search_sorting(
        &mut results,
        filters.sort_by.as_ref(),
        filters.sort_order.as_ref(),
    );

    if results.len() > limit as usize {
        results.truncate(limit as usize);
    }

    Ok(results)
}

/// Command: download a GGUF file using hf-hub and emit progress events.
#[tauri::command]
pub async fn download_hf_model_file(
    app: AppHandle,
    repo_id: String,
    filename: String,
    destination_dir: String,
) -> Result<DownloadedFileInfo, String> {
    let download_id = format!("{}::{}", repo_id, filename);
    let api = ApiBuilder::new()
        .with_progress(false)
        .build()
        .map_err(|e| format!("Failed to initialize hf-hub API: {e}"))?;

    let repo = api.model(repo_id.clone());
    let progress = HubProgressEmitter::new(app.clone(), download_id.clone(), filename.clone());

    let pointer_path = repo
        .download_with_progress(&filename, progress.clone())
        .await
        .map_err(|e| format!("Download failed: {e}"))?;

    let dest_dir = PathBuf::from(&destination_dir);
    let dest_file = dest_dir.join(&filename);
    let dest_for_copy = dest_file.clone();

    let src = pointer_path.clone();
    async_runtime::spawn_blocking(move || -> Result<(), String> {
        if let Some(parent) = dest_for_copy.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create destination directory: {e}"))?;
        }
        fs::copy(&src, &dest_for_copy)
            .map_err(|e| format!("Failed to copy downloaded file: {e}"))?;
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())??;

    let size = fs::metadata(&dest_file)
        .map_err(|e| format!("Failed to inspect downloaded file: {e}"))?
        .len();

    progress.emit(DownloadStage::Finished);

    Ok(DownloadedFileInfo {
        repo_id,
        filename,
        local_path: dest_file,
        size,
    })
}

/// Envelope returned by GGUF parsing helper.
struct MetadataEnvelope {
    metadata: GGUFMetadata,
    detected_arch: Option<ArchKind>,
    validation: ValidationStatus,
    vocab_size: Option<usize>,
}

fn read_gguf_metadata(path: &Path, include_tokens: bool) -> Result<MetadataEnvelope, String> {
    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()));
    }
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| !ext.eq_ignore_ascii_case("gguf"))
        .unwrap_or(true)
    {
        return Err(format!(
            "Only GGUF files are supported, received: {}",
            path.display()
        ));
    }

    let file = fs::File::open(path).map_err(|e| format!("Failed to open GGUF file: {e}"))?;
    let mut reader = BufReader::new(file);
    let content =
        Content::read(&mut reader).map_err(|e| format!("Failed to parse GGUF file: {e}"))?;

    let version = match content.magic {
        VersionedMagic::GgufV3 => 3,
        VersionedMagic::GgufV2 => 2,
        VersionedMagic::GgufV1 => 1,
    };

    let mut metadata = GGUFMetadata {
        format_version: version,
        architecture: metadata_get_string(&content.metadata, "general.architecture"),
        name: metadata_get_string(&content.metadata, "general.name"),
        version: metadata_get_string(&content.metadata, "general.version"),
        author: metadata_get_string(&content.metadata, "general.author"),
        alignment: metadata_get_u64(&content.metadata, "general.alignment")
            .unwrap_or(gguf_file::DEFAULT_ALIGNMENT),
        tensor_count: content.tensor_infos.len(),
        metadata_kv_count: content.metadata.len(),
        parameter_count: metadata_get_u64(&content.metadata, "general.parameter_count"),
        size_label: metadata_get_string(&content.metadata, "general.size_label"),
        context_length: None,
        embedding_length: None,
        block_count: None,
        attention_head_count: None,
        kv_head_count: None,
        rope_dimension: None,
        tokenizer_model: metadata_get_string(&content.metadata, "tokenizer.ggml.model"),
        bos_token_id: metadata_get_u32(&content.metadata, "tokenizer.ggml.bos_token_id"),
        eos_token_id: metadata_get_u32(&content.metadata, "tokenizer.ggml.eos_token_id"),
        tokenizer_tokens: None,
        tokenizer_scores: None,
        custom_metadata: Vec::new(),
    };

    let arch_key = metadata.architecture.clone();
    metadata.context_length =
        metadata_get_arch_u64(&content.metadata, arch_key.as_deref(), "context_length")
            .or_else(|| metadata_get_u64(&content.metadata, "context_length"));
    metadata.embedding_length =
        metadata_get_arch_u64(&content.metadata, arch_key.as_deref(), "embedding_length");
    metadata.block_count =
        metadata_get_arch_u64(&content.metadata, arch_key.as_deref(), "block_count");
    metadata.attention_head_count = metadata_get_arch_u64(
        &content.metadata,
        arch_key.as_deref(),
        "attention.head_count",
    );
    metadata.kv_head_count = metadata_get_arch_u64(
        &content.metadata,
        arch_key.as_deref(),
        "attention.head_count_kv",
    );
    metadata.rope_dimension = metadata_get_arch_u64(
        &content.metadata,
        arch_key.as_deref(),
        "rope.dimension_count",
    );

    let (tokens, scores, vocab_size) = extract_tokenizer_data(&content.metadata, include_tokens)?;
    metadata.tokenizer_tokens = tokens;
    metadata.tokenizer_scores = scores;

    let custom_metadata =
        build_custom_metadata(&content.metadata, metadata.tokenizer_tokens.is_some());
    metadata.custom_metadata = custom_metadata;

    let detected_arch = detect_arch(&content.metadata);
    let validation = validate_metadata(&metadata, detected_arch.is_some());

    Ok(MetadataEnvelope {
        metadata,
        detected_arch,
        validation,
        vocab_size,
    })
}

fn scan_directory(dir: &Path) -> Result<Vec<ModelInfo>, String> {
    if !dir.exists() {
        return Err(format!("Path does not exist: {}", dir.display()));
    }
    if !dir.is_dir() {
        return Err(format!("Path is not a directory: {}", dir.display()));
    }

    let mut models = Vec::new();
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        let entries = match fs::read_dir(&current) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!(
                    "Warning: failed to read directory {}: {e}",
                    current.display()
                );
                continue;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("gguf"))
                .unwrap_or(false)
            {
                match build_model_info(&path) {
                    Ok(info) => models.push(info),
                    Err(err) => eprintln!(
                        "Warning: Failed to parse GGUF metadata from {}: {err}",
                        path.display()
                    ),
                }
            }
        }
    }

    models.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(models)
}

fn build_model_info(path: &Path) -> Result<ModelInfo, String> {
    let envelope = read_gguf_metadata(path, false)?;
    let file_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    let metadata_fs = fs::metadata(path).map_err(|e| format!("Failed to read metadata: {e}"))?;
    let created_at = metadata_fs
        .created()
        .or_else(|_| metadata_fs.modified())
        .map_err(|e| format!("Failed to read timestamp: {e}"))?;
    let created_at = DateTime::<Utc>::from(created_at);

    let quantization =
        extract_quantization_from_filename(path.file_name().and_then(|s| s.to_str()).unwrap_or(""));

    let vocab_size = envelope.vocab_size;
    let detected_architecture = envelope
        .detected_arch
        .as_ref()
        .map(|arch| format!("{arch:?}"));

    let parameter_string = envelope.metadata.size_label.clone().or_else(|| {
        envelope
            .metadata
            .parameter_count
            .map(format_parameter_count)
    });

    Ok(ModelInfo {
        name: file_name,
        path: path.to_path_buf(),
        file_size: metadata_fs.len(),
        architecture: envelope.metadata.architecture.clone(),
        detected_architecture,
        model_name: envelope.metadata.name.clone(),
        version: envelope.metadata.version.clone(),
        context_length: envelope.metadata.context_length,
        parameter_count: parameter_string,
        quantization,
        tokenizer_type: envelope.metadata.tokenizer_model.clone(),
        vocab_size,
        candle_compatible: envelope.detected_arch.is_some(),
        validation_status: envelope.validation,
        created_at,
        metadata: envelope.metadata,
    })
}

fn build_http_client() -> Result<Client, String> {
    Client::builder()
        .user_agent(format!(
            "oxide-lab/{} (https://github.com/FerrisMind/Oxide-Lab)",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))
}

async fn fetch_model_detail(client: &Client, repo_id: &str) -> Result<HFModelDetail, String> {
    let url = format!("https://huggingface.co/api/models/{repo_id}");
    client
        .get(url)
        .query(&[("blobs", "true"), ("config", "true"), ("cardData", "true")])
        .send()
        .await
        .map_err(|e| format!("Failed to fetch model detail: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Model detail request failed: {e}"))?
        .json::<HFModelDetail>()
        .await
        .map_err(|e| format!("Failed to decode model detail: {e}"))
}

fn convert_detail_to_info(
    detail: HFModelDetail,
    filters: &ModelFilters,
) -> Result<Option<HFModelInfo>, String> {
    if detail.private.unwrap_or(false) {
        return Ok(None);
    }

    let license = extract_license(&detail);
    if let Some(expected) = filters.license.as_ref() {
        if license
            .as_ref()
            .map(|l| !l.eq_ignore_ascii_case(expected))
            .unwrap_or(true)
        {
            return Ok(None);
        }
    }

    let architectures = extract_architectures(&detail);
    if let Some(target_arch) = filters.architecture.as_ref() {
        if !architectures
            .iter()
            .any(|arch| arch.eq_ignore_ascii_case(target_arch))
        {
            return Ok(None);
        }
    }

    let gguf_files: Vec<RemoteGGUFFile> = detail
        .siblings
        .iter()
        .filter(|file| file.rfilename.to_lowercase().ends_with(".gguf"))
        .filter_map(|file| {
            let size = file.size?;
            if let Some(limit) = filters.max_file_size {
                if size > limit {
                    return None;
                }
            }
            let quant = extract_quantization_from_filename(&file.rfilename);
            if let Some(expected) = filters.quantization.as_ref() {
                let matches = quant
                    .as_ref()
                    .map(|q| q.eq_ignore_ascii_case(expected))
                    .unwrap_or(false);
                if !matches {
                    return None;
                }
            }
            let revision = detail.sha.as_deref().unwrap_or("main");
            let download_url = format!(
                "https://huggingface.co/{}/resolve/{}/{}",
                detail.id, revision, file.rfilename
            );
            Some(RemoteGGUFFile {
                filename: file.rfilename.clone(),
                size,
                sha256: file.lfs.as_ref().and_then(|lfs| lfs.sha256.clone()),
                quantization: quant,
                download_url,
            })
        })
        .collect();

    if gguf_files.is_empty() {
        return Ok(None);
    }

    let downloads = detail.downloads.unwrap_or(0);
    if let Some(min_downloads) = filters.min_downloads {
        if downloads < min_downloads {
            return Ok(None);
        }
    }

    let quantizations: Vec<String> = gguf_files
        .iter()
        .filter_map(|f| f.quantization.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();

    let description = extract_description(&detail);
    let parameter_count = extract_parameter_count_string(&detail);
    let context_length = extract_context_length(&detail);

    let info = HFModelInfo {
        repo_id: detail.id.clone(),
        name: detail.model_id.clone().unwrap_or_else(|| detail.id.clone()),
        author: detail.author.clone(),
        description,
        license,
        downloads,
        likes: detail.likes.unwrap_or(0),
        tags: detail.tags.unwrap_or_default(),
        architectures,
        quantizations,
        gguf_files,
        last_modified: detail.last_modified.clone(),
        created_at: detail.created_at.clone(),
        parameter_count,
        context_length,
    };

    Ok(Some(info))
}

fn apply_search_sorting(
    models: &mut [HFModelInfo],
    sort_by: Option<&ModelSortField>,
    sort_order: Option<&SortOrder>,
) {
    let field = sort_by.unwrap_or(&ModelSortField::Downloads);
    models.sort_by(|a, b| match field {
        ModelSortField::Downloads => a.downloads.cmp(&b.downloads),
        ModelSortField::Likes => a.likes.cmp(&b.likes),
        ModelSortField::Updated => {
            parse_datetime(&a.last_modified).cmp(&parse_datetime(&b.last_modified))
        }
        ModelSortField::FileSize => {
            let a_size = a.gguf_files.iter().map(|f| f.size).max().unwrap_or(0);
            let b_size = b.gguf_files.iter().map(|f| f.size).max().unwrap_or(0);
            a_size.cmp(&b_size)
        }
    });

    if !matches!(sort_order, Some(SortOrder::Asc)) {
        models.reverse();
    }
}

fn validate_metadata(metadata: &GGUFMetadata, candle_ready: bool) -> ValidationStatus {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if metadata.format_version != 3 {
        errors.push(format!(
            "Unsupported GGUF version {} (expected 3)",
            metadata.format_version
        ));
    }

    if metadata.tensor_count == 0 {
        errors.push("GGUF tensor section is empty".to_string());
    }

    if metadata.architecture.is_none() {
        warnings.push("Missing general.architecture metadata".to_string());
    }

    if metadata.tokenizer_model.is_none() {
        warnings.push("Missing tokenizer.ggml.model metadata".to_string());
    }

    if metadata.tokenizer_tokens.is_none() {
        // Проверим, есть ли другие способы восстановить токенизатор
        let has_tokenizer_json = metadata
            .custom_metadata
            .iter()
            .any(|kv| kv.key.contains("tokenizer") && kv.key.contains("json"));

        if !has_tokenizer_json {
            errors.push(
                "Tokenizer tokens are absent in metadata (GGUF must embed tokenizer definition)"
                    .to_string(),
            );
        }
    }

    if metadata.bos_token_id.is_none() || metadata.eos_token_id.is_none() {
        warnings.push("Tokenizer BOS/EOS identifiers are incomplete".to_string());
    }

    if !candle_ready {
        warnings.push("Architecture currently not supported by Candle backend".to_string());
    }

    if errors.is_empty() {
        if warnings.is_empty() {
            ValidationStatus {
                level: ValidationLevel::Ok,
                messages: Vec::new(),
            }
        } else {
            ValidationStatus {
                level: ValidationLevel::Warning,
                messages: warnings,
            }
        }
    } else {
        let mut messages = errors;
        messages.extend(warnings);
        ValidationStatus {
            level: ValidationLevel::Error,
            messages,
        }
    }
}

fn format_parameter_count(count: u64) -> String {
    const ONE_BILLION: f64 = 1_000_000_000.0;
    const ONE_MILLION: f64 = 1_000_000.0;
    const ONE_THOUSAND: f64 = 1_000.0;

    let count_f = count as f64;
    if count_f >= ONE_BILLION {
        format!("{:.1}B", count_f / ONE_BILLION)
    } else if count_f >= ONE_MILLION {
        format!("{:.1}M", count_f / ONE_MILLION)
    } else if count_f >= ONE_THOUSAND {
        format!("{:.1}K", count_f / ONE_THOUSAND)
    } else {
        count.to_string()
    }
}

fn extract_quantization_from_filename(filename: &str) -> Option<String> {
    static REGEX: OnceCell<Regex> = OnceCell::new();
    let regex = REGEX.get_or_init(|| {
        Regex::new(r"(Q\d+_\w+|Q\d+\w+|F16|F32|FP\d+|INT\d+|BF16)")
            .expect("Failed to compile quantization regex")
    });
    regex
        .find(&filename.to_uppercase())
        .map(|m| m.as_str().to_string())
}

fn metadata_get_string(metadata: &HashMap<String, GgufValue>, key: &str) -> Option<String> {
    metadata.get(key).and_then(|value| match value {
        GgufValue::String(s) => Some(s.clone()),
        _ => None,
    })
}

fn metadata_get_u32(metadata: &HashMap<String, GgufValue>, key: &str) -> Option<u32> {
    metadata.get(key).and_then(|value| match value {
        GgufValue::U32(v) => Some(*v),
        GgufValue::U16(v) => Some(*v as u32),
        GgufValue::U8(v) => Some(*v as u32),
        GgufValue::I32(v) if *v >= 0 => Some(*v as u32),
        GgufValue::I16(v) if *v >= 0 => Some(*v as u32),
        GgufValue::I8(v) if *v >= 0 => Some(*v as u32),
        _ => None,
    })
}

fn metadata_get_u64(metadata: &HashMap<String, GgufValue>, key: &str) -> Option<u64> {
    metadata.get(key).and_then(|value| match value {
        GgufValue::U64(v) => Some(*v),
        GgufValue::U32(v) => Some(*v as u64),
        GgufValue::U16(v) => Some(*v as u64),
        GgufValue::U8(v) => Some(*v as u64),
        GgufValue::I64(v) if *v >= 0 => Some(*v as u64),
        GgufValue::I32(v) if *v >= 0 => Some(*v as u64),
        GgufValue::I16(v) if *v >= 0 => Some(*v as u64),
        GgufValue::I8(v) if *v >= 0 => Some(*v as u64),
        _ => None,
    })
}

fn metadata_get_arch_u64(
    metadata: &HashMap<String, GgufValue>,
    arch: Option<&str>,
    suffix: &str,
) -> Option<u64> {
    arch.and_then(|arch| metadata_get_u64(metadata, &format!("{arch}.{suffix}")))
}

fn extract_tokenizer_data(
    metadata: &HashMap<String, GgufValue>,
    include_tokens: bool,
) -> Result<TokenizerExtraction, String> {
    // Попробуем найти токены в различных возможных ключах
    let tokens = metadata
        .get("tokenizer.ggml.tokens")
        .or_else(|| metadata.get("tokenizer.tokens"))
        .or_else(|| metadata.get("tokenizer.vocab"))
        .or_else(|| metadata.get("tokenizer.ggml.vocab"));

    let scores = metadata
        .get("tokenizer.ggml.scores")
        .or_else(|| metadata.get("tokenizer.scores"));

    let mut vocab_size = None;
    let tokenizer_tokens = if include_tokens {
        match tokens {
            Some(GgufValue::Array(values)) => {
                vocab_size = Some(values.len());
                let mut result = Vec::with_capacity(values.len());
                for value in values {
                    if let GgufValue::String(token) = value {
                        result.push(token.clone());
                    } else {
                        return Err("tokenizer.ggml.tokens contains non-string entry".to_string());
                    }
                }
                Some(result)
            }
            Some(_) => {
                return Err("tokenizer.ggml.tokens has unexpected format".to_string());
            }
            None => None,
        }
    } else {
        if let Some(GgufValue::Array(values)) = tokens {
            vocab_size = Some(values.len());
        }
        None
    };

    let tokenizer_scores = if include_tokens {
        match scores {
            Some(GgufValue::Array(values)) => {
                let mut result = Vec::with_capacity(values.len());
                for value in values {
                    match value {
                        GgufValue::F32(v) => result.push(*v),
                        GgufValue::F64(v) => result.push(*v as f32),
                        _ => {
                            return Err(
                                "tokenizer.ggml.scores contains non-floating entry".to_string()
                            )
                        }
                    }
                }
                Some(result)
            }
            Some(_) => {
                return Err("tokenizer.ggml.scores has unexpected format".to_string());
            }
            None => None,
        }
    } else {
        None
    };

    Ok((tokenizer_tokens, tokenizer_scores, vocab_size))
}

fn build_custom_metadata(
    metadata: &HashMap<String, GgufValue>,
    include_token_payload: bool,
) -> Vec<GGUFKeyValue> {
    const EXCLUDED_KEYS: &[&str] = &[
        "general.architecture",
        "general.name",
        "general.version",
        "general.author",
        "general.alignment",
        "general.parameter_count",
        "general.size_label",
        "tokenizer.ggml.model",
        "tokenizer.ggml.bos_token_id",
        "tokenizer.ggml.eos_token_id",
        "tokenizer.ggml.tokens",
        "tokenizer.ggml.scores",
    ];

    let mut result = Vec::new();
    for (key, value) in metadata {
        if EXCLUDED_KEYS.contains(&key.as_str()) {
            continue;
        }
        if !include_token_payload && key.starts_with("tokenizer.ggml.tokens") {
            continue;
        }
        result.push(GGUFKeyValue {
            key: key.clone(),
            value: gguf_value_to_json(value),
        });
    }
    result.sort_by(|a, b| a.key.cmp(&b.key));
    result
}

fn gguf_value_to_json(value: &GgufValue) -> JsonValue {
    match value {
        GgufValue::U8(v) => JsonValue::from(*v),
        GgufValue::I8(v) => JsonValue::from(*v),
        GgufValue::U16(v) => JsonValue::from(*v),
        GgufValue::I16(v) => JsonValue::from(*v),
        GgufValue::U32(v) => JsonValue::from(*v),
        GgufValue::I32(v) => JsonValue::from(*v),
        GgufValue::U64(v) => JsonValue::from(*v),
        GgufValue::I64(v) => JsonValue::from(*v),
        GgufValue::F32(v) => JsonValue::from(*v),
        GgufValue::F64(v) => JsonValue::from(*v),
        GgufValue::Bool(v) => JsonValue::from(*v),
        GgufValue::String(v) => JsonValue::from(v.clone()),
        GgufValue::Array(items) => {
            let converted: Vec<JsonValue> = items.iter().map(gguf_value_to_json).collect();
            JsonValue::from(converted)
        }
    }
}

fn parse_datetime(value: &Option<String>) -> Option<DateTime<Utc>> {
    value
        .as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

fn extract_license(detail: &HFModelDetail) -> Option<String> {
    if let Some(card_data) = detail.card_data.as_ref() {
        if let Some(license) = card_data.get("license").and_then(|v| v.as_str()) {
            return Some(license.to_string());
        }
    }
    detail.tags.as_ref().and_then(|tags| {
        tags.iter()
            .find(|tag| tag.starts_with("license:"))
            .map(|tag| tag.trim_start_matches("license:").to_string())
    })
}

fn extract_description(detail: &HFModelDetail) -> Option<String> {
    detail
        .card_data
        .as_ref()
        .and_then(|card| card.get("description"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn extract_architectures(detail: &HFModelDetail) -> Vec<String> {
    detail
        .config
        .as_ref()
        .and_then(|cfg| cfg.get("architectures"))
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn extract_parameter_count_string(detail: &HFModelDetail) -> Option<String> {
    if let Some(config) = detail.config.as_ref() {
        if let Some(value) = config.get("num_parameters").and_then(|v| v.as_u64()) {
            return Some(format_parameter_count(value));
        }
        if let Some(value) = config.get("n_params").and_then(|v| v.as_u64()) {
            return Some(format_parameter_count(value));
        }
    }

    detail.tags.as_ref().and_then(|tags| {
        tags.iter()
            .find(|tag| tag.starts_with("size:"))
            .map(|tag| tag.trim_start_matches("size:").to_string())
    })
}

fn extract_context_length(detail: &HFModelDetail) -> Option<u64> {
    detail
        .config
        .as_ref()
        .and_then(|cfg| {
            cfg.get("context_length")
                .or_else(|| cfg.get("max_position_embeddings"))
        })
        .and_then(|val| val.as_u64())
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HFSearchModel {
    id: String,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    downloads: Option<u64>,
    #[serde(default)]
    likes: Option<u64>,
    #[serde(default)]
    private: Option<bool>,
    #[serde(default, rename = "lastModified")]
    last_modified: Option<String>,
    #[serde(default, rename = "createdAt")]
    created_at: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct HFModelDetail {
    id: String,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    downloads: Option<u64>,
    #[serde(default)]
    likes: Option<u64>,
    #[serde(default)]
    private: Option<bool>,
    #[serde(default)]
    tags: Option<Vec<String>>,
    #[serde(default)]
    sha: Option<String>,
    #[serde(default, rename = "lastModified")]
    last_modified: Option<String>,
    #[serde(default, rename = "createdAt")]
    created_at: Option<String>,
    #[serde(default, rename = "cardData")]
    card_data: Option<JsonValue>,
    #[serde(default)]
    config: Option<JsonValue>,
    #[serde(default)]
    siblings: Vec<HFSiblingDetail>,
    #[serde(default, rename = "modelId")]
    model_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HFSiblingDetail {
    rfilename: String,
    #[serde(default)]
    size: Option<u64>,
    #[serde(default, rename = "blobId")]
    blob_id: Option<String>,
    #[serde(default)]
    lfs: Option<HFFileLfs>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HFFileLfs {
    #[serde(default)]
    sha256: Option<String>,
    #[serde(default)]
    size: Option<u64>,
}
type TokenizerExtraction = (Option<Vec<String>>, Option<Vec<f32>>, Option<usize>);
