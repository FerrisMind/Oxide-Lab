//! Единая обвязка для вложений (скрепка): маршрутизация и подготовка для разных архитектур.
//!
//! Идея: мультимодальность реализуем на уровне проекта. Один вход принимает любые файлы,
//! а дальше маршрутизатор решает, как их подать конкретной модели в зависимости от её
//! возможностей (ModalitySupport) и настроек.

use base64::Engine as _;
use candle::Device;

use crate::core::types::{Attachment, ChatMessage};
use crate::models::registry::ArchKind;

/// Результат обогащения: обновлённые сообщения или prompt (ровно одно из двух).
pub struct AugmentedInput {
    pub messages: Option<Vec<ChatMessage>>,
    pub prompt: Option<String>,
}

/// Главная точка входа: обогатить сообщения/промпт вложениями согласно архитектуре.
/// - Для Gemma3: изображения → `<image>`; текстовые файлы → добавляем содержимое.
/// - Для Gemma: игнорируем изображения; добавляем только текстовые файлы.
/// - Для других архитектур: по умолчанию — оставляем без изменений (можно расширить).
pub fn augment_with_attachments(
    arch: Option<ArchKind>,
    _device: &Device,
    messages: Option<Vec<ChatMessage>>,
    prompt: Option<String>,
    attachments: Vec<Attachment>,
) -> Result<AugmentedInput, String> {
    if attachments.is_empty() { return Ok(AugmentedInput { messages, prompt }); }

    let arch = arch.unwrap_or(ArchKind::Llama);
    let mut msgs = messages.clone();
    let mut pr = prompt.clone();

    // Хелпер: дописать текст в последний user или создать новый.
    let mut append_text = |text: String| {
        if let Some(ref mut m) = msgs {
            if let Some(last) = m.last_mut() {
                if last.role.to_lowercase() == "user" { last.content = format!("{}\n{}", last.content, text); return; }
            }
            m.push(ChatMessage { role: "user".into(), content: text });
        } else if let Some(ref mut pp) = pr { *pp = format!("{}\n{}", pp.clone(), text); } else { pr = Some(text); }
    };

    match arch {
        ArchKind::Gemma3 => {
            // Единая логика проекта: изображения маркируем через <image>, текстовые файлы дописываем.
            let mut image_sentinels = String::new();
            for att in attachments.iter() {
                if is_image(att) {
                    if !image_sentinels.is_empty() { image_sentinels.push('\n'); }
                    image_sentinels.push_str("<image>");
                } else if is_text(att) {
                    if let Some(bytes) = read_bytes(att)? { append_text(String::from_utf8_lossy(&bytes).to_string()); }
                } else if let Some(name) = att.name.as_ref() {
                    append_text(format!("[attached file: {}]", name));
                }
            }
            if !image_sentinels.is_empty() { append_text(image_sentinels); }
        }
        ArchKind::Gemma => {
            // Политика для текст‑только моделей: игнорируем изображения, дописываем текст.
            for att in attachments.iter() {
                if is_text(att) {
                    if let Some(bytes) = read_bytes(att)? { append_text(String::from_utf8_lossy(&bytes).to_string()); }
                }
            }
        }
        _ => {
            // По умолчанию можно вставлять <image> и дописывать текст — это и есть единая мультимодальность проекта.
            let mut image_sentinels = String::new();
            for att in attachments.iter() {
                if is_image(att) {
                    if !image_sentinels.is_empty() { image_sentinels.push('\n'); }
                    image_sentinels.push_str("<image>");
                } else if is_text(att) {
                    if let Some(bytes) = read_bytes(att)? { append_text(String::from_utf8_lossy(&bytes).to_string()); }
                } else if let Some(name) = att.name.as_ref() {
                    append_text(format!("[attached file: {}]", name));
                }
            }
            if !image_sentinels.is_empty() { append_text(image_sentinels); }
        }
    }

    Ok(AugmentedInput { messages: msgs, prompt: pr })
}

/// Определить, является ли вложение изображением — по MIME или расширению.
pub fn is_image(att: &Attachment) -> bool {
    if let Some(m) = &att.mime { if m.to_lowercase().starts_with("image/") { return true; } }
    if let Some(name) = &att.name {
        let n = name.to_lowercase();
        return n.ends_with(".png") || n.ends_with(".jpg") || n.ends_with(".jpeg") || n.ends_with(".webp") || n.ends_with(".bmp");
    }
    if let Some(path) = &att.path {
        let p = path.to_lowercase();
        return p.ends_with(".png") || p.ends_with(".jpg") || p.ends_with(".jpeg") || p.ends_with(".webp") || p.ends_with(".bmp");
    }
    false
}

/// Определить, является ли вложение текстовым.
pub fn is_text(att: &Attachment) -> bool {
    if let Some(m) = &att.mime { if m.starts_with("text/") { return true; } }
    if let Some(name) = &att.name { let n = name.to_lowercase(); return n.ends_with(".txt") || n.ends_with(".md") || n.ends_with(".log"); }
    if let Some(path) = &att.path { let p = path.to_lowercase(); return p.ends_with(".txt") || p.ends_with(".md") || p.ends_with(".log"); }
    false
}

/// Прочитать байты вложения из base64 или с диска.
pub fn read_bytes(att: &Attachment) -> Result<Option<Vec<u8>>, String> {
    if let Some(b64) = &att.bytes_b64 {
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| format!("Failed to decode base64 attachment: {}", e))?;
        return Ok(Some(decoded));
    }
    if let Some(p) = &att.path {
        let bytes = std::fs::read(p).map_err(|e| format!("Failed to read attachment from path {}: {}", p, e))?;
        return Ok(Some(bytes));
    }
    Ok(None)
}
