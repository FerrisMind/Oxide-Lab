use std::time::{Duration, Instant};
use tauri::Emitter;

pub struct ChunkEmitter {
    app: tauri::AppHandle,
    buffer: String,
    last_emit_at: Instant,
    emit_interval: Duration,
    max_chunk_len: usize,
    done_emitted: bool,
    // Резерв: буфер для эмиссии токенов
}

impl ChunkEmitter {
    pub fn new(app: tauri::AppHandle) -> Self {
        Self {
            app,
            buffer: String::new(),
            last_emit_at: Instant::now(),
            emit_interval: Duration::from_millis(16),
            max_chunk_len: 2048,
            done_emitted: false,
        }
    }

    pub fn push_maybe_emit(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.buffer.push_str(text);
        let elapsed = self.last_emit_at.elapsed();
        if elapsed >= self.emit_interval || self.buffer.len() >= self.max_chunk_len {
            let chunk = std::mem::take(&mut self.buffer);
            let out = chunk;
            if !out.is_empty() {
                let _ = self.app.emit("token", out);
            }
            self.last_emit_at = Instant::now();
        }
    }

    pub fn flush(&mut self) {
        if !self.buffer.is_empty() {
            let chunk = std::mem::take(&mut self.buffer);
            let out = chunk;
            if !out.is_empty() {
                let _ = self.app.emit("token", out);
            }
            self.last_emit_at = Instant::now();
        }
    }

    pub fn finalize(&mut self) {
        self.flush();
        if !self.done_emitted {
            let _ = self.app.emit("token", "[DONE]");
            self.done_emitted = true;
        }
    }
}

impl Drop for ChunkEmitter {
    fn drop(&mut self) {
        self.finalize();
    }
}
