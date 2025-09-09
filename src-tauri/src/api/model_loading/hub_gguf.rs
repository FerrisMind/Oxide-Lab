use std::fs::File;
use candle::quantized::gguf_file;
use crate::core::state::ModelState;
use crate::core::tokenizer::{mark_special_chat_tokens, tokenizer_from_gguf_metadata, extract_chat_template, find_chat_template_in_metadata};
use crate::models::registry::{detect_arch, get_model_factory};
use crate::models::common::model::ModelBackend;
use crate::{log_hub, log_load, log_template};

pub fn load_hub_gguf_model(
    guard: &mut ModelState<Box<dyn ModelBackend + Send>>,
    repo_id: String,
    revision: Option<String>,
    filename: String,
    context_length: usize,
    _device_pref: Option<crate::core::types::DevicePreference>,
) -> Result<(), String> {
    let revision = revision.unwrap_or_else(|| "main".to_string());
    log_hub!("loading {} from {}", filename, repo_id);
    let api = hf_hub::api::sync::Api::new().map_err(|e| e.to_string())?;
    let repo = hf_hub::Repo::with_revision(
        repo_id,
        hf_hub::RepoType::Model,
        revision,
    );
    let api = api.repo(repo);

    // Скачиваем GGUF-файл в кэш и открываем
    let model_path = api
        .get(&filename)
        .map_err(|e| format!("hf_hub get {} failed: {}", filename, e))?;
    log_hub!("gguf cached at {}", model_path.display());
    let mut file = File::open(&model_path).map_err(|e| e.to_string())?;
    let content = gguf_file::Content::read(&mut file)
        .map_err(|e| format!("{}", e.with_path(model_path.clone())))?;

    // Токенизатор и шаблон чата из GGUF-метаданных
    let mut tokenizer = tokenizer_from_gguf_metadata(&content.metadata)?;
    mark_special_chat_tokens(&mut tokenizer);
    let chat_tpl = extract_chat_template(&tokenizer).or_else(|| find_chat_template_in_metadata(&content.metadata));
    match &chat_tpl {
        Some(tpl) => {
            let head: String = tpl.chars().take(120).collect();
            log_template!("detected: len={}, head=<<<{}>>>", tpl.len(), head);
        }
        None => log_template!("not found in tokenizer.json"),
    }

    let arch = detect_arch(&content.metadata).ok_or_else(|| "Unsupported GGUF architecture".to_string())?;

    if let Some(gg) = content.metadata.get("config.json").and_then(|v| v.to_string().ok()).cloned()
        .or_else(|| content.metadata.get("tokenizer.ggml.config").and_then(|v| v.to_string().ok()).cloned())
        .or_else(|| content.metadata.get("general.config_json").and_then(|v| v.to_string().ok()).cloned())
    {
        guard.model_config_json = Some(gg);
    }

    // Use the model factory to build the model
    let model_backend = get_model_factory()
        .build_from_gguf(arch, content, &mut file, &guard.device, context_length, false)
        .map_err(|e| format!("Failed to build model: {}", e))?;

    guard.gguf_model = Some(model_backend);
    guard.gguf_file = Some(file);
    guard.tokenizer = Some(tokenizer);
    guard.chat_template = chat_tpl;
    let ctx = if context_length == 0 { 1 } else { context_length };
    guard.context_length = ctx;
    guard.model_path = Some(model_path.to_string_lossy().to_string());
    guard.tokenizer_path = None;
    log_load!("hub gguf loaded, context_length={}, tokenizer_source=embedded/bpe", guard.context_length);
    
    Ok(())
}