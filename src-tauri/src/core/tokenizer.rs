use std::collections::HashMap;
use candle::quantized::gguf_file;
use tokenizers::{AddedToken, Tokenizer};
use serde::Deserialize;

pub fn find_tokenizer_json_in_metadata(md: &HashMap<String, gguf_file::Value>) -> Option<String> {
    // 1) Прямые известные ключи
    for key in [
        "tokenizer.json",
        "general.tokenizer_json",
        "qwen3.tokenizer_json",
        "llama.tokenizer_json",
        "gemma.tokenizer_json",
        "tokenizer.ggml.json",
        "tokenizer_json",
        "tokenizer",
    ] {
        if let Some(v) = md.get(key) {
            if let Ok(s) = v.to_string() { return Some(s.clone()); }
        }
    }
    // 2) Эвристика: найти любой строковый JSON, который успешно парсится как tokenizers JSON
    for (_k, v) in md.iter() {
        if let Ok(s) = v.to_string() {
            let st = s.trim();
            if st.starts_with('{') && st.ends_with('}')
                && tokenizers::Tokenizer::from_bytes(st.as_bytes()).is_ok()
            {
                return Some(s.clone());
            }
        }
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
    // Reconstruct only if we have both tokens AND merges. Without merges it's very likely
    // not a GPT-2 style BPE (e.g. SentencePiece/Unigram), and producing a ByteLevel/BPE
    // tokenizer will yield completely wrong ids and gibberish output.
    let vocab_list = get_string_array(md, "tokenizer.ggml.tokens").or_else(|| get_string_array(md, "tokenizer.vocab"))?;
    let merges_list_opt = get_string_array(md, "tokenizer.ggml.merges")
        .or_else(|| get_string_array(md, "tokenizer.ggml.bpe_merges"))
        .or_else(|| get_string_array(md, "tokenizer.merges"));
    let merges_list = match merges_list_opt {
        Some(m) if !m.is_empty() => m,
        _ => return None, // do not attempt incorrect BPE reconstruction
    };

    let mut vocab_obj: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    for (i, tok) in vocab_list.iter().enumerate() { vocab_obj.insert(tok.clone(), serde_json::json!(i as u32)); }
    let json = serde_json::json!({
        "version": "1.0",
        // ByteLevel with add_prefix_space=true better matches GPT-2 style BPE vocabs
        "pre_tokenizer": { "type": "ByteLevel", "add_prefix_space": true, "trim_offsets": true },
        "decoder": { "type": "ByteLevel", "add_prefix_space": true, "trim_offsets": true },
        "model": { "type": "BPE", "vocab": vocab_obj, "merges": merges_list },
    });
    Some(json.to_string())
}

pub fn mark_special_chat_tokens(tokenizer: &mut Tokenizer) {
    let vocab = tokenizer.get_vocab(true);
    let specials = [
        "<|im_start|>", "<|im_end|>", "<|user|>", "<|assistant|>", "<|system|>",
        "<|eot_id|>", "<|endoftext|>", "</s>", "<s>",
        // Gemma/Gemma2/Gemma3 style
        "<start_of_turn>", "<end_of_turn>", "<eos>"
    ];
    let mut to_add: Vec<AddedToken> = Vec::new();
    for &tok in specials.iter() {
        if vocab.contains_key(tok) {
            let mut at = AddedToken::from(tok.to_string(), true);
            at = at.single_word(false).lstrip(false).rstrip(false);
            to_add.push(at);
        }
    }
    if !to_add.is_empty() { tokenizer.add_special_tokens(&to_add); }
}

pub fn tokenizer_from_gguf_metadata(md: &HashMap<String, gguf_file::Value>) -> Result<Tokenizer, String> {
    if let Some(json) = find_tokenizer_json_in_metadata(md) {
        return Tokenizer::from_bytes(json.as_bytes()).map_err(|e| e.to_string());
    }
    // Если не нашли полноценный JSON, пробуем реконструировать BPE (только если поля BPE действительно есть)
    if let Some(json) = try_reconstruct_tokenizer_from_bpe(md) {
        return Tokenizer::from_bytes(json.as_bytes()).map_err(|e| e.to_string());
    }
    Err("GGUF: embedded tokenizer.json не найден; реконструкция невозможна".into())
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
                                if role == Some("eos")
                                    || tok.eq_ignore_ascii_case("</s>")
                                    || tok.eq_ignore_ascii_case("<eos>")
                                    || tok.contains("end_of_turn")
                                    || tok.contains("eot")
                                    || tok.contains("im_end")
                                    || tok.contains("endoftext")
                                {
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
    for key in [
        "<|im_end|>", "<|eot_id|>", "<|endoftext|>", "</s>",
        "<end_of_turn>", "<eos>",
    ] {
        if let Some(&id) = vocab.get(key) { if !ids.contains(&id) { ids.push(id); } }
    }
    ids
}

/// Try to extract BOS token string from tokenizer config or known specials
pub fn extract_bos_token_str(tokenizer: &Tokenizer) -> Option<String> {
    // Parse JSON to search for role-specific tokens
    if let Ok(json) = tokenizer.to_string(true) {
        if let Ok(cfg) = serde_json::from_str::<TokenizerConfig>(&json) {
            let vocab = tokenizer.get_vocab(true);
            for entry in cfg.special_tokens.iter().chain(cfg.added_tokens.iter()) {
                if let Some(obj) = entry.as_object() {
                    let content = obj.get("content").and_then(|v| v.as_str());
                    let special = obj.get("special").and_then(|v| v.as_bool()).unwrap_or(false);
                    let role = obj.get("role").and_then(|v| v.as_str());
                    if special && role == Some("bos") {
                        if let Some(tok) = content {
                            if vocab.contains_key(tok) {
                                return Some(tok.to_string());
                            }
                        }
                    }
                }
            }
            // Heuristics: common BOS strings
            for key in ["<s>", "<bos>", "<BOS>", "<|begin_of_text|>"] {
                if tokenizer.get_vocab(true).contains_key(key) { return Some(key.to_string()); }
            }
        }
    }
    None
}

pub fn extract_chat_template(tokenizer: &Tokenizer) -> Option<String> {
    let json = tokenizer.to_string(true).ok()?;
    let cfg: TokenizerConfig = serde_json::from_str(&json).ok()?;
    cfg.chat_template
}

pub fn find_chat_template_in_metadata(md: &HashMap<String, gguf_file::Value>) -> Option<String> {
    // 1) Прямые известные ключи
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
    // 2) Эвристика: ищем большие строковые значения, содержащие конструкции Jinja
    let mut best: Option<String> = None;
    for (_k, v) in md.iter() {
        if let Ok(s) = v.to_string() {
            if (s.contains("add_generation_prompt") || s.contains("messages") && s.contains("role"))
                && best.as_ref().is_none_or(|cur| s.len() > cur.len())
            {
                best = Some(s.clone());
            }
        }
    }
    best
}


