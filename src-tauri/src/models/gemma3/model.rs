use candle::quantized::gguf_file::Content;
use candle::Device;
use candle::Tensor;
use std::io::{Read, Seek};

/// Wrapper around candle_transformers' quantized Gemma3 implementation
pub struct ModelWeights {
    inner: candle_transformers::models::quantized_gemma3::ModelWeights,
}

impl ModelWeights {
    /// Build from GGUF content and reader. We ignore context-specific knobs
    /// as candle_transformers handles necessary metadata internally.
    pub fn from_gguf<R: Read + Seek>(
        content: Content,
        reader: &mut R,
        device: &Device,
        _context_length: usize,
        _flag: bool,
    ) -> Result<Self, String> {
        let inner = candle_transformers::models::quantized_gemma3::ModelWeights::from_gguf(
            content, reader, device,
        )
        .map_err(|e| e.to_string())?;
        Ok(Self { inner })
    }
}

impl crate::models::common::model::ModelBackend for ModelWeights {
    fn forward_layered(&mut self, input: &Tensor, position: usize) -> Result<Tensor, String> {
        self.inner
            .forward(input, position)
            .map_err(|e| e.to_string())
    }
}
