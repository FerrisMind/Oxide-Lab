//! Multimodal preprocessing utilities and unified interfaces.
//!
//! This module provides unified interfaces for processing different types of multimodal data
//! (images, audio, text) and integrates with the specialized vision and audio processors.
//! It follows patterns from Candle examples and provides a common API for multimodal models.

use candle::{Device, Result, Tensor};
use std::path::Path;

use super::vision::{VisionConfig, VisionProcessor};
use super::audio::{AudioConfig, AudioProcessor};

/// Unified configuration for multimodal preprocessing
#[derive(Debug, Clone)]
pub struct MultimodalConfig {
    /// Vision preprocessing configuration
    pub vision: VisionConfig,
    /// Audio preprocessing configuration
    pub audio: AudioConfig,
    /// Whether to enable streaming processing for large files
    pub streaming: bool,
    /// Maximum batch size for processing
    pub max_batch_size: usize,
    /// Whether to apply data augmentation during training
    pub augmentation: bool,
}

impl Default for MultimodalConfig {
    fn default() -> Self {
        Self {
            vision: VisionConfig::default(),
            audio: AudioConfig::default(),
            streaming: false,
            max_batch_size: 32,
            augmentation: false,
        }
    }
}

impl MultimodalConfig {
    /// Create a configuration optimized for CLIP-style models
    pub fn clip() -> Self {
        Self {
            vision: VisionConfig::clip(),
            audio: AudioConfig::default(),
            streaming: false,
            max_batch_size: 16,
            augmentation: false,
        }
    }

    /// Create a configuration optimized for Whisper-style models
    pub fn whisper() -> Self {
        Self {
            vision: VisionConfig::default(),
            audio: AudioConfig::whisper(),
            streaming: true,
            max_batch_size: 8,
            augmentation: false,
        }
    }

    /// Create a configuration optimized for multimodal training
    pub fn training() -> Self {
        Self {
            vision: VisionConfig::training(),
            audio: AudioConfig::encodec(),
            streaming: true,
            max_batch_size: 16,
            augmentation: true,
        }
    }

    /// Create a configuration optimized for MusicGen-style models
    pub fn musicgen() -> Self {
        Self {
            vision: VisionConfig::default(),
            audio: AudioConfig::musicgen(),
            streaming: true,
            max_batch_size: 4,
            augmentation: false,
        }
    }
}

/// Unified multimodal processor that handles different data types
pub struct MultimodalProcessor {
    /// Configuration for preprocessing
    pub config: MultimodalConfig,
    /// Vision processor
    vision_processor: VisionProcessor,
    /// Audio processor
    audio_processor: AudioProcessor,
    device: Device,
}

impl MultimodalProcessor {
    /// Create a new multimodal processor with the given configuration and device
    pub fn new(config: MultimodalConfig, device: Device) -> Self {
        let vision_processor = VisionProcessor::new(config.vision.clone(), device.clone());
        let audio_processor = AudioProcessor::new(config.audio.clone(), device.clone());
        
        Self {
            config,
            vision_processor,
            audio_processor,
            device,
        }
    }

    /// Create a new multimodal processor with precomputed mel filters
    pub fn new_with_audio_filters(config: MultimodalConfig, device: Device, mel_filters: Vec<f32>) -> Self {
        let vision_processor = VisionProcessor::new(config.vision.clone(), device.clone());
        let audio_processor = AudioProcessor::new_with_filters(config.audio.clone(), device.clone(), mel_filters);
        
        Self {
            config,
            vision_processor,
            audio_processor,
            device,
        }
    }

    /// Process an image file and return a tensor
    pub fn process_image<P: AsRef<Path>>(&self, path: P) -> Result<Tensor> {
        self.vision_processor.load_and_preprocess_image(path)
    }

    /// Process an image from bytes and return a tensor
    pub fn process_image_bytes(&self, bytes: &[u8]) -> Result<Tensor> {
        self.vision_processor.load_and_preprocess_from_bytes(bytes)
    }

