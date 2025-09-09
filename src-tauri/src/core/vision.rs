//! Vision preprocessing utilities for multimodal models.
//!
//! This module provides comprehensive preprocessing functions for computer vision tasks,
//! including image loading, normalization, resizing, augmentation, and conversion to tensors.
//! It follows patterns from Candle examples and is designed to work with models
//! like ViT, CLIP, DINOv2, and other vision transformers.

use candle::{Device, Result, Tensor, DType};
use std::path::Path;
use image::{DynamicImage, GenericImageView, ImageBuffer, RgbImage};
// Fixed FilterType import - it's in imageops module now
use image::imageops::FilterType;
// Added rand crate for random number generation
use rand;

/// Configuration for vision preprocessing
#[derive(Debug, Clone)]
pub struct VisionConfig {
    /// Target image size (width = height)
    pub image_size: usize,
    /// Normalization mean values for each channel (typically [0.485, 0.456, 0.406] for ImageNet)
    pub mean: [f32; 3],
    /// Normalization std values for each channel (typically [0.229, 0.224, 0.225] for ImageNet)
    pub std: [f32; 3],
    /// Whether to normalize the image
    pub normalize: bool,
    /// Resize method to use
    pub resize_method: ResizeMethod,
    /// Whether to center crop the image
    pub center_crop: bool,
    /// Whether to apply random horizontal flip during training
    pub random_flip: bool,
    /// Color jitter parameters (hue, saturation, brightness, contrast)
    pub color_jitter: Option<ColorJitter>,
}

/// Resize method for images
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResizeMethod {
    /// Resize to exact dimensions
    Exact,
    /// Resize to fill the target dimensions (may crop)
    Fill,
    /// Resize longest side to target size, maintaining aspect ratio
    ResizeLongest,
    /// Resize shortest side to target size, maintaining aspect ratio
    ResizeShortest,
}

/// Color jitter parameters
#[derive(Debug, Clone)]
pub struct ColorJitter {
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub hue: f32,
}

impl Default for VisionConfig {
    fn default() -> Self {
        Self {
            image_size: 224,
            mean: [0.485, 0.456, 0.406],
            std: [0.229, 0.224, 0.225],
            normalize: true,
            resize_method: ResizeMethod::Fill,
            center_crop: false,
            random_flip: false,
            color_jitter: None,
        }
    }
}

impl VisionConfig {
    /// Create a configuration for ImageNet-style preprocessing
    pub fn imagenet() -> Self {
        Self::default()
    }

    /// Create a configuration for CLIP-style preprocessing
    pub fn clip() -> Self {
        Self {
            image_size: 224,
            mean: [0.48145466, 0.4578275, 0.40821073],
            std: [0.26862954, 0.2613026, 0.2757771],
            normalize: true,
            resize_method: ResizeMethod::Fill,
            center_crop: false,
            random_flip: false,
            color_jitter: None,
        }
    }

    /// Create a configuration for DINOv2-style preprocessing
    pub fn dinov2() -> Self {
        Self {
            image_size: 518,
            mean: [0.485, 0.456, 0.406],
            std: [0.229, 0.224, 0.225],
            normalize: true,
            resize_method: ResizeMethod::ResizeLongest,
            center_crop: true,
            random_flip: false,
            color_jitter: None,
        }
    }

    /// Create a configuration for training with augmentations
    pub fn training() -> Self {
        Self {
            image_size: 224,
            mean: [0.485, 0.456, 0.406],
            std: [0.229, 0.224, 0.225],
            normalize: true,
            resize_method: ResizeMethod::ResizeLongest,
            center_crop: true,
            random_flip: true,
            color_jitter: Some(ColorJitter {
                brightness: 0.2,
                contrast: 0.2,
                saturation: 0.2,
                hue: 0.1,
            }),
        }
    }
}

/// Vision preprocessing utilities
pub struct VisionProcessor {
    /// Configuration for preprocessing
    pub config: VisionConfig,
    device: Device,
}

impl VisionProcessor {
    /// Create a new vision processor with the given configuration and device
    pub fn new(config: VisionConfig, device: Device) -> Self {
        Self { config, device }
    }

    /// Load an image from a file path and preprocess it
    pub fn load_and_preprocess_image<P: AsRef<Path>>(&self, path: P) -> Result<Tensor> {
        let image = image::ImageReader::open(path)
            .map_err(candle::Error::wrap)?
            .decode()
            .map_err(candle::Error::wrap)?;
        self.preprocess_image(image)
    }

