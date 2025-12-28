use std::time::{Duration, Instant};
use tauri::Emitter;

use crate::core::types::StreamMessage;
use crate::generate::thinking_parser::ParsedChunk;

pub struct ChunkEmitter {
    app: tauri::AppHandle,
    buffer: String,
    thinking_buffer: String,
    content_buffer: String,
    last_emit_at: Instant,
    emit_interval: Duration,
    max_chunk_len: usize,
    done_emitted: bool,
}

impl ChunkEmitter {
    pub fn new(app: tauri::AppHandle) -> Self {
        Self {
            app,
            buffer: String::new(),
            thinking_buffer: String::new(),
            content_buffer: String::new(),
            last_emit_at: Instant::now(),
            emit_interval: Duration::from_millis(16),
            max_chunk_len: 2048,
            done_emitted: false,
        }
    }

    /// Legacy method for backward compatibility — emits raw tokens.
    pub fn push_maybe_emit(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.buffer.push_str(text);
        let elapsed = self.last_emit_at.elapsed();
        if elapsed >= self.emit_interval || self.buffer.len() >= self.max_chunk_len {
            let chunk = std::mem::take(&mut self.buffer);
            if !chunk.is_empty() {
                let _ = self.app.emit("token", chunk);
            }
            self.last_emit_at = Instant::now();
        }
    }

    /// Emit structured message with thinking and content fields.
    pub fn emit_message(&mut self, chunk: ParsedChunk) {
        if chunk.is_empty() {
            return;
        }

        self.thinking_buffer.push_str(&chunk.thinking);
        self.content_buffer.push_str(&chunk.content);

        let elapsed = self.last_emit_at.elapsed();
        let total_len = self.thinking_buffer.len() + self.content_buffer.len();

        if elapsed >= self.emit_interval || total_len >= self.max_chunk_len {
            self.flush_message();
        }
    }

    /// Flush accumulated message buffers.
    pub fn flush_message(&mut self) {
        let thinking = std::mem::take(&mut self.thinking_buffer);
        let content = std::mem::take(&mut self.content_buffer);

        if !thinking.is_empty() || !content.is_empty() {
            log::debug!(
                "[emit] message: thinking={}chars, content={}chars",
                thinking.len(),
                content.len()
            );
            let msg = StreamMessage { thinking, content };
            let _ = self.app.emit("message", &msg);
            self.last_emit_at = Instant::now();
        }
    }

    /// Emit start signal to initialize assistant message on frontend.
    pub fn emit_start(&self) {
        log::debug!("[emit] message_start");
        let _ = self.app.emit("message_start", ());
    }

    pub fn flush(&mut self) {
        // Flush legacy buffer
        if !self.buffer.is_empty() {
            let chunk = std::mem::take(&mut self.buffer);
            if !chunk.is_empty() {
                let _ = self.app.emit("token", chunk);
            }
            self.last_emit_at = Instant::now();
        }
        // Flush structured buffers
        self.flush_message();
    }

    pub fn finalize(&mut self) {
        self.flush();
        if !self.done_emitted {
            let _ = self.app.emit("token", "[DONE]");
            // Also emit done signal for structured stream
            let _ = self.app.emit("message_done", ());
            self.done_emitted = true;
        }
    }
}

impl Drop for ChunkEmitter {
    fn drop(&mut self) {
        self.finalize();
    }
}