    /// Process an audio file and return a tensor
    #[cfg(feature = "audio")]
    pub fn process_audio<P: AsRef<Path>>(&self, path: P) -> Result<Tensor> {
        self.audio_processor.load_and_preprocess_audio(path)
    }

    /// Process audio from bytes and return a tensor
    #[cfg(feature = "audio")]
    pub fn process_audio_bytes(&self, bytes: &[u8]) -> Result<Tensor> {
        self.audio_processor.load_and_preprocess_from_bytes(bytes)
    }

    /// Process audio samples and return a tensor
    pub fn process_audio_samples(&self, samples: &[f32], sample_rate: u32) -> Result<Tensor> {
        self.audio_processor.preprocess_audio(samples, sample_rate)
    }

    /// Convert audio waveform to mel spectrogram
    pub fn audio_to_mel(&self, waveform: &Tensor) -> Result<Tensor> {
        self.audio_processor.waveform_to_mel(waveform)
    }

    /// Process multiple images in batch
    pub fn process_images_batch(&self, image_paths: Vec<&Path>) -> Result<Tensor> {
        let mut tensors = Vec::with_capacity(image_paths.len());
        for path in image_paths {
            tensors.push(self.process_image(path)?);
        }
        Tensor::stack(&tensors, 0)
    }

    /// Process multiple audio files in batch
    #[cfg(feature = "audio")]
    pub fn process_audio_batch(&self, audio_paths: Vec<&Path>) -> Result<Tensor> {
        let mut tensors = Vec::with_capacity(audio_paths.len());
        for path in audio_paths {
            tensors.push(self.process_audio(path)?);
        }
        Tensor::stack(&tensors, 0)
    }

    /// Apply augmentations to a tensor (for training)
    pub fn augment(&self, tensor: &Tensor, modality: Modality) -> Result<Tensor> {
        match modality {
            Modality::Vision => self.vision_processor.augment(tensor),
            Modality::Audio => {
                // For audio, we can apply time-domain augmentations
                // This is a simplified implementation
                Ok(tensor.clone())
            }
        }
    }

    /// Extract features from processed data
    pub fn extract_features(&self, tensor: &Tensor, modality: Modality) -> Result<Tensor> {
        match modality {
            Modality::Vision => self.vision_processor.extract_features(tensor),
            Modality::Audio => {
                // For audio, convert to mel spectrogram if not already
                if tensor.dims().len() == 2 {
                    self.audio_to_mel(tensor)
                } else {
                    Ok(tensor.clone())
                }
            }
        }
    }

    /// Get the device used by the processor
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Get the vision processor
    pub fn vision_processor(&self) -> &VisionProcessor {
        &self.vision_processor
    }

    /// Get the audio processor
    pub fn audio_processor(&self) -> &AudioProcessor {
        &self.audio_processor
    }

    /// Update vision configuration
    pub fn update_vision_config(&mut self, config: VisionConfig) {
        self.config.vision = config.clone();
        self.vision_processor = VisionProcessor::new(config, self.device.clone());
    }

    /// Update audio configuration
    pub fn update_audio_config(&mut self, config: AudioConfig) {
        self.config.audio = config.clone();
        self.audio_processor = AudioProcessor::new(config, self.device.clone());
    }
}

/// Data modality types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modality {
    /// Visual data (images, videos)
    Vision,
    /// Audio data (speech, music, sounds)
    Audio,
}

/// Unified data sample that can contain different modalities
#[derive(Debug, Clone)]
pub struct MultimodalSample {
    /// Visual data tensor
    pub vision: Option<Tensor>,
    /// Audio data tensor
    pub audio: Option<Tensor>,
    /// Text data (for future extension)
    pub text: Option<String>,
    /// Metadata about the sample
    pub metadata: SampleMetadata,
}

