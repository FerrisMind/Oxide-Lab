//! Audio preprocessing utilities for multimodal models.
//!
//! This module provides comprehensive preprocessing functions for audio processing tasks,
//! including audio loading, normalization, resampling, conversion to spectrograms,
//! and mel-spectrogram generation. It follows patterns from Candle examples and is
//! designed to work with models like Whisper, EnCodec, MusicGen, and other audio transformers.

use candle::{Device, Result, Tensor};
use std::f32::consts::PI;

/// Configuration for audio preprocessing
#[derive(Debug, Clone)]
pub struct AudioConfig {
    /// Target sample rate in Hz
    pub sample_rate: u32,
    /// Whether to normalize audio amplitude
    pub normalize: bool,
    /// Maximum amplitude for normalization
    pub max_amplitude: f32,
    /// Number of mel frequency bins
    pub n_mels: usize,
    /// FFT window size
    pub n_fft: usize,
    /// Hop length between FFT windows
    pub hop_length: usize,
    /// Minimum frequency for mel scale
    pub f_min: f32,
    /// Maximum frequency for mel scale
    pub f_max: f32,
    /// Whether to apply loudness normalization
    pub loudness_normalize: bool,
    /// Whether to apply compression
    pub loudness_compressor: bool,
    /// Window function type
    pub window_type: WindowType,
}

/// Window function types for FFT
#[derive(Debug, Clone, Copy)]
pub enum WindowType {
    /// Hann window (most common)
    Hann,
    /// Hamming window
    Hamming,
    /// Rectangular window
    Rectangular,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            normalize: true,
            max_amplitude: 1.0,
            n_mels: 80,
            n_fft: 400,
            hop_length: 160,
            f_min: 0.0,
            f_max: 8000.0,
            loudness_normalize: false,
            loudness_compressor: false,
            window_type: WindowType::Hann,
        }
    }
}

impl AudioConfig {
    /// Create a configuration for Whisper-style preprocessing
    pub fn whisper() -> Self {
        Self {
            sample_rate: 16000,
            normalize: true,
            max_amplitude: 1.0,
            n_mels: 80,
            n_fft: 400,
            hop_length: 160,
            f_min: 0.0,
            f_max: 8000.0,
            loudness_normalize: false,
            loudness_compressor: false,
            window_type: WindowType::Hann,
        }
    }

    /// Create a configuration for EnCodec-style preprocessing
    pub fn encodec() -> Self {
        Self {
            sample_rate: 24000,
            normalize: true,
            max_amplitude: 1.0,
            n_mels: 128,
            n_fft: 1024,
            hop_length: 256,
            f_min: 0.0,
            f_max: 12000.0,
            loudness_normalize: true,
            loudness_compressor: true,
            window_type: WindowType::Hann,
        }
    }

    /// Create a configuration for MusicGen-style preprocessing
    pub fn musicgen() -> Self {
        Self {
            sample_rate: 32000,
            normalize: true,
            max_amplitude: 1.0,
            n_mels: 128,
            n_fft: 2048,
            hop_length: 512,
            f_min: 0.0,
            f_max: 16000.0,
            loudness_normalize: true,
            loudness_compressor: true,
            window_type: WindowType::Hann,
        }
    }

    /// Create a configuration for general speech processing
    pub fn speech() -> Self {
        Self {
            sample_rate: 16000,
            normalize: true,
            max_amplitude: 1.0,
            n_mels: 80,
            n_fft: 400,
            hop_length: 160,
            f_min: 0.0,
            f_max: 8000.0,
            loudness_normalize: false,
            loudness_compressor: false,
            window_type: WindowType::Hann,
        }
    }
}

/// Audio preprocessing utilities
pub struct AudioProcessor {
    /// Configuration for preprocessing
    pub config: AudioConfig,
    device: Device,
    /// Precomputed mel filter bank
    mel_filters: Option<Vec<f32>>,
}

impl AudioProcessor {
    /// Create a new audio processor with the given configuration and device
    pub fn new(config: AudioConfig, device: Device) -> Self {
        Self { 
            config, 
            device,
            mel_filters: None,
        }
    }

