use std::fs::File;
use candle::quantized::gguf_file;
use crate::core::state::ModelState;
use crate::core::tokenizer::{mark_special_chat_tokens, tokenizer_from_gguf_metadata, extract_chat_template, find_chat_template_in_metadata};
use crate::models::registry::{detect_arch, get_model_factory};
use crate::models::common::model::ModelBackend;
use crate::{log_hub, log_load, log_template};
use super::emit_load_progress;
use crate::generate::cancel::CANCEL_LOADING;
use std::sync::atomic::Ordering;

pub fn load_hub_gguf_model(
    app: &tauri::AppHandle,
    guard: &mut ModelState<Box<dyn ModelBackend + Send>>,
    repo_id: String,
    revision: Option<String>,
    filename: String,
    context_length: usize,
    _device_pref: Option<crate::core::types::DevicePreference>,
) -> Result<(), String> {
    emit_load_progress(app, "start", 0, Some("Начало загрузки из HF Hub (GGUF)"), false, None);
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
        .map_err(|e| {
            emit_load_progress(app, "hub_get", 10, None, false, Some(&e.to_string()));
            format!("hf_hub get {} failed: {}", filename, e)
        })?;
    log_hub!("gguf cached at {}", model_path.display());
    emit_load_progress(app, "hub_get", 15, Some("Файл получен/закэширован"), false, None);
    let mut file = File::open(&model_path).map_err(|e| e.to_string())?;
    let content = gguf_file::Content::read(&mut file)
        .map_err(|e| {
            let msg = e.with_path(model_path.clone()).to_string();
            emit_load_progress(app, "read_header", 25, None, false, Some(&msg));
            msg
        })?;
    emit_load_progress(app, "read_header", 30, Some("GGUF заголовок прочитан"), false, None);
    if CANCEL_LOADING.load(Ordering::SeqCst) { emit_load_progress(app, "cancel", 32, Some("Отменено"), true, Some("cancelled")); return Err("cancelled".into()); }

    // Определение архитектуры для индикации больше не требуется

    // Токенизатор должен присутствовать в метаданных GGUF; не допускается использование внешнего tokenizer.json
    let mut tokenizer = match tokenizer_from_gguf_metadata(&content.metadata) {
        Ok(tk) => tk,
        Err(e) => return Err(format!("Tokenizer must be embedded in GGUF metadata: {}", e)),
    };
    mark_special_chat_tokens(&mut tokenizer);
    let chat_tpl = extract_chat_template(&tokenizer).or_else(|| find_chat_template_in_metadata(&content.metadata));
    match &chat_tpl {
        Some(tpl) => {
            let head: String = tpl.chars().take(120).collect();
            log_template!("detected: len={}, head=<<<{}>>>", tpl.len(), head);
        }
        None => log_template!("not found in tokenizer.json"),
    }
    emit_load_progress(app, "tokenizer", 40, Some("Инициализирован"), false, None);
    let arch = detect_arch(&content.metadata).ok_or_else(|| {
        let err = "Unsupported GGUF architecture".to_string();
        emit_load_progress(app, "detect_arch", 45, None, false, Some(&err));
        err
    })?;
    emit_load_progress(app, "detect_arch", 50, Some(&format!("{:?}", arch)), false, None);
    if CANCEL_LOADING.load(Ordering::SeqCst) { emit_load_progress(app, "cancel", 52, Some("Отменено"), true, Some("cancelled")); return Err("cancelled".into()); }

    if let Some(gg) = content.metadata.get("config.json").and_then(|v| v.to_string().ok()).cloned()
        .or_else(|| content.metadata.get("tokenizer.ggml.config").and_then(|v| v.to_string().ok()).cloned())
        .or_else(|| content.metadata.get("general.config_json").and_then(|v| v.to_string().ok()).cloned())
    {
        guard.model_config_json = Some(gg.clone());
        // Попытка распарсить и применить конфиг перед созданием модели если доступен
        // Попытка распарсить конфиг, но не используем значение здесь — применим после сборки модели
        if serde_json::from_str::<serde_json::Value>(&gg).is_err() {
            log_hub!("config.json in metadata is present but failed to parse as JSON");
        }
    }

    // Use the model factory to build the model
    emit_load_progress(app, "build_model", 60, None, false, None);
    // Build model
    let mut model_backend = get_model_factory()
        .build_from_gguf(arch, content, &mut file, &guard.device, context_length, false)
        .map_err(|e| {
            emit_load_progress(app, "build_model", 65, None, false, Some(&e));
            format!("Failed to build model: {}", e)
        })?;

    // Если есть JSON-конфигурация в guard.model_config_json — применим её
    if let Some(gg) = guard.model_config_json.as_ref() {
        if let Ok(cfg_val) = serde_json::from_str::<serde_json::Value>(gg) {
            if let Err(e) = model_backend.apply_config(&cfg_val) {
                emit_load_progress(app, "apply_config", 75, None, false, Some(&format!("Model apply_config failed: {}", e)));
            } else {
                emit_load_progress(app, "apply_config", 75, None, false, Some("Model config applied"));
            }
        }
    }
    emit_load_progress(app, "build_model_done", 85, Some("Модель сконструирована"), false, None);
    if CANCEL_LOADING.load(Ordering::SeqCst) { emit_load_progress(app, "cancel", 90, Some("Отменено"), true, Some("cancelled")); return Err("cancelled".into()); }

    guard.gguf_model = Some(model_backend);
    guard.gguf_file = Some(file);
    guard.tokenizer = Some(tokenizer);
    guard.chat_template = chat_tpl;
    let ctx = if context_length == 0 { 1 } else { context_length };
    guard.context_length = ctx;
    guard.model_path = Some(model_path.to_string_lossy().to_string());
    guard.tokenizer_path = None;
    log_load!("hub gguf loaded, context_length={}, tokenizer_source=embedded/bpe", guard.context_length);
    emit_load_progress(app, "finalize", 95, Some("Состояние обновлено"), false, None);
    emit_load_progress(app, "complete", 100, Some("Готово"), true, None);

    Ok(())
}