    /// Load an image from bytes and preprocess it
    pub fn load_and_preprocess_from_bytes(&self, bytes: &[u8]) -> Result<Tensor> {
        let image = image::load_from_memory(bytes)
            .map_err(candle::Error::wrap)?;
        self.preprocess_image(image)
    }

    /// Preprocess an image buffer with comprehensive transformations
    pub fn preprocess_image(&self, mut img: DynamicImage) -> Result<Tensor> {
        // Convert to RGB if not already
        img = img.to_rgb8().into();

        // Apply resizing based on configuration
        img = self.resize_image(img)?;

        // Apply center crop if configured
        if self.config.center_crop {
            img = self.center_crop_image(img)?;
        }

        // Convert to tensor with shape (3, height, width)
        let (width, height) = img.dimensions();
        let img_data = img.into_rgb8().into_raw();
        let tensor = Tensor::from_vec(img_data, (height as usize, width as usize, 3), &self.device)?
            .permute((2, 0, 1))?;

        // Convert to float and normalize to [0, 1]
        let tensor = tensor.to_dtype(DType::F32)?.affine(1.0 / 255.0, 0.0)?;

        // Apply normalization if requested
        if self.config.normalize {
            self.normalize_tensor(&tensor)
        } else {
            Ok(tensor)
        }
    }

    /// Normalize tensor using mean and std
    fn normalize_tensor(&self, tensor: &Tensor) -> Result<Tensor> {
        let mean = Tensor::from_slice(&self.config.mean, (3, 1, 1), &self.device)?;
        let std = Tensor::from_slice(&self.config.std, (3, 1, 1), &self.device)?;
        tensor.broadcast_sub(&mean)?.broadcast_div(&std)
    }

    /// Resize image based on configuration
    fn resize_image(&self, img: DynamicImage) -> Result<DynamicImage> {
        let (current_width, current_height) = img.dimensions();
        let target_size = self.config.image_size as u32;

        let (new_width, new_height) = match self.config.resize_method {
            ResizeMethod::Exact => (target_size, target_size),
            ResizeMethod::Fill => (target_size, target_size),
            ResizeMethod::ResizeLongest => {
                if current_width > current_height {
                    (target_size, (target_size * current_height) / current_width)
                } else {
                    ((target_size * current_width) / current_height, target_size)
                }
            }
            ResizeMethod::ResizeShortest => {
                if current_width < current_height {
                    (target_size, (target_size * current_height) / current_width)
                } else {
                    ((target_size * current_width) / current_height, target_size)
                }
            }
        };

        let filter = match self.config.resize_method {
            ResizeMethod::Exact | ResizeMethod::Fill => FilterType::Triangle,
            ResizeMethod::ResizeLongest | ResizeMethod::ResizeShortest => FilterType::CatmullRom,
        };

        // Fixed: resize returns ImageBuffer, need to convert to DynamicImage
        Ok(DynamicImage::ImageRgba8(image::imageops::resize(&img, new_width, new_height, filter)))
    }

    /// Center crop image to target size
    fn center_crop_image(&self, img: DynamicImage) -> Result<DynamicImage> {
        let (current_width, current_height) = img.dimensions();
        let target_size = self.config.image_size as u32;

        if current_width == target_size && current_height == target_size {
            return Ok(img);
        }

        let left = (current_width - target_size) / 2;
        let top = (current_height - target_size) / 2;

        // Fixed: crop_imm is now in imageops module and we need to convert to DynamicImage
        Ok(DynamicImage::ImageRgba8(image::imageops::crop_imm(&img, left, top, target_size, target_size).to_image()))
    }

    /// Apply horizontal flip to tensor
    fn horizontal_flip(&self, tensor: &Tensor) -> Result<Tensor> {
        let (_c, _h, w) = tensor.dims3()?;  // Fixed: prefix unused variables with underscore
        let indices: Vec<i64> = (0..w as i64).rev().collect();
        let indices = Tensor::from_slice(&indices, (w,), &self.device)?;
        tensor.index_select(&indices, 2)
    }

    /// Apply augmentations to an image tensor (for training)
    pub fn augment(&self, tensor: &Tensor) -> Result<Tensor> {
        let mut result = tensor.clone();

        // Apply random horizontal flip if configured
        if self.config.random_flip && rand::random::<f32>() < 0.5 {
            result = self.horizontal_flip(&result)?;
        }

        // Apply color jitter if configured
        if let Some(ref color_jitter) = self.config.color_jitter {
            result = self.apply_color_jitter(&result, color_jitter)?;
        }

        Ok(result)
    }