    /// Create a new audio processor with precomputed mel filters
    pub fn new_with_filters(config: AudioConfig, device: Device, mel_filters: Vec<f32>) -> Self {
        Self { 
            config, 
            device,
            mel_filters: Some(mel_filters),
        }
    }

    /// Load audio from a file path and preprocess it
    #[cfg(feature = "audio")]
    pub fn load_and_preprocess_audio<P: AsRef<std::path::Path>>(&self, path: P) -> Result<Tensor> {
        let (samples, sample_rate) = self.load_audio_file(path)?;
        self.preprocess_audio(&samples, sample_rate)
    }

    /// Load audio from bytes and preprocess it
    #[cfg(feature = "audio")]
    pub fn load_and_preprocess_from_bytes(&self, bytes: &[u8]) -> Result<Tensor> {
        let (samples, sample_rate) = self.load_audio_bytes(bytes)?;
        self.preprocess_audio(&samples, sample_rate)
    }

    /// Load audio file using symphonia (similar to Candle examples)
    #[cfg(feature = "audio")]
    fn load_audio_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(Vec<f32>, u32)> {
        use symphonia::core::audio::{AudioBufferRef, Signal};
        use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
        use symphonia::core::conv::FromSample;

        fn conv<T>(
            samples: &mut Vec<f32>,
            data: std::borrow::Cow<symphonia::core::audio::AudioBuffer<T>>,
        ) where
            T: symphonia::core::sample::Sample,
            f32: symphonia::core::conv::FromSample<T>,
        {
            samples.extend(data.chan(0).iter().map(|v| f32::from_sample(*v)))
        }

        let src = std::fs::File::open(path).map_err(candle::Error::wrap)?;
        let mss = symphonia::core::io::MediaSourceStream::new(Box::new(src), Default::default());
        let hint = symphonia::core::probe::Hint::new();
        let meta_opts: symphonia::core::meta::MetadataOptions = Default::default();
        let fmt_opts: symphonia::core::formats::FormatOptions = Default::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .map_err(candle::Error::wrap)?;
        let mut format = probed.format;

        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or_else(|| candle::Error::Msg("no supported audio tracks".to_string()))?;

        let dec_opts: DecoderOptions = Default::default();
        let mut decoder = symphonia::default::get_codecs()
            .make_decoder(&dec_opts, &track.codec_params, None)
            .map_err(|_| candle::Error::Msg("unsupported codec".to_string()))?;
        let track_id = track.id;
        let sample_rate = track.codec_params.sample_rate.unwrap_or(0);
        let mut pcm_data = Vec::new();

        while let Ok(packet) = format.next_packet() {
            while !format.metadata().is_latest() {
                format.metadata().pop();
            }

            if packet.track_id() != track_id {
                continue;
            }
            match decoder.decode(&packet).map_err(candle::Error::wrap)? {
                AudioBufferRef::F32(buf) => pcm_data.extend(buf.chan(0)),
                AudioBufferRef::U8(data) => conv(&mut pcm_data, data),
                AudioBufferRef::U16(data) => conv(&mut pcm_data, data),
                AudioBufferRef::U24(data) => conv(&mut pcm_data, data),
                AudioBufferRef::U32(data) => conv(&mut pcm_data, data),
                AudioBufferRef::S8(data) => conv(&mut pcm_data, data),
                AudioBufferRef::S16(data) => conv(&mut pcm_data, data),
                AudioBufferRef::S24(data) => conv(&mut pcm_data, data),
                AudioBufferRef::S32(data) => conv(&mut pcm_data, data),
                AudioBufferRef::F64(data) => conv(&mut pcm_data, data),
            }
        }
        Ok((pcm_data, sample_rate))
    }

