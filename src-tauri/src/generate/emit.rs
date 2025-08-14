use std::time::{Duration, Instant};
use tauri::Emitter;

pub struct ChunkEmitter {
    app: tauri::AppHandle,
    buffer: String,
    last_emit_at: Instant,
    emit_interval: Duration,
    max_chunk_len: usize,
}

impl ChunkEmitter {
    pub fn new(app: tauri::AppHandle) -> Self {
        Self {
            app,
            buffer: String::new(),
            last_emit_at: Instant::now(),
            emit_interval: Duration::from_millis(16),
            max_chunk_len: 2048,
        }
    }

    pub fn push_maybe_emit(&mut self, text: &str) {
        if text.is_empty() { return; }
        self.buffer.push_str(text);
        let elapsed = self.last_emit_at.elapsed();
        if elapsed >= self.emit_interval || self.buffer.len() >= self.max_chunk_len {
            let chunk = std::mem::take(&mut self.buffer);
            let _ = self.app.emit("token", chunk);
            self.last_emit_at = Instant::now();
        }
    }

    pub fn flush(&mut self) {
        if !self.buffer.is_empty() {
            let chunk = std::mem::take(&mut self.buffer);
            let _ = self.app.emit("token", chunk);
            self.last_emit_at = Instant::now();
        }
    }
}


