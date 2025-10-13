use super::emit_load_progress;
use crate::core::device::{device_label, select_device};
use crate::core::performance::ModelLoadTracker;
use crate::core::state::ModelState;
use crate::core::tokenizer::{
    extract_chat_template, find_chat_template_in_metadata, mark_special_chat_tokens,
    tokenizer_from_gguf_metadata,
};
use crate::generate::cancel::CANCEL_LOADING;
use crate::models::common::model::ModelBackend;
use crate::models::registry::{detect_arch, get_model_factory};
use crate::{log_load, log_template};
use candle::quantized::gguf_file;
use std::collections::HashSet;
use std::fs::File;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use tauri::Emitter;

pub fn load_gguf_model(
    app: &tauri::AppHandle,
    guard: &mut ModelState<Box<dyn ModelBackend + Send>>,
    model_path: String,
    context_length: usize,
    device_pref: Option<crate::core::types::DevicePreference>,
) -> Result<(), String> {
    // Создаём трекер загрузки модели
    let tracker_result = tokio::runtime::Runtime::new()
        .map_err(|e| e.to_string())?
        .block_on(async { ModelLoadTracker::new(guard.performance_monitor.clone()).await });
    let mut tracker = tracker_result;

    emit_load_progress(app, "start", 0, Some("Начало загрузки GGUF"), false, None);
    tracker.start_stage("device_selection");

    let dev = select_device(device_pref);
    guard.device = dev;
    log_load!("device selected: {}", device_label(&guard.device));
    emit_load_progress(
        app,
        "device",
        5,
        Some(device_label(&guard.device)),
        false,
        None,
    );

    let mut file = File::open(&model_path).map_err(|e| {
        emit_load_progress(app, "open_file", 8, None, false, Some(&e.to_string()));
        e.to_string()
    })?;
    tracker.start_stage("file_opening");
    emit_load_progress(app, "open_file", 10, Some("Файл открыт"), false, None);
    if CANCEL_LOADING.load(Ordering::SeqCst) {
        emit_load_progress(app, "cancel", 12, Some("Отменено"), true, Some("cancelled"));
        return Err("cancelled".into());
    }

    tracker.start_stage("read_header");
    let content = gguf_file::Content::read(&mut file).map_err(|e| {
        let error_msg = e.with_path(PathBuf::from(model_path.clone())).to_string();

        // Улучшаем сообщение об ошибке для пользователя
        let enhanced_msg = if error_msg.contains("unknown dtype") {
            format!("{} - This GGUF file contains quantization types that are not supported by the current version of Candle. Try updating Candle to the latest version or use a model with different quantization (Q4_K, Q8_0, etc.)", error_msg)
        } else {
            error_msg
        };

        emit_load_progress(app, "read_header", 20, None, false, Some(&enhanced_msg));
        enhanced_msg
    })?;
    emit_load_progress(
        app,
        "read_header",
        25,
        Some("GGUF заголовок прочитан"),
        false,
        None,
    );
    if CANCEL_LOADING.load(Ordering::SeqCst) {
        emit_load_progress(app, "cancel", 28, Some("Отменено"), true, Some("cancelled"));
        return Err("cancelled".into());
    }

    tracker.start_stage("tokenizer_init");
    // Токенизатор обязан быть в метаданных GGUF. Никакого внешнего tokenizer.json не допускается.
    let (mut tokenizer, tokenizer_source): (tokenizers::Tokenizer, &'static str) =
        match tokenizer_from_gguf_metadata(&content.metadata) {
            Ok(tk) => (tk, "embedded"),
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
    emit_load_progress(app, "tokenizer", 35, Some("Инициализирован"), false, None);

    // Модальности теперь определяются строго по архитектуре
    let arch = detect_arch(&content.metadata).ok_or_else(|| {
        let err = "Unsupported GGUF architecture".to_string();
        emit_load_progress(app, "detect_arch", 38, None, false, Some(&err));
        err
    })?;

    // Проверяем наличие неподдерживаемых типов данных в тензорах
    if let Err(dtype_error) = check_supported_dtypes(&content) {
        let warning_msg = format!("Warning: {}", dtype_error);
        emit_load_progress(app, "dtype_check", 35, Some(&warning_msg), false, None);

        // Покажем пользователю предупреждение, но продолжим загрузку
        log::warn!(
            "GGUF file contains potentially unsupported data types: {}",
            dtype_error
        );
    }
    emit_load_progress(
        app,
        "detect_arch",
        40,
        Some(&format!("{:?}", arch)),
        false,
        None,
    );
    // Модальная индикация удалена: единая обработка вложений реализуется на уровне проекта.
    // Persist detected architecture in state
    guard.arch = Some(arch.clone());
    if CANCEL_LOADING.load(Ordering::SeqCst) {
        emit_load_progress(app, "cancel", 42, Some("Отменено"), true, Some("cancelled"));
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
        guard.model_config_json = Some(gg);
    }

    tracker.start_stage("model_building");
    // Use the model factory to build the model
    emit_load_progress(app, "build_model", 50, None, false, None);
    if CANCEL_LOADING.load(Ordering::SeqCst) {
        emit_load_progress(app, "cancel", 50, Some("Отменено"), true, Some("cancelled"));
        return Err("cancelled".into());
    }
    // Если в метаданных присутствует конфигурация модели, попробуем распарсить и применить её
    let config_json_opt = content
        .metadata
        .get("config.json")
        .and_then(|v| v.to_string().ok())
        .or_else(|| {
            content
                .metadata
                .get("general.config_json")
                .and_then(|v| v.to_string().ok())
        });
    let config_value: Option<serde_json::Value> = match config_json_opt {
        Some(s) => match serde_json::from_str(s) {
            Ok(v) => Some(v),
            Err(e) => {
                emit_load_progress(
                    app,
                    "build_model",
                    55,
                    None,
                    false,
                    Some(&format!(
                        "Failed to parse config.json from GGUF metadata: {}",
                        e
                    )),
                );
                None
            }
        },
        None => None,
    };

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
            emit_load_progress(app, "build_model", 60, None, false, Some(&e));
            format!("Failed to build model: {}", e)
        })?;

    // Если модель предоставляет возможность применения конфигурации — применим
    if let Some(cfg) = config_value.as_ref() {
        if let Err(e) = model_backend.apply_config(cfg) {
            emit_load_progress(
                app,
                "apply_config",
                70,
                None,
                false,
                Some(&format!("Model apply_config failed: {}", e)),
            );
        } else {
            emit_load_progress(
                app,
                "apply_config",
                70,
                None,
                false,
                Some("Model config applied"),
            );
        }
    }
    emit_load_progress(
        app,
        "build_model_done",
        85,
        Some("Модель сконструирована"),
        false,
        None,
    );

    if CANCEL_LOADING.load(Ordering::SeqCst) {
        emit_load_progress(app, "cancel", 90, Some("Отменено"), true, Some("cancelled"));
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
    guard.model_path = Some(model_path);
    guard.tokenizer_path = None;
    log_load!(
        "gguf loaded, context_length={}, tokenizer_source={}",
        guard.context_length,
        tokenizer_source
    );
    emit_load_progress(
        app,
        "finalize",
        95,
        Some("Состояние обновлено"),
        false,
        None,
    );

    // Финализируем метрики загрузки
    let model_size_mb = std::fs::metadata(guard.model_path.as_ref().unwrap())
        .map(|m| m.len() as f64 / (1024.0 * 1024.0))
        .unwrap_or(0.0);

    let metrics = tokio::runtime::Runtime::new()
        .map_err(|e| e.to_string())?
        .block_on(async { tracker.finish(model_size_mb).await });

    log_load!(
        "Метрики загрузки: total_time={}ms, memory_delta={:.2}MB, stages={:?}",
        metrics.total_duration_ms,
        metrics.memory_delta_mb,
        metrics
            .stages
            .iter()
            .map(|s| format!("{}:{}ms", s.name, s.duration_ms))
            .collect::<Vec<_>>()
    );

    // Отправляем метрики на фронтенд
    let _ = app.emit("model_load_metrics", &metrics);

    emit_load_progress(app, "complete", 100, Some("Готово"), true, None);

    Ok(())
}

/// Проверяет наличие поддерживаемых типов данных в GGUF файле
/// Возвращает ошибку, если найдены неподдерживаемые типы данных
fn check_supported_dtypes(content: &gguf_file::Content) -> Result<(), String> {
    // Известные поддерживаемые типы данных в текущей версии Candle
    let supported_dtypes: HashSet<u32> = [
        0, 1, 2, 3, 6, 7, 8,
        9, // Старые типы (F32, F16, Q4_0, Q4_1, Q5_0, Q5_1, Q8_0, Q8_1)
        10, 11, 12, 13, 14, 15, // Новые K-типы (Q2_K, Q3_K, Q4_K, Q5_K, Q6_K, Q8_K)
        16, 17, 18, 19, 20, 21, 22, 23,
        29, // IQ типы (могут быть поддержаны в новой версии)
        24, 25, 26, 27, 28, // Целые типы (I8, I16, I32, I64, F64)
    ]
    .into_iter()
    .collect();

    // Известные неподдерживаемые типы данных (если они еще не поддержаны)
    let unsupported_dtypes: HashSet<u32> = [
        // Пока оставляем пустым - в новой версии могут поддерживаться все известные типы
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
                "Found unsupported IQ quantization types: {:?}. ",
                found_unsupported
            ));
            error_msg
                .push_str("These require a newer version of Candle with IQ quantization support. ");
        }

        if !found_unknown.is_empty() {
            error_msg.push_str(&format!("Found unknown data types: {:?}. ", found_unknown));
            error_msg.push_str("These may be from a newer GGUF format version. ");
        }

        error_msg.push_str(
            "Consider updating Candle to the latest version or using a different model file.",
        );

        return Err(error_msg);
    }

    Ok(())
}
