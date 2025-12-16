use super::{LoadDebugCtx, emit_load_progress_debug};
use crate::core::state::ModelState;
use crate::core::tokenizer::{
    extract_chat_template, find_chat_template_in_metadata, mark_special_chat_tokens,
    tokenizer_from_gguf_metadata,
};
use crate::generate::cancel::CANCEL_LOADING;
use crate::models::common::model::ModelBackend;
use crate::models::registry::{detect_arch, get_model_factory};
use crate::{log_hub, log_load, log_template};
use candle::quantized::gguf_file;
use std::collections::HashSet;
use std::fs::File;
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
    let dbg = LoadDebugCtx::new();
    emit_load_progress_debug(
        &dbg,
        app,
        "start",
        0,
        Some("Начало загрузки из HF Hub (GGUF)"),
        false,
        None,
    );
    let revision = revision.unwrap_or_else(|| "main".to_string());
    log_hub!("loading {} from {}", filename, repo_id);
    let api = hf_hub::api::sync::Api::new().map_err(|e| e.to_string())?;
    let repo = hf_hub::Repo::with_revision(repo_id, hf_hub::RepoType::Model, revision);
    let api = api.repo(repo);

    // Скачиваем GGUF-файл в кэш и открываем
    let model_path = api.get(&filename).map_err(|e| {
        emit_load_progress_debug(&dbg, app, "hub_get", 10, None, false, Some(&e.to_string()));
        format!("hf_hub get {} failed: {}", filename, e)
    })?;
    log_hub!("gguf cached at {}", model_path.display());
    emit_load_progress_debug(
        &dbg,
        app,
        "hub_get",
        15,
        Some("Файл получен/закэширован"),
        false,
        None,
    );
    let mut file = File::open(&model_path).map_err(|e| e.to_string())?;
    let content = gguf_file::Content::read(&mut file).map_err(|e| {
        let msg = e.with_path(model_path.clone()).to_string();
        emit_load_progress_debug(&dbg, app, "read_header", 25, None, false, Some(&msg));
        msg
    })?;
    emit_load_progress_debug(
        &dbg,
        app,
        "read_header",
        30,
        Some("GGUF заголовок прочитан"),
        false,
        None,
    );
    if CANCEL_LOADING.load(Ordering::SeqCst) {
        emit_load_progress_debug(
            &dbg,
            app,
            "cancel",
            32,
            Some("Отменено"),
            true,
            Some("cancelled"),
        );
        return Err("cancelled".into());
    }

    // Определение архитектуры для индикации больше не требуется

    // Токенизатор должен присутствовать в метаданных GGUF; не допускается использование внешнего tokenizer.json
    let mut tokenizer = match tokenizer_from_gguf_metadata(&content.metadata) {
        Ok(tk) => tk,
        Err(e) => {
            return Err(format!(
                "Tokenizer must be embedded in GGUF metadata: {}",
                e
            ));
        }
    };
    mark_special_chat_tokens(&mut tokenizer);
    let chat_tpl = extract_chat_template(&tokenizer)
        .or_else(|| find_chat_template_in_metadata(&content.metadata));
    match &chat_tpl {
        Some(tpl) => {
            let head: String = tpl.chars().take(120).collect();
            log_template!("detected: len={}, head=<<<{}>>>", tpl.len(), head);
        }
        None => log_template!("not found in tokenizer.json"),
    }
    emit_load_progress_debug(
        &dbg,
        app,
        "tokenizer",
        40,
        Some("Инициализирован"),
        false,
        None,
    );
    let arch = detect_arch(&content.metadata).ok_or_else(|| {
        let err = "Unsupported GGUF architecture".to_string();
        emit_load_progress_debug(&dbg, app, "detect_arch", 45, None, false, Some(&err));
        err
    })?;

    // Проверяем наличие неподдерживаемых типов данных в тензорах
    check_supported_dtypes(&content).map_err(|dtype_error| {
        let error_msg = format!("Unsupported quantization types: {}", dtype_error);
        emit_load_progress_debug(&dbg, app, "dtype_check", 45, None, false, Some(&error_msg));
        log::error!("Model loading blocked: {}", dtype_error);
        error_msg
    })?;

    emit_load_progress_debug(
        &dbg,
        app,
        "detect_arch",
        50,
        Some(&format!("{:?}", arch)),
        false,
        None,
    );
    if CANCEL_LOADING.load(Ordering::SeqCst) {
        emit_load_progress_debug(
            &dbg,
            app,
            "cancel",
            52,
            Some("Отменено"),
            true,
            Some("cancelled"),
        );
        return Err("cancelled".into());
    }

    if let Some(gg) = content
        .metadata
        .get("config.json")
        .and_then(|v| v.to_string().ok())
        .cloned()
        .or_else(|| {
            content
                .metadata
                .get("tokenizer.ggml.config")
                .and_then(|v| v.to_string().ok())
                .cloned()
        })
        .or_else(|| {
            content
                .metadata
                .get("general.config_json")
                .and_then(|v| v.to_string().ok())
                .cloned()
        })
    {
        guard.model_config_json = Some(gg.clone());
        // Попытка распарсить и применить конфиг перед созданием модели если доступен
        // Попытка распарсить конфиг, но не используем значение здесь — применим после сборки модели
        if serde_json::from_str::<serde_json::Value>(&gg).is_err() {
            log_hub!("config.json in metadata is present but failed to parse as JSON");
        }
    }

    // Use the model factory to build the model
    emit_load_progress_debug(&dbg, app, "build_model", 60, None, false, None);
    // Build model
    let mut model_backend = get_model_factory()
        .build_from_gguf(
            arch,
            content,
            &mut file,
            &guard.device,
            context_length,
            false,
        )
        .map_err(|e| {
            emit_load_progress_debug(&dbg, app, "build_model", 65, None, false, Some(&e));
            format!("Failed to build model: {}", e)
        })?;

    // Если есть JSON-конфигурация в guard.model_config_json — применим её
    if let Some(gg) = guard.model_config_json.as_ref()
        && let Ok(cfg_val) = serde_json::from_str::<serde_json::Value>(gg)
    {
        if let Err(e) = model_backend.apply_config(&cfg_val) {
            emit_load_progress_debug(
                &dbg,
                app,
                "apply_config",
                75,
                None,
                false,
                Some(&format!("Model apply_config failed: {}", e)),
            );
        } else {
            emit_load_progress_debug(
                &dbg,
                app,
                "apply_config",
                75,
                None,
                false,
                Some("Model config applied"),
            );
        }
    }
    emit_load_progress_debug(
        &dbg,
        app,
        "build_model_done",
        85,
        Some("Модель сконструирована"),
        false,
        None,
    );
    if CANCEL_LOADING.load(Ordering::SeqCst) {
        emit_load_progress_debug(
            &dbg,
            app,
            "cancel",
            90,
            Some("Отменено"),
            true,
            Some("cancelled"),
        );
        return Err("cancelled".into());
    }

    guard.gguf_model = Some(model_backend);
    guard.gguf_file = Some(file);
    guard.tokenizer = Some(tokenizer);
    guard.chat_template = chat_tpl;
    let ctx = if context_length == 0 {
        1
    } else {
        context_length
    };
    guard.context_length = ctx;
    guard.model_path = Some(model_path.to_string_lossy().to_string());
    guard.tokenizer_path = None;
    log_load!(
        "hub gguf loaded, context_length={}, tokenizer_source=embedded/bpe",
        guard.context_length
    );
    emit_load_progress_debug(
        &dbg,
        app,
        "finalize",
        95,
        Some("Состояние обновлено"),
        false,
        None,
    );
    emit_load_progress_debug(&dbg, app, "complete", 100, Some("Готово"), true, None);

    Ok(())
}

