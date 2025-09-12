use std::fs::File;
use std::path::PathBuf;
use candle::quantized::gguf_file;
use crate::core::device::{device_label, select_device};
use crate::core::state::ModelState;
use crate::core::tokenizer::{mark_special_chat_tokens, tokenizer_from_gguf_metadata, extract_chat_template, find_chat_template_in_metadata};
use crate::models::registry::{detect_arch, get_model_factory};
use crate::models::common::model::ModelBackend;
use crate::{log_load, log_template};
use super::emit_load_progress;
use crate::generate::cancel::CANCEL_LOADING;
use std::sync::atomic::Ordering;

pub fn load_gguf_model(
    app: &tauri::AppHandle,
    guard: &mut ModelState<Box<dyn ModelBackend + Send>>,
    model_path: String,
    context_length: usize,
    device_pref: Option<crate::core::types::DevicePreference>,
) -> Result<(), String> {
    emit_load_progress(app, "start", 0, Some("Начало загрузки GGUF"), false, None);
    let dev = select_device(device_pref);
    guard.device = dev;
    log_load!("device selected: {}", device_label(&guard.device));
    emit_load_progress(app, "device", 5, Some(device_label(&guard.device)), false, None);

    let mut file = File::open(&model_path).map_err(|e| {
        emit_load_progress(app, "open_file", 8, None, false, Some(&e.to_string()));
        e.to_string()
    })?;
    emit_load_progress(app, "open_file", 10, Some("Файл открыт"), false, None);
    if CANCEL_LOADING.load(Ordering::SeqCst) { emit_load_progress(app, "cancel", 12, Some("Отменено"), true, Some("cancelled")); return Err("cancelled".into()); }
    let content = gguf_file::Content::read(&mut file)
        .map_err(|e| {
            let msg = e.with_path(PathBuf::from(model_path.clone())).to_string();
            emit_load_progress(app, "read_header", 20, None, false, Some(&msg));
            msg
        })?;
    emit_load_progress(app, "read_header", 25, Some("GGUF заголовок прочитан"), false, None);
    if CANCEL_LOADING.load(Ordering::SeqCst) { emit_load_progress(app, "cancel", 28, Some("Отменено"), true, Some("cancelled")); return Err("cancelled".into()); }

    // Попытка достать токенизатор строго из GGUF (с расширенными эвристиками внутри tokenizer_from_gguf_metadata)
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
    emit_load_progress(app, "tokenizer", 35, Some("Инициализирован"), false, None);

    let arch = detect_arch(&content.metadata).ok_or_else(|| {
        let err = "Unsupported GGUF architecture".to_string();
        emit_load_progress(app, "detect_arch", 38, None, false, Some(&err));
        err
    })?;
    emit_load_progress(app, "detect_arch", 40, Some(&format!("{:?}", arch)), false, None);
    if CANCEL_LOADING.load(Ordering::SeqCst) { emit_load_progress(app, "cancel", 42, Some("Отменено"), true, Some("cancelled")); return Err("cancelled".into()); }

    if let Some(gg) = content.metadata.get("config.json").and_then(|v| v.to_string().ok()).cloned()
        .or_else(|| content.metadata.get("tokenizer.ggml.config").and_then(|v| v.to_string().ok()).cloned())
        .or_else(|| content.metadata.get("general.config_json").and_then(|v| v.to_string().ok()).cloned())
    {
        guard.model_config_json = Some(gg);
    }

    // Use the model factory to build the model
    emit_load_progress(app, "build_model", 50, None, false, None);
    if CANCEL_LOADING.load(Ordering::SeqCst) { emit_load_progress(app, "cancel", 50, Some("Отменено"), true, Some("cancelled")); return Err("cancelled".into()); }
    let model_backend = get_model_factory()
        .build_from_gguf(arch, content, &mut file, &guard.device, context_length, false)
        .map_err(|e| {
            emit_load_progress(app, "build_model", 60, None, false, Some(&e));
            format!("Failed to build model: {}", e)
        })?;
    emit_load_progress(app, "build_model_done", 85, Some("Модель сконструирована"), false, None);

    if CANCEL_LOADING.load(Ordering::SeqCst) { emit_load_progress(app, "cancel", 90, Some("Отменено"), true, Some("cancelled")); return Err("cancelled".into()); }
    guard.gguf_model = Some(model_backend);
    guard.gguf_file = Some(file);
    guard.tokenizer = Some(tokenizer);
    guard.chat_template = chat_tpl;
    let ctx = if context_length == 0 { 1 } else { context_length };
    guard.context_length = ctx;
    guard.model_path = Some(model_path);
    guard.tokenizer_path = None;
    log_load!("gguf loaded, context_length={}, tokenizer_source=embedded/bpe", guard.context_length);
    emit_load_progress(app, "finalize", 95, Some("Состояние обновлено"), false, None);
    emit_load_progress(app, "complete", 100, Some("Готово"), true, None);
    
    Ok(())
}
