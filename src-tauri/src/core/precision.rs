//! Centralized dtype/precision policy management.
//! 
//! This module provides a unified policy for selecting appropriate data types
//! based on the target device and user preferences. The policy helps ensure
//! consistent memory usage and performance across different hardware platforms.

use candle::{DType, Device};
use serde::{Deserialize, Serialize};

/// Precision policy configuration enum for UI settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrecisionPolicy {
    /// Default precision policy (CPU=F32, GPU=BF16)
    Default,
    /// Memory efficient policy (GPU=F16)
    MemoryEfficient,
    /// Maximum precision policy (GPU=F32)
    MaximumPrecision,
}

/// Precision policy configuration
#[derive(Debug, Clone, PartialEq)]
pub struct PrecisionConfig {
    /// Default dtype for CPU devices
    pub cpu_dtype: DType,
    /// Default dtype for GPU devices (CUDA/Metal)
    pub gpu_dtype: DType,
    /// Whether to allow user override of dtype
    pub allow_override: bool,
}

impl Default for PrecisionConfig {
    /// Creates a default precision configuration with recommended settings:
    /// - CPU: F32 for maximum compatibility
    /// - GPU: BF16 for better performance and memory usage
    fn default() -> Self {
        Self {
            cpu_dtype: DType::F32,
            gpu_dtype: DType::BF16,
            allow_override: true,
        }
    }
}

impl PrecisionConfig {
    /// Creates a new precision configuration
    pub fn new(cpu_dtype: DType, gpu_dtype: DType, allow_override: bool) -> Self {
        Self {
            cpu_dtype,
            gpu_dtype,
            allow_override,
        }
    }

    /// Creates a configuration optimized for memory efficiency
    /// - CPU: F32 (no change, as it's already the most compatible)
    /// - GPU: F16 (uses less memory than BF16)
    pub fn memory_efficient() -> Self {
        Self {
            cpu_dtype: DType::F32,
            gpu_dtype: DType::F16,
            allow_override: true,
        }
    }

    /// Creates a configuration optimized for maximum precision
    /// - CPU: F32
    /// - GPU: F32 (highest precision)
    pub fn maximum_precision() -> Self {
        Self {
            cpu_dtype: DType::F32,
            gpu_dtype: DType::F32,
            allow_override: true,
        }
    }
}

/// Selects the appropriate dtype based on device and configuration
/// 
/// # Arguments
/// * `device` - Target device for computation
/// * `config` - Precision configuration policy
/// 
/// # Returns
/// * `DType` - Recommended data type for the given device
pub fn select_dtype(device: &Device, config: &PrecisionConfig) -> DType {
    match device {
        Device::Cpu => config.cpu_dtype,
        Device::Cuda(_) | Device::Metal(_) => config.gpu_dtype,
    }
}

/// Selects the appropriate dtype based on device with default configuration
/// 
/// This function uses the default precision configuration which:
/// - Uses F32 for CPU devices for maximum compatibility
/// - Uses BF16 for GPU devices for better performance
/// 
/// # Arguments
/// * `device` - Target device for computation
/// 
/// # Returns
/// * `DType` - Recommended data type for the given device
pub fn select_dtype_default(device: &Device) -> DType {
    select_dtype(device, &PrecisionConfig::default())
}

/// Converts a PrecisionPolicy to a PrecisionConfig
/// 
/// # Arguments
/// * `policy` - The precision policy to convert
/// 
/// # Returns
/// * `PrecisionConfig` - The corresponding precision configuration
pub fn policy_to_config(policy: &PrecisionPolicy) -> PrecisionConfig {
    match policy {
        PrecisionPolicy::Default => PrecisionConfig::default(),
        PrecisionPolicy::MemoryEfficient => PrecisionConfig::memory_efficient(),
        PrecisionPolicy::MaximumPrecision => PrecisionConfig::maximum_precision(),
    }
}

/// Selects the appropriate dtype based on device and precision policy
/// 
/// # Arguments
/// * `device` - Target device for computation
/// * `policy` - Precision policy to use
/// 
/// # Returns
/// * `DType` - Recommended data type for the given device and policy
pub fn select_dtype_by_policy(device: &Device, policy: &PrecisionPolicy) -> DType {
    let config = policy_to_config(policy);
    select_dtype(device, &config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PrecisionConfig::default();
        assert_eq!(config.cpu_dtype, DType::F32);
        assert_eq!(config.gpu_dtype, DType::BF16);
        assert_eq!(config.allow_override, true);
    }

    #[test]
    fn test_memory_efficient_config() {
        let config = PrecisionConfig::memory_efficient();
        assert_eq!(config.cpu_dtype, DType::F32);
        assert_eq!(config.gpu_dtype, DType::F16);
        assert_eq!(config.allow_override, true);
    }

    #[test]
    fn test_maximum_precision_config() {
        let config = PrecisionConfig::maximum_precision();
        assert_eq!(config.cpu_dtype, DType::F32);
        assert_eq!(config.gpu_dtype, DType::F32);
        assert_eq!(config.allow_override, true);
    }

    #[test]
    fn test_dtype_selection() {
        let cpu_device = Device::Cpu;
        let config = PrecisionConfig::default();
        
        // CPU should use F32
        assert_eq!(select_dtype(&cpu_device, &config), DType::F32);
    }

    #[test]
    #[cfg(feature = "cuda")]
    fn test_cuda_dtype_selection() {
        // This test requires CUDA to be available
        if let Ok(cuda_device) = Device::new_cuda(0) {
            let config = PrecisionConfig::default();
            // CUDA should use BF16
            assert_eq!(select_dtype(&cuda_device, &config), DType::BF16);
        }
    }

    #[test]
    #[cfg(feature = "metal")]
    fn test_metal_dtype_selection() {
        // This test requires Metal to be available
        if let Ok(metal_device) = Device::new_metal(0) {
            let config = PrecisionConfig::default();
            // Metal should use BF16
            assert_eq!(select_dtype(&metal_device, &config), DType::BF16);
        }
    }
}