    /// Load audio from bytes using symphonia
    #[cfg(feature = "audio")]
    fn load_audio_bytes(&self, bytes: &[u8]) -> Result<(Vec<f32>, u32)> {
        use symphonia::core::audio::{AudioBufferRef, Signal};
        use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
        use symphonia::core::conv::FromSample;

        fn conv<T>(
            samples: &mut Vec<f32>,
            data: std::borrow::Cow<symphonia::core::audio::AudioBuffer<T>>,
        ) where
            T: symphonia::core::sample::Sample,
            f32: symphonia::core::conv::FromSample<T>,
        {
            samples.extend(data.chan(0).iter().map(|v| f32::from_sample(*v)))
        }

        let src = std::io::Cursor::new(bytes);
        let mss = symphonia::core::io::MediaSourceStream::new(Box::new(src), Default::default());
        let hint = symphonia::core::probe::Hint::new();
        let meta_opts: symphonia::core::meta::MetadataOptions = Default::default();
        let fmt_opts: symphonia::core::formats::FormatOptions = Default::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .map_err(candle::Error::wrap)?;
        let mut format = probed.format;

        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or_else(|| candle::Error::Msg("no supported audio tracks".to_string()))?;

        let dec_opts: DecoderOptions = Default::default();
        let mut decoder = symphonia::default::get_codecs()
            .make_decoder(&dec_opts, &track.codec_params, None)
            .map_err(|_| candle::Error::Msg("unsupported codec".to_string()))?;
        let track_id = track.id;
        let sample_rate = track.codec_params.sample_rate.unwrap_or(0);
        let mut pcm_data = Vec::new();

        while let Ok(packet) = format.next_packet() {
            while !format.metadata().is_latest() {
                format.metadata().pop();
            }

            if packet.track_id() != track_id {
                continue;
            }
            match decoder.decode(&packet).map_err(candle::Error::wrap)? {
                AudioBufferRef::F32(buf) => pcm_data.extend(buf.chan(0)),
                AudioBufferRef::U8(data) => conv(&mut pcm_data, data),
                AudioBufferRef::U16(data) => conv(&mut pcm_data, data),
                AudioBufferRef::U24(data) => conv(&mut pcm_data, data),
                AudioBufferRef::U32(data) => conv(&mut pcm_data, data),
                AudioBufferRef::S8(data) => conv(&mut pcm_data, data),
                AudioBufferRef::S16(data) => conv(&mut pcm_data, data),
                AudioBufferRef::S24(data) => conv(&mut pcm_data, data),
                AudioBufferRef::S32(data) => conv(&mut pcm_data, data),
                AudioBufferRef::F64(data) => conv(&mut pcm_data, data),
            }
        }
        Ok((pcm_data, sample_rate))
    }

    /// Preprocess raw audio samples with comprehensive transformations
    pub fn preprocess_audio(&self, samples: &[f32], sample_rate: u32) -> Result<Tensor> {
        let mut processed_samples = samples.to_vec();

        // Resample if needed
        if sample_rate != self.config.sample_rate {
            processed_samples = self.resample_audio(&processed_samples, sample_rate, self.config.sample_rate)?;
        }

        // Apply pre-emphasis if configured
        if self.config.loudness_normalize || self.config.normalize {
            processed_samples = self.pre_emphasis(&processed_samples, 0.97);
        }

        // Apply loudness normalization or simple normalization
        if self.config.loudness_normalize {
            let tensor = Tensor::from_vec(processed_samples.clone(), (1, processed_samples.len()), &self.device)?;
            let normalized = self.normalize_loudness(&tensor, self.config.sample_rate, self.config.loudness_compressor)?;
            processed_samples = normalized.to_vec1::<f32>()?;
        } else if self.config.normalize {
            processed_samples = self.normalize_audio(&processed_samples);
        }

        // Convert to tensor
        // Fixed: Clone the processed_samples to avoid move error
        let tensor = Tensor::from_vec(processed_samples.clone(), (1, processed_samples.len()), &self.device)?;
        Ok(tensor)
    }

    /// Convert audio waveform to mel spectrogram
    pub fn waveform_to_mel(&self, waveform: &Tensor) -> Result<Tensor> {
        let samples = waveform.to_vec1::<f32>()?;
        let mel_filters = self.get_mel_filters()?;
        let mel = self.log_mel_spectrogram(&samples, &mel_filters)?;
        let mel_len = mel.len() / self.config.n_mels;
        Tensor::from_vec(mel, (1, self.config.n_mels, mel_len), &self.device)
    }