/// Проверяет наличие поддерживаемых типов данных в GGUF файле
/// Возвращает ошибку, если найдены неподдерживаемые типы данных
fn check_supported_dtypes(content: &gguf_file::Content) -> Result<(), String> {
    // Поддерживаемые типы данных в текущей версии Candle (на основе candle-core/src/quantized/mod.rs)
    let supported_dtypes: HashSet<u32> = [
        0,  // F32
        1,  // F16
        2,  // Q4_0
        3,  // Q4_1
        6,  // Q5_0
        7,  // Q5_1
        8,  // Q8_0
        9,  // Q8_1
        10, // Q2K
        11, // Q3K
        12, // Q4K
        13, // Q5K
        14, // Q6K
        15, // Q8K
        30, // BF16
    ]
    .into_iter()
    .collect();

    // Известные неподдерживаемые типы данных (на основе исходников GGML)
    let unsupported_dtypes: HashSet<u32> = [
        16, 17, 18, 19, 20, 21, 22,
        23, // IQ типы: IQ2_XXS, IQ2_XS, IQ3_XXS, IQ3_S, IQ2_S, IQ4_NL, IQ4_XS, IQ3_M
        29, // IQ1_M
        24, 25, 26, 27,
        28, // Целочисленные типы: I8, I16, I32, I64, F64 (не используются в моделях)
    ]
    .into_iter()
    .collect();

    let mut found_unsupported = Vec::new();
    let mut found_unknown = Vec::new();

    // Проверяем каждый тензор в файле
    for tensor_info in content.tensor_infos.values() {
        let dtype = tensor_info.ggml_dtype as u32;

        if unsupported_dtypes.contains(&dtype) {
            found_unsupported.push(dtype);
        } else if !supported_dtypes.contains(&dtype) {
            found_unknown.push(dtype);
        }
    }

    if !found_unsupported.is_empty() || !found_unknown.is_empty() {
        let mut error_msg = String::new();

        if !found_unsupported.is_empty() {
            error_msg.push_str(&format!(
                "Found unsupported quantization types: {:?}. ",
                found_unsupported
            ));
            error_msg.push_str("These IQ quantization types require a newer version of Candle. ");
            error_msg.push_str(
                "Currently supported: F32, F16, BF16, Q4_0, Q4_1, Q5_0, Q5_1, Q8_0, Q8_1, Q2K-Q8K. "
            );
        }

        if !found_unknown.is_empty() {
            error_msg.push_str(&format!("Found unknown data types: {:?}. ", found_unknown));
            error_msg.push_str("These may be from a newer GGUF format version. ");
        }

        error_msg.push_str(
            "Model loading is blocked. Please use a model with supported quantizations (Q4_K_M, Q5_K_M, Q8_0, etc.).",
        );

        return Err(error_msg);
    }

    Ok(())
}
