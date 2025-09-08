use std::time::{Duration, Instant};
use tauri::Emitter;

pub struct ChunkEmitter {
    app: tauri::AppHandle,
    buffer: String,
    last_emit_at: Instant,
    emit_interval: Duration,
    max_chunk_len: usize,
    // Если true — удаляем содержимое между тегами <think>...</think> в потоке
    strip_think: bool,
    // Флаг состояния — находимся ли мы внутри незакрытого блока <think>
    in_think_block: bool,
}

impl ChunkEmitter {
    pub fn new(app: tauri::AppHandle, strip_think: bool) -> Self {
        Self {
            app,
            buffer: String::new(),
            last_emit_at: Instant::now(),
            emit_interval: Duration::from_millis(16),
            max_chunk_len: 2048,
            strip_think,
            in_think_block: false,
        }
    }

    pub fn push_maybe_emit(&mut self, text: &str) {
        if text.is_empty() { return; }
        self.buffer.push_str(text);
        let elapsed = self.last_emit_at.elapsed();
        if elapsed >= self.emit_interval || self.buffer.len() >= self.max_chunk_len {
            let chunk = std::mem::take(&mut self.buffer);
            let out = if self.strip_think {
                self.filter_think(&chunk)
            } else { chunk };
            if !out.is_empty() {
                let _ = self.app.emit("token", out);
            }
            self.last_emit_at = Instant::now();
        }
    }

    pub fn flush(&mut self) {
        if !self.buffer.is_empty() {
            let chunk = std::mem::take(&mut self.buffer);
            let out = if self.strip_think { self.filter_think(&chunk) } else { chunk };
            if !out.is_empty() {
                let _ = self.app.emit("token", out);
            }
            self.last_emit_at = Instant::now();
        }
    }

    // Удаляет содержимое между <think> и </think>, корректно обрабатывая границы между чанками
    fn filter_think(&mut self, mut s: &str) -> String {
        let mut out = String::new();
        while !s.is_empty() {
            if self.in_think_block {
                if let Some(pos) = find_case_insensitive(s, "</think>") {
                    // Найден закрывающий тег — переходим к символам после него
                    s = &s[pos + "</think>".len()..];
                    self.in_think_block = false;
                    continue;
                } else {
                    // Закрывающий тег не найден — отбрасываем весь фрагмент
                    return out;
                }
            } else if let Some(pos) = find_case_insensitive(s, "<think>") {
                // Добавляем всё до открывающего тега и начинаем пропуск
                out.push_str(&s[..pos]);
                s = &s[pos + "<think>".len()..];
                self.in_think_block = true;
                continue;
            } else {
                // Открывающий тег не найден — добавляем остаток и выходим
                out.push_str(s);
                break;
            }
        }
        out
    }
}

// Небольшая помощница для поиска без учёта регистра
fn find_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    let h = haystack.to_lowercase();
    let n = needle.to_lowercase();
    h.find(&n)
}