    /// Apply color jitter to tensor
    fn apply_color_jitter(&self, tensor: &Tensor, jitter: &ColorJitter) -> Result<Tensor> {
        let mut result = tensor.clone();

        // Apply brightness
        if jitter.brightness > 0.0 {
            let brightness_factor = 1.0 + (rand::random::<f32>() - 0.5) * jitter.brightness;
            result = result.affine(brightness_factor as f64, 0.0)?;
        }

        // Apply contrast
        if jitter.contrast > 0.0 {
            let contrast_factor = 1.0 + (rand::random::<f32>() - 0.5) * jitter.contrast;
            result = result.affine(contrast_factor as f64, 0.0)?;
        }

        // Apply saturation (simplified - in practice would need HSV conversion)
        if jitter.saturation > 0.0 {
            let saturation_factor = 1.0 + (rand::random::<f32>() - 0.5) * jitter.saturation;
            result = result.affine(saturation_factor as f64, 0.0)?;
        }

        Ok(result)
    }

    /// Resize an image tensor to a target size
    pub fn resize_tensor(&self, tensor: &Tensor, target_height: usize, target_width: usize) -> Result<Tensor> {
        tensor.interpolate2d(target_height, target_width)
    }

    /// Convert tensor back to image for visualization
    pub fn tensor_to_image(&self, tensor: &Tensor) -> Result<RgbImage> {
        let (c, h, w) = tensor.dims3()?;
        if c != 3 {
            return Err(candle::Error::Msg("Expected 3 channels for RGB image".to_string()));
        }

        // Denormalize if normalized
        let tensor = if self.config.normalize {
            let mean = Tensor::from_slice(&self.config.mean, (3, 1, 1), &self.device)?;
            let std = Tensor::from_slice(&self.config.std, (3, 1, 1), &self.device)?;
            tensor.broadcast_mul(&std)?.broadcast_add(&mean)?
        } else {
            tensor.clone()
        };

        // Clamp to [0, 1] and convert to u8
        let tensor = tensor.clamp(0.0, 1.0)?;
        let tensor = tensor.affine(255.0, 0.0)?.to_dtype(DType::U8)?;

        // Convert to image
        let data = tensor.permute((1, 2, 0))?.flatten_all()?.to_vec1::<u8>()?;
        ImageBuffer::from_raw(w as u32, h as u32, data)
            .ok_or_else(|| candle::Error::Msg("Failed to create image from tensor".to_string()))
    }

    /// Extract features from image using a vision model
    pub fn extract_features(&self, tensor: &Tensor) -> Result<Tensor> {
        // This is a placeholder - in practice would use a loaded vision model
        // For now, just return the tensor as-is
        Ok(tensor.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vision_config() {
        let config = VisionConfig::imagenet();
        assert_eq!(config.image_size, 224);
        assert_eq!(config.mean, [0.485, 0.456, 0.406]);
        assert_eq!(config.std, [0.229, 0.224, 0.225]);
        
        let config = VisionConfig::clip();
        assert_eq!(config.image_size, 224);
        assert_eq!(config.mean, [0.48145466, 0.4578275, 0.40821073]);
        
        let config = VisionConfig::dinov2();
        assert_eq!(config.image_size, 518);
        assert_eq!(config.center_crop, true);
        
        let config = VisionConfig::training();
        assert_eq!(config.random_flip, true);
        assert!(config.color_jitter.is_some());
    }

    #[test]
    fn test_vision_processor_creation() {
        let device = Device::Cpu;
        let config = VisionConfig::default();
        let processor = VisionProcessor::new(config, device);
        assert_eq!(processor.config.image_size, 224);
    }

    #[test]
    fn test_resize_methods() {
        let device = Device::Cpu;
        let mut config = VisionConfig::default();
        
        // Test different resize methods
        config.resize_method = ResizeMethod::Exact;
        let processor = VisionProcessor::new(config.clone(), device.clone());
        assert_eq!(processor.config.resize_method, ResizeMethod::Exact);
        
        config.resize_method = ResizeMethod::ResizeLongest;
        let processor = VisionProcessor::new(config.clone(), device.clone());
        assert_eq!(processor.config.resize_method, ResizeMethod::ResizeLongest);
    }

    #[test]
    fn test_color_jitter() {
        let jitter = ColorJitter {
            brightness: 0.2,
            contrast: 0.2,
            saturation: 0.2,
            hue: 0.1,
        };
        
        assert_eq!(jitter.brightness, 0.2);
        assert_eq!(jitter.contrast, 0.2);
        assert_eq!(jitter.saturation, 0.2);
        assert_eq!(jitter.hue, 0.1);
    }
}