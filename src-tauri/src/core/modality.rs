use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModalitySupportDto {
    pub text: bool,
    pub image: bool,
    pub audio: bool,
    pub video: bool,
}

/// Detect supported modalities from a config.json (HF style) value.
/// Rules are deterministic for provided configs:
/// - text: always true for LLMs in this app context
/// - image: presence of vision-related fields (vision_config, mm_vision_tower, image_token_index, mm_tokens_per_image, use_mm_proj)
/// - audio: presence of audio-related fields (audio_config, audio_token_index, model_type contains "audio", architectures contains "Audio")
/// - video: vision + presence of video flags (add_time_instruction or add_faster_video)
pub fn detect_from_config(config: &serde_json::Value) -> ModalitySupportDto {
    let mut out = ModalitySupportDto { text: true, image: false, audio: false, video: false };

    let has = |k: &str| config.get(k).is_some();
    let has_str = |k: &str, needle: &str| {
        config
            .get(k)
            .and_then(|v| v.as_str())
            .map(|s| s.to_lowercase().contains(&needle.to_lowercase()))
            .unwrap_or(false)
    };

    // image input detection
    let image_from_fields = has("vision_config")
        || has("mm_vision_tower")
        || has("image_token_index")
        || has("mm_tokens_per_image")
        || config.get("use_mm_proj").and_then(|v| v.as_bool()).unwrap_or(false);
    out.image = image_from_fields;

    // audio input detection
    let audio_from_fields = has("audio_config") || has("audio_token_index") || has_str("model_type", "audio");
    let audio_from_arch = config
        .get("architectures")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().any(|x| x.as_str().map(|s| s.to_lowercase().contains("audio")).unwrap_or(false)))
        .unwrap_or(false);
    out.audio = audio_from_fields || audio_from_arch;

    // video input requires vision + explicit video flags
    let video_flags_present = has("add_time_instruction") || has("add_faster_video");
    out.video = out.image && video_flags_present;

    out
}