/// Metadata for a multimodal sample
#[derive(Debug, Clone)]
pub struct SampleMetadata {
    /// Original file path (if applicable)
    pub file_path: Option<String>,
    /// Sample duration in seconds (for audio/video)
    pub duration: Option<f32>,
    /// Image dimensions (width, height) for vision data
    pub image_size: Option<(u32, u32)>,
    /// Audio sample rate (for audio data)
    pub sample_rate: Option<u32>,
    /// Processing timestamp
    pub timestamp: std::time::SystemTime,
}

impl Default for SampleMetadata {
    fn default() -> Self {
        Self {
            file_path: None,
            duration: None,
            image_size: None,
            sample_rate: None,
            timestamp: std::time::SystemTime::now(),
        }
    }
}

impl Default for MultimodalSample {
    fn default() -> Self {
        Self::new()
    }
}

impl MultimodalSample {
    /// Create a new multimodal sample
    pub fn new() -> Self {
        Self {
            vision: None,
            audio: None,
            text: None,
            metadata: SampleMetadata::default(),
        }
    }

    /// Create a vision-only sample
    pub fn vision(tensor: Tensor, metadata: SampleMetadata) -> Self {
        Self {
            vision: Some(tensor),
            audio: None,
            text: None,
            metadata,
        }
    }

    /// Create an audio-only sample
    pub fn audio(tensor: Tensor, metadata: SampleMetadata) -> Self {
        Self {
            vision: None,
            audio: Some(tensor),
            text: None,
            metadata,
        }
    }

    /// Create a vision-audio sample
    pub fn vision_audio(vision: Tensor, audio: Tensor, metadata: SampleMetadata) -> Self {
        Self {
            vision: Some(vision),
            audio: Some(audio),
            text: None,
            metadata,
        }
    }

    /// Check if the sample has vision data
    pub fn has_vision(&self) -> bool {
        self.vision.is_some()
    }

    /// Check if the sample has audio data
    pub fn has_audio(&self) -> bool {
        self.audio.is_some()
    }

    /// Check if the sample has text data
    pub fn has_text(&self) -> bool {
        self.text.is_some()
    }

    /// Get the number of modalities in the sample
    pub fn modality_count(&self) -> usize {
        let mut count = 0;
        if self.has_vision() { count += 1; }
        if self.has_audio() { count += 1; }
        if self.has_text() { count += 1; }
        count
    }
}

/// Batch processor for multimodal data
pub struct MultimodalBatchProcessor {
    processor: MultimodalProcessor,
    batch_size: usize,
}

impl MultimodalBatchProcessor {
    /// Create a new batch processor
    pub fn new(processor: MultimodalProcessor, batch_size: usize) -> Self {
        Self {
            processor,
            batch_size,
        }
    }

    /// Process a batch of multimodal samples
    pub fn process_batch(&self, samples: Vec<MultimodalSample>) -> Result<Vec<MultimodalSample>> {
        let mut processed_samples = Vec::with_capacity(samples.len());
        
        for sample in samples {
            let mut processed_sample = MultimodalSample::new();
            processed_sample.metadata = sample.metadata.clone();
            
            // Process vision data if present
            if let Some(vision_tensor) = sample.vision {
                let processed_vision = self.processor.extract_features(&vision_tensor, Modality::Vision)?;
                processed_sample.vision = Some(processed_vision);
            }
            
            // Process audio data if present
            if let Some(audio_tensor) = sample.audio {
                let processed_audio = self.processor.extract_features(&audio_tensor, Modality::Audio)?;
                processed_sample.audio = Some(processed_audio);
            }
            
            // Copy text data
            processed_sample.text = sample.text;
            
            processed_samples.push(processed_sample);
        }
        
        Ok(processed_samples)
    }

    /// Process samples in streaming mode for large datasets
    pub fn process_streaming<F>(&self, samples: Vec<MultimodalSample>, mut callback: F) -> Result<()>
    where
        F: FnMut(Vec<MultimodalSample>) -> Result<()>,
    {
        for chunk in samples.chunks(self.batch_size) {
            let processed_chunk = self.process_batch(chunk.to_vec())?;
            callback(processed_chunk)?;
        }
        Ok(())
    }
}

