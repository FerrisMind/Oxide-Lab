use std::collections::HashMap;
use candle::quantized::gguf_file;
use tokenizers::{AddedToken, Tokenizer};
use serde::Deserialize;

pub fn find_tokenizer_json_in_metadata(md: &HashMap<String, gguf_file::Value>) -> Option<String> {
    for key in [
        "tokenizer.json",
        "qwen3.tokenizer_json",
        "general.tokenizer_json",
        "tokenizer.ggml",
        "tokenizer",
    ] {
        if let Some(v) = md.get(key) { if let Ok(s) = v.to_string() { return Some(s.clone()); } }
    }
    None
}

fn get_string_array(md: &HashMap<String, gguf_file::Value>, key: &str) -> Option<Vec<String>> {
    match md.get(key) {
        Some(gguf_file::Value::Array(vs)) => {
            let mut out: Vec<String> = Vec::with_capacity(vs.len());
            for v in vs { if let Ok(s) = v.to_string() { out.push(s.clone()); } }
            Some(out)
        }
        _ => None,
    }
}

pub fn try_reconstruct_tokenizer_from_bpe(md: &HashMap<String, gguf_file::Value>) -> Option<String> {
    let vocab_list = get_string_array(md, "tokenizer.ggml.tokens").or_else(|| get_string_array(md, "tokenizer.vocab"))?;
    let merges_list = get_string_array(md, "tokenizer.ggml.merges")
        .or_else(|| get_string_array(md, "tokenizer.ggml.bpe_merges"))
        .or_else(|| get_string_array(md, "tokenizer.merges"))
        .unwrap_or_else(|| Vec::new());
    let mut vocab_obj: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    for (i, tok) in vocab_list.iter().enumerate() { vocab_obj.insert(tok.clone(), serde_json::json!(i as u32)); }
    let json = serde_json::json!({
        "version": "1.0",
        "pre_tokenizer": { "type": "ByteLevel", "add_prefix_space": false, "trim_offsets": true },
        "decoder": { "type": "ByteLevel", "add_prefix_space": false, "trim_offsets": true },
        "model": { "type": "BPE", "vocab": vocab_obj, "merges": merges_list },
    });
    Some(json.to_string())
}

pub fn mark_special_chat_tokens(tokenizer: &mut Tokenizer) {
    let vocab = tokenizer.get_vocab(true);
    let specials = [
        "<|im_start|>", "<|im_end|>", "<|user|>", "<|assistant|>", "<|system|>",
        "<|eot_id|>", "<|endoftext|>", "</s>", "<s>"
    ];
    let mut to_add: Vec<AddedToken> = Vec::new();
    for &tok in specials.iter() {
        if vocab.get(tok).is_some() {
            let mut at = AddedToken::from(tok.to_string(), true);
            at = at.single_word(false).lstrip(false).rstrip(false);
            to_add.push(at);
        }
    }
    if !to_add.is_empty() { tokenizer.add_special_tokens(&to_add); }
}

pub fn tokenizer_from_gguf_metadata(md: &HashMap<String, gguf_file::Value>) -> Result<Tokenizer, String> {
    if let Some(json) = find_tokenizer_json_in_metadata(md) {
        Tokenizer::from_bytes(json.as_bytes()).map_err(|e| e.to_string())
    } else if let Some(json) = try_reconstruct_tokenizer_from_bpe(md) {
        Tokenizer::from_bytes(json.as_bytes()).map_err(|e| e.to_string())
    } else {
        Err("GGUF: embedded tokenizer not found and cannot reconstruct from metadata".into())
    }
}

#[derive(Debug, Deserialize)]
struct TokenizerConfig {
    #[serde(default)]
    added_tokens: Vec<serde_json::Value>,
    #[serde(default)]
    special_tokens: Vec<serde_json::Value>,
    #[serde(default)]
    chat_template: Option<String>,
}

pub fn extract_eos_ids(tokenizer: &Tokenizer) -> Vec<u32> {
    let mut ids = Vec::new();
    // 1) Попробуем вытащить из json-конфига токенизатора все special_tokens с ролью EOS
    if let Ok(json) = tokenizer.to_string(true) {
        if let Ok(cfg) = serde_json::from_str::<TokenizerConfig>(&json) {
            let vocab = tokenizer.get_vocab(true);
            for entry in cfg.special_tokens.into_iter().chain(cfg.added_tokens.into_iter()) {
                if let Some(obj) = entry.as_object() {
                    let content = obj.get("content").and_then(|v| v.as_str());
                    let special = obj.get("special").and_then(|v| v.as_bool()).unwrap_or(false);
                    let role = obj.get("role").and_then(|v| v.as_str());
                    if special {
                        if let Some(tok) = content {
                            if let Some(&id) = vocab.get(tok) {
                                // Грубая эвристика: role=="eos" либо имя/контент содержит признаки EOS
                                if role == Some("eos") || tok.eq_ignore_ascii_case("</s>") || tok.contains("eot") || tok.contains("im_end") || tok.contains("endoftext") {
                                    ids.push(id);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // 2) Резервные эвристики по известным строкам
    let vocab = tokenizer.get_vocab(true);
    for key in ["<|im_end|>", "<|eot_id|>", "<|endoftext|>", "</s>", "<s>"] {
        if let Some(&id) = vocab.get(key) { if !ids.contains(&id) { ids.push(id); } }
    }
    ids
}

pub fn extract_chat_template(tokenizer: &Tokenizer) -> Option<String> {
    let json = tokenizer.to_string(true).ok()?;
    let cfg: TokenizerConfig = serde_json::from_str(&json).ok()?;
    cfg.chat_template
}

pub fn find_chat_template_in_metadata(md: &HashMap<String, gguf_file::Value>) -> Option<String> {
    for key in [
        "tokenizer.chat_template",
        "tokenizer.ggml.chat_template",
        "general.chat_template",
        "chat_template",
    ] {
        if let Some(v) = md.get(key) {
            if let Ok(s) = v.to_string() { return Some(s.clone()); }
        }
    }
    None
}


