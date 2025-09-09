//! Tests for multimodal preprocessing utilities

use candle::{Device, Result};
use llm_chat_lib::core::vision::{VisionConfig, VisionProcessor};
use llm_chat_lib::core::audio::{AudioConfig, AudioProcessor};

#[test]
fn test_vision_preprocessing() -> Result<()> {
    let device = Device::Cpu;
    let config = VisionConfig::imagenet();
    let processor = VisionProcessor::new(config, device);
    
    // Test configuration
    assert_eq!(processor.config.image_size, 224);
    assert_eq!(processor.config.mean, [0.485, 0.456, 0.406]);
    assert_eq!(processor.config.std, [0.229, 0.224, 0.225]);
    
    // Test CLIP configuration
    let clip_config = VisionConfig::clip();
    assert_eq!(clip_config.mean, [0.48145466, 0.4578275, 0.40821073]);
    assert_eq!(clip_config.std, [0.26862954, 0.26130258, 0.27577711]);
    
    println!("Vision preprocessing tests passed!");
    Ok(())
}

#[test]
fn test_audio_preprocessing() -> Result<()> {
    let device = Device::Cpu;
    let config = AudioConfig::whisper();
    let processor = AudioProcessor::new(config, device);
    
    // Test configuration
    assert_eq!(processor.config.sample_rate, 16000);
    assert_eq!(processor.config.n_mels, 80);
    assert_eq!(processor.config.n_fft, 400);
    assert_eq!(processor.config.hop_length, 160);
    
    // Test audio normalization
    let samples = vec![0.5, -0.8, 0.3, -0.2];
    let normalized = processor.normalize_audio(&samples);
    
    // Check that maximum absolute value is 1.0
    let max_abs = normalized.iter().map(|&x| x.abs()).fold(0.0, f32::max);
    assert!((max_abs - 1.0).abs() < 1e-6);
    
    // Test pre-emphasis filter
    let samples = vec![1.0, 2.0, 3.0, 4.0];
    let filtered = processor.pre_emphasis(&samples, 0.97);
    
    assert_eq!(filtered.len(), 4);
    assert_eq!(filtered[0], 1.0); // First sample unchanged
    assert_eq!(filtered[1], 2.0 - 0.97 * 1.0); // 1.03
    assert_eq!(filtered[2], 3.0 - 0.97 * 2.0); // 1.06
    assert_eq!(filtered[3], 4.0 - 0.97 * 3.0); // 1.09
    
    // Test pad or truncate
    let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let truncated = processor.pad_or_truncate(&samples, 3);
    assert_eq!(truncated, vec![1.0, 2.0, 3.0]);
    
    let samples = vec![1.0, 2.0];
    let padded = processor.pad_or_truncate(&samples, 5);
    assert_eq!(padded, vec![1.0, 2.0, 0.0, 0.0, 0.0]);
    
    println!("Audio preprocessing tests passed!");
    Ok(())
}

#[test]
fn test_audio_configs() {
    let whisper_config = AudioConfig::whisper();
    assert_eq!(whisper_config.sample_rate, 16000);
    assert_eq!(whisper_config.n_mels, 80);
    
    let encodec_config = AudioConfig::encodec();
    assert_eq!(encodec_config.sample_rate, 24000);
    assert_eq!(encodec_config.n_mels, 128);
    
    println!("Audio config tests passed!");
}