/// Utility functions for multimodal data processing
pub mod utils {
    use super::*;

    /// Convert a tensor to a format suitable for visualization
    pub fn tensor_to_visualizable(tensor: &Tensor, modality: Modality) -> Result<Vec<u8>> {
        match modality {
            Modality::Vision => {
                // Convert tensor back to image bytes
                // This is a simplified implementation
                let data = tensor.flatten_all()?.to_vec1::<f32>()?;
                // In practice, you would convert to actual image format
                Ok(data.iter().map(|&x| (x * 255.0) as u8).collect())
            }
            Modality::Audio => {
                // Convert tensor to audio bytes
                let data = tensor.flatten_all()?.to_vec1::<f32>()?;
                // In practice, you would convert to actual audio format
                Ok(data.iter().map(|&x| (x * 32767.0) as i16 as u8).collect())
            }
        }
    }

    /// Validate that a tensor has the expected shape for a modality
    pub fn validate_tensor_shape(tensor: &Tensor, modality: Modality) -> Result<bool> {
        let dims = tensor.dims();
        match modality {
            Modality::Vision => {
                // Vision tensors should have shape (batch, channels, height, width)
                Ok(dims.len() == 4 && dims[1] == 3)
            }
            Modality::Audio => {
                // Audio tensors should have shape (batch, time) or (batch, channels, time)
                Ok(dims.len() >= 2)
            }
        }
    }

    /// Get the memory usage of a tensor in bytes
    pub fn get_tensor_memory_usage(tensor: &Tensor) -> usize {
        let dims = tensor.dims();
        let element_count: usize = dims.iter().product();
        // Fixed: Handle all DType variants
        let element_size = match tensor.dtype() {
            candle::DType::F32 => 4,
            candle::DType::F16 => 2,
            candle::DType::BF16 => 2,
            candle::DType::U8 => 1,
            candle::DType::U32 => 4,
            candle::DType::I64 => 8,
            // Handle new DType variants
            _ => 8, // Default to 8 bytes for unknown types
        };
        element_count * element_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multimodal_config() {
        let config = MultimodalConfig::clip();
        assert_eq!(config.vision.image_size, 224);
        assert_eq!(config.max_batch_size, 16);
        
        let config = MultimodalConfig::whisper();
        assert_eq!(config.audio.sample_rate, 16000);
        assert_eq!(config.streaming, true);
        
        let config = MultimodalConfig::training();
        assert_eq!(config.augmentation, true);
        assert_eq!(config.vision.random_flip, true);
    }

    #[test]
    fn test_multimodal_processor_creation() {
        let device = Device::Cpu;
        let config = MultimodalConfig::default();
        let _processor = MultimodalProcessor::new(config, device);
        // Just test that we can create the processor successfully
    }

    #[test]
    fn test_multimodal_sample() {
        let device = Device::Cpu;
        let tensor = Tensor::zeros((1, 3, 224, 224), candle::DType::F32, &device).unwrap();
        let metadata = SampleMetadata::default();
        
        let sample = MultimodalSample::vision(tensor, metadata);
        assert!(sample.has_vision());
        assert!(!sample.has_audio());
        assert!(!sample.has_text());
        assert_eq!(sample.modality_count(), 1);
    }

    #[test]
    fn test_modality_enum() {
        assert_eq!(Modality::Vision, Modality::Vision);
        assert_ne!(Modality::Vision, Modality::Audio);
    }

    #[test]
    fn test_utils() {
        let device = Device::Cpu;
        let tensor = Tensor::zeros((1, 3, 224, 224), candle::DType::F32, &device).unwrap();
        
        let is_valid = utils::validate_tensor_shape(&tensor, Modality::Vision).unwrap();
        assert!(is_valid);
        
        let memory_usage = utils::get_tensor_memory_usage(&tensor);
        assert!(memory_usage > 0);
    }
}
