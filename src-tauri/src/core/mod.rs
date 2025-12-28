pub mod audio_capture;
pub mod config;
pub mod device;
pub mod log;
pub mod performance;
pub mod precision;
pub mod prompt;
pub mod state;
pub mod stt_whisper;
pub mod token_output_stream;
pub mod tokenizer;
pub mod types;
pub mod weights;
// Убрали мультимодальность: vision/audio/multimodal/attachments/attachment_router удалены
pub mod attachments_text;
pub mod background_mode;
pub mod rayon_pool;
pub mod thread_priority;

pub use rayon_pool::INFERENCE_POOL;
