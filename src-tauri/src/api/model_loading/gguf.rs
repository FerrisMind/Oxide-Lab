use std::fs::File;
use std::path::PathBuf;
use candle::quantized::gguf_file;
use crate::core::device::{device_label, select_device};
use crate::core::state::ModelState;
use crate::core::tokenizer::{mark_special_chat_tokens, tokenizer_from_gguf_metadata, extract_chat_template, find_chat_template_in_metadata};
use crate::models::qwen3::ModelWeights as Qwen3Gguf;
use crate::models::registry::{detect_arch, ArchKind};
use crate::models::common::model::{AnyModel, ModelBackend};

pub fn load_gguf_model(
    guard: &mut ModelState<Box<dyn ModelBackend + Send>>,
    model_path: String,
    context_length: usize,
    device_pref: Option<crate::core::types::DevicePreference>,
) -> Result<(), String> {
    let dev = select_device(device_pref);
    guard.device = dev;
    println!("[load] device selected: {}", device_label(&guard.device));

    let mut file = File::open(&model_path).map_err(|e| e.to_string())?;
    let content = gguf_file::Content::read(&mut file)
        .map_err(|e| format!("{}", e.with_path(PathBuf::from(model_path.clone()))))?;

    let mut tokenizer = tokenizer_from_gguf_metadata(&content.metadata)?;
    mark_special_chat_tokens(&mut tokenizer);
    let chat_tpl = extract_chat_template(&tokenizer).or_else(|| find_chat_template_in_metadata(&content.metadata));
    match &chat_tpl {
        Some(tpl) => {
            let head: String = tpl.chars().take(120).collect();
            println!("[template] detected: len={}, head=<<<{}>>>", tpl.len(), head);
        }
        None => println!("[template] not found in tokenizer.json"),
    }
    let arch = detect_arch(&content.metadata).ok_or_else(|| "Unsupported GGUF architecture".to_string())?;

    if let Some(gg) = content.metadata.get("config.json").and_then(|v| v.to_string().ok()).cloned()
        .or_else(|| content.metadata.get("tokenizer.ggml.config").and_then(|v| v.to_string().ok()).cloned())
        .or_else(|| content.metadata.get("general.config_json").and_then(|v| v.to_string().ok()).cloned())
    {
        guard.model_config_json = Some(gg);
    }

    let model_any = match arch {
        ArchKind::Qwen3 => {
            let m = Qwen3Gguf::from_gguf(content, &mut file, &guard.device, context_length, false)
                .map_err(|e| e.to_string())?;
            AnyModel::from_qwen3(m)
        }
    };

    guard.gguf_model = Some(Box::new(model_any));
    guard.gguf_file = Some(file);
    guard.tokenizer = Some(tokenizer);
    guard.chat_template = chat_tpl;
    let ctx = if context_length == 0 { 1 } else { context_length };
    guard.context_length = ctx;
    guard.model_path = Some(model_path);
    guard.tokenizer_path = None;
    println!("[load] gguf loaded, context_length={}, tokenizer_source=embedded/bpe", guard.context_length);
    
    Ok(())
}