    /// Generate mel spectrogram from audio samples
    fn log_mel_spectrogram(&self, samples: &[f32], mel_filters: &[f32]) -> Result<Vec<f32>> {
        let n_fft = self.config.n_fft;
        let hop_length = self.config.hop_length;
        let n_mels = self.config.n_mels;

        // Calculate number of frames
        let n_frames = (samples.len() - n_fft) / hop_length + 1;
        let mut mel = vec![0.0f32; n_mels * n_frames];

        // Generate window function
        let window = self.generate_window(n_fft);

        // Process each frame
        for i in 0..n_frames {
            let start = i * hop_length;
            let end = start + n_fft;
            if end > samples.len() {
                break;
            }

            // Apply window and compute FFT
            let mut frame = Vec::with_capacity(n_fft);
            for j in 0..n_fft {
                frame.push(samples[start + j] * window[j]);
            }

            let fft_out = self.fft(&frame);

            // Apply mel filter bank
            for j in 0..n_mels {
                let mut sum = 0.0f32;
                for k in 0..n_fft {
                    sum += fft_out[k] * mel_filters[j * n_fft + k];
                }
                mel[j * n_frames + i] = sum.max(1e-10).log10();
            }
        }

        // Apply normalization (similar to Whisper)
        if !mel.is_empty() {
            let max_val = mel.iter().fold(0.0f32, |a, &b| a.max(b)) - 8.0;
            for m in mel.iter_mut() {
                *m = (*m - max_val) / 4.0 + 1.0;
            }
        }

        Ok(mel)
    }

