use std::fs::File;
use std::path::PathBuf;
use candle::quantized::gguf_file;
use crate::core::device::device_label;
use crate::core::state::ModelState;
use crate::core::tokenizer::{mark_special_chat_tokens, tokenizer_from_gguf_metadata, extract_chat_template, find_chat_template_in_metadata};
use crate::models::qwen3::ModelWeights as Qwen3Gguf;
use crate::models::registry::{detect_arch, ArchKind};
use crate::models::common::model::{AnyModel, ModelBackend};

pub fn set_device(
    guard: &mut ModelState<Box<dyn ModelBackend + Send>>,
    pref: crate::core::types::DevicePreference,
) -> Result<(), String> {
    // Явно проверяем запрос CUDA и возвращаем ошибку, если инициализация не удалась
    match pref {
        crate::core::types::DevicePreference::Cuda { index } => {
            match candle::Device::cuda_if_available(index) {
                Ok(dev) => {
                    guard.device = dev;
                }
                Err(e) => {
                    return Err(format!("CUDA init failed (index={}): {}", index, e));
                }
            }
        }
        crate::core::types::DevicePreference::Cpu => {
            guard.device = candle::Device::Cpu;
        }
        crate::core::types::DevicePreference::Auto => {
            guard.device = candle::Device::Cpu;
        }
        crate::core::types::DevicePreference::Metal => {
            guard.device = candle::Device::Cpu;
        }
    }
    let label = device_label(&guard.device);
    println!("[device] switched -> {}", label);
    // Если модель загружена — перезагрузим её под выбранное устройство
    if guard.gguf_model.is_some() {
        // Перечитываем с диска по сохранённому пути
        let model_path = match guard.model_path.clone() {
            Some(p) => p,
            None => return Ok(()),
        };
        let ctx_len = guard.context_length.max(1);
        let mut file = File::open(&model_path).map_err(|e| e.to_string())?;
        let content = gguf_file::Content::read(&mut file)
            .map_err(|e| format!("{}", e.with_path(PathBuf::from(model_path.clone()))))?;

        // Токенизатор и шаблон чата
        let mut tokenizer = tokenizer_from_gguf_metadata(&content.metadata)?;
        mark_special_chat_tokens(&mut tokenizer);
        let chat_tpl = extract_chat_template(&tokenizer).or_else(|| find_chat_template_in_metadata(&content.metadata));

        // Архитектура
        let arch = detect_arch(&content.metadata).ok_or_else(|| "Unsupported GGUF architecture".to_string())?;

        // Создание модели на новом устройстве
        let model = match arch {
            ArchKind::Qwen3 => Qwen3Gguf::from_gguf(content, &mut file, &guard.device, ctx_len, false)
                .map_err(|e| e.to_string())?,
        };

        // Wrap concrete model into AnyModel and box as trait object
        let any = AnyModel::from_qwen3(model);
        guard.gguf_model = Some(Box::new(any));
        guard.gguf_file = Some(file);
        guard.tokenizer = Some(tokenizer);
        guard.chat_template = chat_tpl;
        println!("[device] model reloaded for {}", label);
    }
    Ok(())
}