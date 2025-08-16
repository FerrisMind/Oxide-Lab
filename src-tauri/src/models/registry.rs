use std::collections::HashMap;

pub enum ArchKind {
    Qwen3,
}

pub fn detect_arch(metadata: &HashMap<String, candle::quantized::gguf_file::Value>) -> Option<ArchKind> {
    // Простая эвристика: ищем имя конфигурации с qwen
    for (_k, v) in metadata.iter() {
        if let Ok(s) = v.to_string() {
            if s.to_lowercase().contains("qwen") {
                return Some(ArchKind::Qwen3);
            }
        }
    }
    None
}


