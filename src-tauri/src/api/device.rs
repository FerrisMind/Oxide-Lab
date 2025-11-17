use crate::core::device::device_label;
use crate::core::state::ModelState;
use crate::core::tokenizer::{
    extract_chat_template, find_chat_template_in_metadata, mark_special_chat_tokens,
    tokenizer_from_gguf_metadata,
};
use crate::models::common::model::ModelBackend;
use crate::models::registry::detect_arch;
use crate::models::registry::get_model_factory;
use crate::{log_device, log_device_error, log_load};
use candle::quantized::gguf_file;
use candle::utils::{cuda_is_available, metal_is_available};
use std::fs::File;
use std::path::PathBuf;

pub fn set_device(
    guard: &mut ModelState<Box<dyn ModelBackend + Send>>,
    pref: crate::core::types::DevicePreference,
) -> Result<(), String> {
    // Явно проверяем запрос CUDA и возвращаем ошибку, если инициализация не удалась
    match pref {
        crate::core::types::DevicePreference::Cuda { index } => {
            match candle::Device::new_cuda(index) {
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
        crate::core::types::DevicePreference::Metal => match candle::Device::new_metal(0) {
            Ok(dev) => {
                guard.device = dev;
            }
            Err(e) => {
                return Err(format!("Metal init failed: {}", e));
            }
        },
        crate::core::types::DevicePreference::Auto => {
            // Авто-выбор с предпочтением CUDA → Metal → CPU
            if cuda_is_available() {
                match candle::Device::new_cuda(0) {
                    Ok(dev) => {
                        guard.device = dev;
                        log_device!("auto -> CUDA");
                    }
                    Err(e) => {
                        log_device_error!("CUDA init failed: {}, fallback to CPU", e);
                        guard.device = candle::Device::Cpu;
                    }
                }
            } else if metal_is_available() {
                match candle::Device::new_metal(0) {
                    Ok(dev) => {
                        guard.device = dev;
                        log_device!("auto -> Metal");
                    }
                    Err(e) => {
                        log_device_error!("Metal init failed: {}, fallback to CPU", e);
                        guard.device = candle::Device::Cpu;
                    }
                }
            } else {
                guard.device = candle::Device::Cpu;
                log_device!("auto -> CPU");
            }
        }
    }
    let label = device_label(&guard.device);
    log_device!("switched -> {}", label);
    {
        let kcfg = crate::core::precision::GpuKernelConfig::from_policy(&guard.precision_policy);
        kcfg.apply_for_device(&guard.device);
    }
    log_device!(
        "hw caps: avx={}, neon={}, simd128={}, f16c={}",
        candle::utils::with_avx(),
        candle::utils::with_neon(),
        candle::utils::with_simd128(),
        candle::utils::with_f16c()
    );
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
        let chat_tpl = extract_chat_template(&tokenizer)
            .or_else(|| find_chat_template_in_metadata(&content.metadata));

        // Архитектура
        let arch = detect_arch(&content.metadata)
            .ok_or_else(|| "Unsupported GGUF architecture".to_string())?;

        // Универсальное создание модели через фабрику (под выбранное устройство)
        let model_backend = get_model_factory()
            .build_from_gguf(arch, content, &mut file, &guard.device, ctx_len, false)
            .map_err(|e| format!("Failed to rebuild model for new device: {}", e))?;

        guard.gguf_model = Some(model_backend);
        guard.gguf_file = Some(file);
        guard.tokenizer = Some(tokenizer);
        guard.chat_template = chat_tpl;
        log_load!("model reloaded for {}", label);
    }
    Ok(())
}