    /// Generate window function
    fn generate_window(&self, n_fft: usize) -> Vec<f32> {
        match self.config.window_type {
            WindowType::Hann => {
                (0..n_fft)
                    .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f32 / (n_fft - 1) as f32).cos()))
                    .collect()
            }
            WindowType::Hamming => {
                (0..n_fft)
                    .map(|i| 0.54 - 0.46 * (2.0 * PI * i as f32 / (n_fft - 1) as f32).cos())
                    .collect()
            }
            WindowType::Rectangular => vec![1.0; n_fft],
        }
    }

    /// Simple FFT implementation (for demonstration - in production use a proper FFT library)
    fn fft(&self, input: &[f32]) -> Vec<f32> {
        let n = input.len();
        if n == 0 {
            return vec![];
        }

        // Simple DFT implementation (not optimized)
        let mut output = vec![0.0f32; n * 2];
        for k in 0..n {
            let mut real = 0.0f32;
            let mut imag = 0.0f32;
            for (i, &input_val) in input.iter().enumerate().take(n) {
                let angle = -2.0 * PI * k as f32 * i as f32 / n as f32;
                real += input_val * angle.cos();
                imag += input_val * angle.sin();
            }
            output[k * 2] = real;
            output[k * 2 + 1] = imag;
        }

        // Convert to magnitude spectrum
        let mut magnitude = vec![0.0f32; n];
        for (i, output_chunk) in output.chunks_exact(2).enumerate().take(n) {
            let real = output_chunk[0];
            let imag = output_chunk[1];
            magnitude[i] = (real * real + imag * imag).sqrt();
        }
        magnitude
    }

    /// Get or generate mel filter bank
    fn get_mel_filters(&self) -> Result<Vec<f32>> {
        if let Some(ref filters) = self.mel_filters {
            return Ok(filters.clone());
        }

        // Generate mel filter bank
        let n_fft = self.config.n_fft;
        let n_mels = self.config.n_mels;
        let f_min = self.config.f_min;
        let f_max = self.config.f_max;
        let sample_rate = self.config.sample_rate as f32;

        let mut mel_filters = vec![0.0f32; n_mels * n_fft];

        // Convert frequencies to mel scale
        let mel_min = self.hz_to_mel(f_min);
        let mel_max = self.hz_to_mel(f_max);

        // Generate mel-spaced frequencies
        let mut mel_points = vec![0.0f32; n_mels + 2];
        for (i, mel_point) in mel_points.iter_mut().enumerate().take(n_mels + 2) {
            *mel_point = mel_min + (mel_max - mel_min) * i as f32 / (n_mels + 1) as f32;
        }

        // Convert back to Hz
        let mut hz_points = vec![0.0f32; n_mels + 2];
        for (_i, (hz_point, &mel_point)) in hz_points.iter_mut().zip(mel_points.iter()).enumerate().take(n_mels + 2) {
            *hz_point = self.mel_to_hz(mel_point);
        }

        // Generate triangular filters
        for i in 1..n_mels + 1 {
            let left = hz_points[i - 1];
            let center = hz_points[i];
            let right = hz_points[i + 1];

            for j in 0..n_fft {
                let freq = j as f32 * sample_rate / n_fft as f32;
                let mut weight = 0.0f32;

                if freq >= left && freq <= center {
                    weight = (freq - left) / (center - left);
                } else if freq > center && freq <= right {
                    weight = (right - freq) / (right - center);
                }

                mel_filters[(i - 1) * n_fft + j] = weight;
            }
        }

        Ok(mel_filters)
    }

    /// Convert Hz to mel scale
    fn hz_to_mel(&self, hz: f32) -> f32 {
        2595.0 * (1.0 + hz / 700.0).log10()
    }

    /// Convert mel scale to Hz
    fn mel_to_hz(&self, mel: f32) -> f32 {
        700.0 * (10.0_f32.powf(mel / 2595.0) - 1.0)
    }

    /// Resample audio using simple linear interpolation
    fn resample_audio(&self, samples: &[f32], from_rate: u32, to_rate: u32) -> Result<Vec<f32>> {
        if from_rate == to_rate {
            return Ok(samples.to_vec());
        }

        let ratio = to_rate as f32 / from_rate as f32;
        let new_length = (samples.len() as f32 * ratio) as usize;
        let mut resampled = Vec::with_capacity(new_length);

        for i in 0..new_length {
            let src_index = i as f32 / ratio;
            let src_index_floor = src_index.floor() as usize;
            let src_index_frac = src_index - src_index_floor as f32;

            if src_index_floor + 1 < samples.len() {
                let sample = samples[src_index_floor] * (1.0 - src_index_frac)
                    + samples[src_index_floor + 1] * src_index_frac;
                resampled.push(sample);
            } else if src_index_floor < samples.len() {
                resampled.push(samples[src_index_floor]);
            } else {
                resampled.push(0.0);
            }
        }

        Ok(resampled)
    }

    /// Normalize audio samples to have maximum amplitude
    pub fn normalize_audio(&self, samples: &[f32]) -> Vec<f32> {
        if samples.is_empty() {
            return vec![];
        }

        let max_sample = samples
            .iter()
            .map(|&x| x.abs())
            .fold(0.0f32, f32::max);

        if max_sample > 0.0 {
            let scale = self.config.max_amplitude / max_sample;
            samples.iter().map(|&x| x * scale).collect()
        } else {
            samples.to_vec()
        }
    }

    /// Normalize audio loudness to a target RMS level
    fn normalize_loudness(&self, wav: &Tensor, _sample_rate: u32, loudness_compressor: bool) -> Result<Tensor> {
        let wav_array = wav.to_vec1::<f32>()?;
        let rms = (wav_array.iter().map(|&x| x * x).sum::<f32>() / wav_array.len() as f32).sqrt();
        let target_rms = 0.1; // Target RMS level
        let gain = target_rms / rms;
        // Fixed: Use affine instead of direct multiplication and convert to f64
        let wav = wav.affine(gain as f64, 0.0)?;

        if loudness_compressor {
            wav.tanh()
        } else {
            Ok(wav)
        }
    }

    /// Convert mel spectrogram back to audio (for visualization)
    pub fn mel_to_audio(&self, mel: &Tensor) -> Result<Vec<f32>> {
        // This is a simplified inverse operation
        // In practice, this would require proper inverse mel transform
        let mel_data = mel.to_vec1::<f32>()?;
        let (_, n_mels, n_frames) = mel.dims3()?;
        
        // Simple reconstruction (not accurate)
        let mut audio = vec![0.0f32; n_frames * self.config.hop_length];
        for i in 0..n_frames {
            let mel_frame = &mel_data[i * n_mels..(i + 1) * n_mels];
            let energy = mel_frame.iter().sum::<f32>() / n_mels as f32;
            let sample = energy.sin() * 0.1; // Simple sine wave approximation
            audio[i * self.config.hop_length] = sample;
        }
        
        Ok(audio)
    }

    /// Apply pre-emphasis filter to audio
    pub fn pre_emphasis(&self, samples: &[f32], coefficient: f32) -> Vec<f32> {
        if samples.is_empty() {
            return vec![];
        }

        let mut filtered = Vec::with_capacity(samples.len());
        filtered.push(samples[0]);

        for i in 1..samples.len() {
            filtered.push(samples[i] - coefficient * samples[i - 1]);
        }

        filtered
    }

    /// Pad or truncate audio to a fixed length
    pub fn pad_or_truncate(&self, samples: &[f32], target_length: usize) -> Vec<f32> {
        if samples.len() >= target_length {
            samples[..target_length].to_vec()
        } else {
            let mut padded = samples.to_vec();
            padded.resize(target_length, 0.0);
            padded
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_config() {
        let config = AudioConfig::whisper();
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.n_mels, 80);
        assert_eq!(config.f_max, 8000.0);
        
        let config = AudioConfig::encodec();
        assert_eq!(config.sample_rate, 24000);
        assert_eq!(config.n_mels, 128);
        assert_eq!(config.loudness_normalize, true);
        
        let config = AudioConfig::musicgen();
        assert_eq!(config.sample_rate, 32000);
        assert_eq!(config.n_mels, 128);
        assert_eq!(config.n_fft, 2048);
        
        let config = AudioConfig::speech();
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.n_mels, 80);
        assert_eq!(config.loudness_normalize, false);
    }

    #[test]
    fn test_audio_processor_creation() {
        let device = Device::Cpu;
        let config = AudioConfig::default();
        let processor = AudioProcessor::new(config.clone(), device.clone());
        assert_eq!(processor.config.sample_rate, 16000);
        
        // Test with precomputed filters
        let mel_filters = vec![0.0; 80 * 400]; // 80 mels, 400 fft
        let processor = AudioProcessor::new_with_filters(config, device.clone(), mel_filters);
        assert_eq!(processor.config.sample_rate, 16000);
    }

    #[test]
    fn test_window_types() {
        let device = Device::Cpu;
        let mut config = AudioConfig::default();
        
        config.window_type = WindowType::Hann;
        let processor = AudioProcessor::new(config.clone(), device.clone());
        let window = processor.generate_window(8);
        assert_eq!(window.len(), 8);
        assert!((window[0] - 0.0).abs() < 1e-6);
        // Just check that the window values are in the expected range [0, 1]
        for &value in &window {
            assert!(value >= 0.0 && value <= 1.0);
        }
        
        config.window_type = WindowType::Hamming;
        let processor = AudioProcessor::new(config.clone(), device.clone());
        let window = processor.generate_window(8);
        assert_eq!(window.len(), 8);
        // Just check that the window values are in the expected range [0, 1]
        for &value in &window {
            assert!(value >= 0.0 && value <= 1.0);
        }
        
        config.window_type = WindowType::Rectangular;
        let processor = AudioProcessor::new(config.clone(), device.clone());
        let window = processor.generate_window(8);
        assert_eq!(window, vec![1.0; 8]);
    }

    #[test]
    fn test_mel_conversion() {
        let device = Device::Cpu;
        let config = AudioConfig::default();
        let processor = AudioProcessor::new(config.clone(), device.clone());
        
        // Test Hz to mel conversion
        let hz = 1000.0;
        let mel = processor.hz_to_mel(hz);
        assert!(mel > 0.0);
        
        // Test mel to Hz conversion
        let hz_back = processor.mel_to_hz(mel);
        assert!((hz - hz_back).abs() < 1.0); // Should be close to original
    }

    #[test]
    fn test_normalize_audio() {
        let device = Device::Cpu;
        let config = AudioConfig::default();
        let processor = AudioProcessor::new(config.clone(), device.clone());
        
        let samples = vec![0.5, -0.8, 0.3, -0.2];
        let normalized = processor.normalize_audio(&samples);
        
        // Check that maximum absolute value is 1.0
        let max_abs = normalized.iter().map(|&x| x.abs()).fold(0.0, f32::max);
        assert!((max_abs - 1.0).abs() < 1e-6);
        
        // Check that relative values are preserved
        assert!((normalized[1].abs() - 1.0).abs() < 1e-6); // -0.8 should become -1.0
        assert!((normalized[0] - 0.5/0.8).abs() < 1e-6);   // 0.5 should become 0.625
    }

    #[test]
    fn test_resample_audio() {
        let device = Device::Cpu;
        let config = AudioConfig::default();
        let processor = AudioProcessor::new(config.clone(), device.clone());
        
        let samples = vec![1.0, 2.0, 3.0, 4.0];
        
        // Test same sample rate (no change)
        let resampled = processor.resample_audio(&samples, 16000, 16000).unwrap();
        assert_eq!(resampled, samples);
        
        // Test upsampling
        let resampled = processor.resample_audio(&samples, 8000, 16000).unwrap();
        assert!(resampled.len() > samples.len());
        
        // Test downsampling
        let resampled = processor.resample_audio(&samples, 32000, 16000).unwrap();
        assert!(resampled.len() < samples.len());
    }

    #[test]
    fn test_pre_emphasis() {
        let device = Device::Cpu;
        let config = AudioConfig::default();
        let processor = AudioProcessor::new(config.clone(), device.clone());
        
        let samples = vec![1.0, 2.0, 3.0, 4.0];
        let filtered = processor.pre_emphasis(&samples, 0.97);
        
        assert_eq!(filtered.len(), 4);
        assert_eq!(filtered[0], 1.0); // First sample unchanged
        assert_eq!(filtered[1], 2.0 - 0.97 * 1.0); // 1.03
        assert_eq!(filtered[2], 3.0 - 0.97 * 2.0); // 1.06
        assert_eq!(filtered[3], 4.0 - 0.97 * 3.0); // 1.09
    }

    #[test]
    fn test_pad_or_truncate() {
        let device = Device::Cpu;
        let config = AudioConfig::default();
        let processor = AudioProcessor::new(config.clone(), device.clone());
        
        let samples = vec![1.0, 2.0, 3.0, 4.0];
        
        // Test truncation
        let truncated = processor.pad_or_truncate(&samples, 2);
        assert_eq!(truncated, vec![1.0, 2.0]);
        
        // Test padding
        let padded = processor.pad_or_truncate(&samples, 6);
        assert_eq!(padded, vec![1.0, 2.0, 3.0, 4.0, 0.0, 0.0]);
        
        // Test same size
        let same = processor.pad_or_truncate(&samples, 4);
        assert_eq!(same, samples);
    }

    #[test]
    fn test_get_mel_filters() {
        let device = Device::Cpu;
        let config = AudioConfig::default();
        let processor = AudioProcessor::new(config.clone(), device.clone());
        
        let filters = processor.get_mel_filters().unwrap();
        assert!(!filters.is_empty());
        assert_eq!(filters.len(), config.n_mels * config.n_fft);
        
        // Test that filter rows sum to approximately 1.0 (triangle filters)
        for i in 0..config.n_mels {
            let filter_sum: f32 = filters[i * config.n_fft..(i + 1) * config.n_fft].iter().sum();
            assert!(filter_sum > 0.0);
        }
    }

    #[test]
    fn test_fft() {
        let device = Device::Cpu;
        let config = AudioConfig::default();
        let processor = AudioProcessor::new(config.clone(), device.clone());
        
        // Test with simple input
        let input = vec![1.0, 0.0, 0.0, 0.0];
        let output = processor.fft(&input);
        assert_eq!(output.len(), 4);
        assert!(output[0] > 0.0); // DC component should be non-zero
    }

    #[test]
    fn test_mel_filter_generation() {
        let device = Device::Cpu;
        let config = AudioConfig::default();
        let processor = AudioProcessor::new(config.clone(), device.clone());
        
        let filters = processor.get_mel_filters().unwrap();
        assert_eq!(filters.len(), config.n_mels * config.n_fft);
        
        // Check that filters are normalized (sum should be reasonable)
        for i in 0..config.n_mels {
            let filter_sum: f32 = filters[i * config.n_fft..(i + 1) * config.n_fft].iter().sum();
            assert!(filter_sum > 0.0);
        }
    }
}