use candle::Device;
use candle::Tensor;
use candle::quantized::gguf_file::Content;
use std::io::{Read, Seek};

/// Wrapper around candle_transformers' quantized Qwen3 implementation
pub struct ModelWeights {
    inner: candle_transformers::models::quantized_qwen3::ModelWeights,
}

impl ModelWeights {
    /// Build from GGUF content and reader. We ignore `context_length` and `flag`
    /// as candle_transformers handles necessary metadata internally.
    pub fn from_gguf<R: Read + Seek>(
        content: Content,
        reader: &mut R,
        device: &Device,
        _context_length: usize,
        _flag: bool,
    ) -> Result<Self, String> {
        let cw = candle_transformers::models::quantized_qwen3::ModelWeights::from_gguf(
            content, reader, device,
        )
        .map_err(|e| {
            let error_msg = e.to_string();
            if error_msg.contains("unknown dtype") {
                format!("{} - This may be due to unsupported quantization types in the GGUF file. Consider using a different quantization or updating Candle.", error_msg)
            } else {
                error_msg
            }
        })?;
        Ok(ModelWeights { inner: cw })
    }
}

impl crate::models::common::model::ModelBackend for ModelWeights {
    fn forward_layered(&mut self, input: &Tensor, position: usize) -> Result<Tensor, String> {
        self.inner
            .forward(input, position)
            .map_err(|e| e.to_string())
    }

    fn clear_kv_cache(&mut self) {
        self.inner.clear_kv_cache();
    }
}
