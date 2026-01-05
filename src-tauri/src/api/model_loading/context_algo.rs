use candle::Device;

/// Constants for memory estimation
const VRAM_HEADROOM_MB: usize = 1024; // 1GB reserved safety buffer
const WHISPER_RESERVE_MB: usize = 384; // Reserve for Whisper/STT (~300MB for small models + buffers)
const BYTES_PER_MB: usize = 1024 * 1024;

/// Parameters required to calculate KV cache size
pub struct ModelCacheParams {
    pub n_layer: usize,
    pub n_kv_head: usize,
    pub head_dim: usize,
    pub dtype_size: usize, // e.g. 2 for f16, 4 for f32
}

impl ModelCacheParams {
    /// Calculate the memory required for a specific context length (in bytes)
    pub fn memory_required(&self, ctx_len: usize) -> usize {
        // KV Cache = 2 (K+V) * n_layer * ctx_len * n_kv_head * head_dim * dtype_size
        2 * self.n_layer * ctx_len * self.n_kv_head * self.head_dim * self.dtype_size
    }
}

/// Helper to get available VRAM (approximate).
/// This depends on the device backend.
pub fn get_available_vram(device: &Device) -> Result<usize, String> {
    match device {
        Device::Cuda(_dev) => {
            // candle::CudaDevice provides memory info
            // let (free, _total) = dev.mem_get_info().map_err(|e| e.to_string())?;
            // Ok(free)
            Ok(0) // Fallback to probe-only for now due to API issue
        }
        Device::Metal(_) => {
            // Metal API placeholder
            Err("Metal VRAM query not implemented".into())
        }
        Device::Cpu => {
            // CPU placeholder
            Err("CPU RAM query not implemented".into())
        } // _ => Err("Unsupported device".into()),
    }
}

/// Probes if a given context length can actually be allocated.
/// Returns true if successful.
pub fn probe_allocation(device: &Device, params: &ModelCacheParams, ctx_len: usize) -> bool {
    let required_bytes = params.memory_required(ctx_len);

    // Safety check: don't probe ridiculous sizes > 100GB
    if required_bytes > 100 * 1024 * 1024 * 1024 {
        return false;
    }

    // Use u8 to have exact byte control
    candle::Tensor::zeros(required_bytes, candle::DType::U8, device).is_ok()
}

/// Main algorithm to find the best context length
pub fn estimate_best_context(
    device: &Device,
    params: &ModelCacheParams,
    candidates: &[usize], // e.g. [4096, 8192, 16384, 32768, 65536]
) -> usize {
    let free_vram = get_available_vram(device).unwrap_or(0);
    // Use the first candidate as the absolute fallback
    let mut best_ctx = candidates.first().copied().unwrap_or(4096);

    // We assume candidates are sorted Ascending
    for &ctx in candidates {
        let req = params.memory_required(ctx);

        // 1. Theoretical Check
        if free_vram > 0 && req + (VRAM_HEADROOM_MB + WHISPER_RESERVE_MB) * BYTES_PER_MB > free_vram {
            // If this candidate fails theory, larger ones will too.
            break;
        }

        // 2. Allocation Probe Check
        // 2. Allocation Probe Check
        // Only probe if it passed theoretical or if we don't know VRAM
        let is_allocation_successful = probe_allocation(device, params, ctx);

        let mb = req / BYTES_PER_MB;
        if is_allocation_successful {
            log::info!("Autotune probe: ctx={} ({} MB) -> OK", ctx, mb);
            best_ctx = ctx;
        } else {
            log::warn!("Autotune probe: ctx={} ({} MB) -> OOM / Failed", ctx, mb);
            // Probe failed. Since we assume linear growth, stop.
            break;
        }
    }

    best_ctx
}
