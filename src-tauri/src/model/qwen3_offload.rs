use candle::{DType, Device, Result, Tensor};
use candle::quantized::gguf_file;
use candle_nn::{Module};
use std::io::{Read, Seek};
use std::sync::Arc;

// Минимально вендоренная версия квантованной Qwen3 с доступом к слоям для послойного offload.
// ВНИМАНИЕ: это упрощённая версия, ориентированная на forward; покрывает только нужные нам пути.

use candle_transformers::models::with_tracing::QMatMul as TracedQMatMul;
use candle_transformers::quantized_nn::{RmsNorm as QRmsNorm};
use candle_nn as nn;
use candle_transformers::utils;
#[cfg(feature = "flash-attn")]
use candle_flash_attn as _;
#[derive(Debug, Clone)]
pub struct EmbeddingWrap { inner: nn::Embedding }
impl EmbeddingWrap { pub fn from_weight(weight: Tensor, out: usize) -> Result<Self> { Ok(Self { inner: nn::Embedding::new(weight, out) }) } }
impl nn::Module for EmbeddingWrap { fn forward(&self, xs: &Tensor) -> Result<Tensor> { self.inner.forward(xs) } }

#[derive(Debug, Clone)]
struct RotaryEmbedding {
    sin: Tensor,
    cos: Tensor,
}

impl RotaryEmbedding {
    fn new(dtype: DType, head_dim: usize, max_pos: usize, rope_theta: f64, dev: &Device) -> Result<Self> {
        let inv_freq: Vec<_> = (0..head_dim)
            .step_by(2)
            .map(|i| 1f32 / rope_theta.powf(i as f64 / head_dim as f64) as f32)
            .collect();
        let inv_freq_len = inv_freq.len();
        let inv = Tensor::from_vec(inv_freq, (1, inv_freq_len), dev)?.to_dtype(dtype)?;
        let t = Tensor::arange(0u32, max_pos as u32, dev)?.to_dtype(dtype)?.reshape((max_pos, 1))?;
        let freqs = t.matmul(&inv)?;
        Ok(Self { sin: freqs.sin()?, cos: freqs.cos()? })
    }
    fn apply(&self, q: &Tensor, k: &Tensor, offset: usize) -> Result<(Tensor, Tensor)> {
        let (_, _, seq_len, _) = q.dims4()?;
        // Переносим sin/cos на устройство входа, чтобы избежать конфликтов устройств при offload
        let cos = self
            .cos
            .narrow(0, offset, seq_len)?
            .to_dtype(q.dtype())?
            .to_device(q.device())?;
        let sin = self
            .sin
            .narrow(0, offset, seq_len)?
            .to_dtype(q.dtype())?
            .to_device(q.device())?;
        let q_embed = candle_nn::rotary_emb::rope(&q.contiguous()?, &cos, &sin)?;
        let k_embed = candle_nn::rotary_emb::rope(&k.contiguous()?, &cos, &sin)?;
        Ok((q_embed, k_embed))
    }
}

#[derive(Debug, Clone)]
struct AttentionWeights {
    q_proj: TracedQMatMul,
    k_proj: TracedQMatMul,
    v_proj: TracedQMatMul,
    o_proj: TracedQMatMul,
    q_norm: QRmsNorm,
    k_norm: QRmsNorm,
    num_heads: usize,
    num_kv_heads: usize,
    num_kv_groups: usize,
    head_dim: usize,
    rotary: Arc<RotaryEmbedding>,
    kv_cache: candle_nn::kv_cache::KvCache,
    use_flash_attn: bool,
}

impl AttentionWeights {
    fn new<R: Read + Seek>(
        gg: &mut Gguf<R>,
        nh: usize,
        n_kv: usize,
        hd: usize,
        eps: f64,
        rotary: Arc<RotaryEmbedding>,
        device: &Device,
        prefix: &str,
    ) -> Result<Self> {
        let q_proj = gg.qmatmul_on(&format!("{prefix}.attn_q.weight"), device)?;
        let k_proj = gg.qmatmul_on(&format!("{prefix}.attn_k.weight"), device)?;
        let v_proj = gg.qmatmul_on(&format!("{prefix}.attn_v.weight"), device)?;
        let o_proj = gg.qmatmul_on(&format!("{prefix}.attn_output.weight"), device)?;
        let q_norm = gg.rms_norm_on(&format!("{prefix}.attn_q_norm.weight"), eps, device)?;
        let k_norm = gg.rms_norm_on(&format!("{prefix}.attn_k_norm.weight"), eps, device)?;
        // Размер кэша будет установлен сверху через LayerWeights::new
        let kv_cache = candle_nn::kv_cache::KvCache::new(2, 1);
        Ok(Self { q_proj, k_proj, v_proj, o_proj, q_norm, k_norm, num_heads: nh, num_kv_heads: n_kv, num_kv_groups: nh / n_kv, head_dim: hd, rotary, kv_cache, use_flash_attn: false })
    }
    fn forward(&mut self, x: &Tensor, attn_mask: Option<&Tensor>, offset: usize) -> Result<Tensor> {
        let (b, l, _) = x.dims3()?;
        let q = self.q_proj.forward(x)?;
        let k = self.k_proj.forward(x)?;
        let v = self.v_proj.forward(x)?;
        let q = q.reshape((b, l, self.num_heads, self.head_dim))?.transpose(1, 2)?;
        let k = k.reshape((b, l, self.num_kv_heads, self.head_dim))?.transpose(1, 2)?;
        let v = v.reshape((b, l, self.num_kv_heads, self.head_dim))?.transpose(1, 2)?;
        let q_flat = self.q_norm.forward(&q.flatten(0, 2)?)?;
        let k_flat = self.k_norm.forward(&k.flatten(0, 2)?)?;
        let q = q_flat.reshape((b, self.num_heads, l, self.head_dim))?;
        let k = k_flat.reshape((b, self.num_kv_heads, l, self.head_dim))?;
        let (q, k) = self.rotary.apply(&q, &k, offset)?;
        if offset == 0 { self.kv_cache.reset(); }
        let (k, v) = self.kv_cache.append(&k.contiguous()?, &v.contiguous()?)?;
        let k = k.contiguous()?; let v = v.contiguous()?;
        let k = utils::repeat_kv(k, self.num_kv_groups)?.contiguous()?;
        let v = utils::repeat_kv(v, self.num_kv_groups)?.contiguous()?;
        let scale = 1.0 / (self.head_dim as f64).sqrt();
        let mut scores = (q.matmul(&k.transpose(2, 3)?)? * scale)?;
        if let Some(m) = attn_mask { let m = if m.dtype() != scores.dtype() { m.to_dtype(scores.dtype())? } else { m.clone() }; scores = scores.broadcast_add(&m)?; }
        let probs = candle_nn::ops::softmax_last_dim(&scores)?;
        let ctx = probs.matmul(&v)?;
        let reshaped_ctx = ctx.transpose(1, 2)?.reshape((b, l, self.num_heads * self.head_dim))?;
        self.o_proj.forward(&reshaped_ctx)
    }
}

#[derive(Debug, Clone)]
struct LayerWeights {
    self_attn: AttentionWeights,
    mlp: MlpWeights,
    ln1: QRmsNorm,
    ln2: QRmsNorm,
    device: Device,
}

impl LayerWeights {
    fn new<R: Read + Seek>(
        gg: &mut Gguf<R>,
        nh: usize,
        n_kv: usize,
        hd: usize,
        eps: f64,
        rotary: Arc<RotaryEmbedding>,
        device: &Device,
        kv_len: usize,
        idx: usize,
    ) -> Result<Self> {
        let prefix = format!("blk.{idx}");
        let ln1 = gg.rms_norm_on(&format!("{prefix}.attn_norm.weight"), eps, device)?;
        let ln2 = gg.rms_norm_on(&format!("{prefix}.ffn_norm.weight"), eps, device)?;
        let mut self_attn = AttentionWeights::new(gg, nh, n_kv, hd, eps, rotary, device, &prefix)?;
        // Переинициализируем kv_cache под выбранную длину контекста
        self_attn.kv_cache = candle_nn::kv_cache::KvCache::new(2, kv_len);
        let mlp = MlpWeights::new_on(gg, device, &prefix)?;
        Ok(Self { self_attn, mlp, ln1, ln2, device: device.clone() })
    }
    fn forward(&mut self, x: &Tensor, mask: Option<&Tensor>, offset: usize) -> Result<Tensor> {
        let h = self.ln1.forward(x)?;
        let h = self.self_attn.forward(&h, mask, offset)?;
        let x = (x + h)?;
        let h2 = self.ln2.forward(&x)?;
        let h2 = h2.apply(&self.mlp)?;
        Ok((x + h2)?)
    }
}

#[derive(Debug, Clone)]
struct MlpWeights {
    gate_proj: TracedQMatMul,
    up_proj: TracedQMatMul,
    down_proj: TracedQMatMul,
}

impl MlpWeights {
    fn new_on<R: Read + Seek>(gg: &mut Gguf<R>, device: &Device, prefix: &str) -> Result<Self> {
        let gate_proj = gg.qmatmul_on(&format!("{prefix}.ffn_gate.weight"), device)?;
        let up_proj = gg.qmatmul_on(&format!("{prefix}.ffn_up.weight"), device)?;
        let down_proj = gg.qmatmul_on(&format!("{prefix}.ffn_down.weight"), device)?;
        Ok(Self { gate_proj, up_proj, down_proj })
    }
}

impl Module for MlpWeights {
    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let gate = self.gate_proj.forward(x)?.apply(&candle_nn::Activation::Silu)?;
        let up = self.up_proj.forward(x)?;
        let gated = (gate * up)?;
        self.down_proj.forward(&gated)
    }
}

#[derive(Debug, Clone)]
pub struct ModelWeights {
    pub embed: EmbeddingWrap,
    pub layers: Vec<LayerWeights>,
    pub norm: QRmsNorm,
    pub lm_head: TracedQMatMul,
    pub device: Device,
    pub dtype: DType,
    pub use_flash_attn: bool,
}

struct Gguf<R: Read + Seek> {
    ct: gguf_file::Content,
    reader: R,
    device: Device,
}
impl<R: Read + Seek> Gguf<R> {
    fn new(ct: gguf_file::Content, reader: R, device: Device) -> Self { Self { ct, reader, device } }
    fn qmatmul(&mut self, name: &str) -> Result<TracedQMatMul> {
        let ws = self.ct.tensor(&mut self.reader, name, &self.device)?;
        TracedQMatMul::from_weights(ws.into())
    }
    fn qmatmul_on(&mut self, name: &str, device: &Device) -> Result<TracedQMatMul> {
        let ws = self.ct.tensor(&mut self.reader, name, device)?;
        TracedQMatMul::from_weights(ws.into())
    }
    fn rms_norm(&mut self, name: &str, eps: f64) -> Result<QRmsNorm> {
        let ws = self.ct.tensor(&mut self.reader, name, &self.device)?;
        QRmsNorm::from_qtensor(ws, eps)
    }
    fn rms_norm_on(&mut self, name: &str, eps: f64, device: &Device) -> Result<QRmsNorm> {
        let ws = self.ct.tensor(&mut self.reader, name, device)?;
        QRmsNorm::from_qtensor(ws, eps)
    }
    fn tensor(&mut self, name: &str) -> Result<candle::quantized::QTensor> {
        self.ct.tensor(&mut self.reader, name, &self.device)
    }
    fn tensor_on(&mut self, name: &str, device: &Device) -> Result<candle::quantized::QTensor> {
        self.ct.tensor(&mut self.reader, name, device)
    }
    fn metadata(&self) -> &std::collections::HashMap<String, gguf_file::Value> { &self.ct.metadata }
}

impl ModelWeights {
    pub fn from_gguf<R: Read + Seek>(
        ct: gguf_file::Content,
        reader: &mut R,
        gpu: &Device,
        cpu: &Device,
        kv_len: usize,
        use_flash_attn: bool,
    ) -> Result<Self> {
        // Инициализируем с любым устройством, не важно: ниже для каждого веса выбираем нужное
        let mut gg = Gguf::new(ct, reader, gpu.clone());
        let md_get = |s: &str| match gg.metadata().get(s) { None => candle::bail!("cannot find {s} in metadata"), Some(v) => Ok(v) };
        let num_heads = md_get("qwen3.attention.head_count")?.to_u32()? as usize;
        let num_kv_heads = md_get("qwen3.attention.head_count_kv")?.to_u32()? as usize;
        let head_dim = md_get("qwen3.attention.key_length")?.to_u32()? as usize;
        let num_layers = md_get("qwen3.block_count")?.to_u32()? as usize;
        let hidden_size = md_get("qwen3.embedding_length")?.to_u32()? as usize;
        let max_position = md_get("qwen3.context_length")?.to_u32()? as usize;
        let eps = md_get("qwen3.attention.layer_norm_rms_epsilon")?.to_f32()? as f64;
        let rope = md_get("qwen3.rope.freq_base")?.to_f32()? as f64;
        let dtype = DType::F16;
        // Грузим embedding: сначала пытаемся на GPU, при OOM — на CPU
        let (embed, mut gpu_remaining) = match gg.tensor_on("token_embd.weight", gpu) {
            Ok(t) => {
                match EmbeddingWrap::from_weight(t.dequantize(gpu)?, hidden_size) { 
                    Ok(e) => (e, true),
                    Err(e) => { let s = e.to_string().to_lowercase(); if s.contains("out of memory") { let t2 = gg.tensor_on("token_embd.weight", cpu)?; (EmbeddingWrap::from_weight(t2.dequantize(cpu)?, hidden_size)?, false) } else { return Err(e) } }
                }
            }
            Err(e) => { let s = e.to_string().to_lowercase(); if s.contains("out of memory") { let t2 = gg.tensor_on("token_embd.weight", cpu)?; (EmbeddingWrap::from_weight(t2.dequantize(cpu)?, hidden_size)?, false) } else { return Err(e) } }
        };
        let rotary = Arc::new(RotaryEmbedding::new(dtype, head_dim, max_position, rope, gpu)?);
        let mut layers = Vec::with_capacity(num_layers);
        let mut use_gpu = gpu_remaining;
        for i in 0..num_layers {
            let preferred = if use_gpu { gpu } else { cpu };
            match LayerWeights::new(&mut gg, num_heads, num_kv_heads, head_dim, eps, rotary.clone(), preferred, kv_len, i) {
                Ok(l) => layers.push(l),
                Err(e) => {
                    let s = e.to_string().to_lowercase();
                    if use_gpu && s.contains("out of memory") {
                        // Падает из‑за OOM на GPU — создаём слой на CPU и фиксируем дальнейшие слои на CPU
                        use_gpu = false;
                        let l = LayerWeights::new(&mut gg, num_heads, num_kv_heads, head_dim, eps, rotary.clone(), cpu, kv_len, i)?;
                        layers.push(l);
                    } else { return Err(e); }
                }
            }
        }
        let last_dev = if use_gpu { gpu } else { cpu };
        let norm = gg.rms_norm_on("output_norm.weight", eps, last_dev)?;
        let lm_head_tensor = match gg.tensor_on("output.weight", last_dev) { Ok(t) => t, Err(_) => gg.tensor_on("token_embd.weight", last_dev)? };
        let lm_head = TracedQMatMul::from_weights(lm_head_tensor.into())?;
        // Проставим флаг flash-attn во все attention-слои
        if use_flash_attn {
            for l in layers.iter_mut() { l.self_attn.use_flash_attn = true; }
        }
        Ok(Self { embed, layers, norm, lm_head, device: gpu.clone(), dtype, use_flash_attn })
    }
    pub fn causal_mask(&self, b: usize, tgt: usize, offset: usize) -> Result<Tensor> {
        let minf = f32::NEG_INFINITY;
        let mask: Vec<_> = (0..tgt).flat_map(|i| (0..(tgt + offset)).map(move |j| if j <= i + offset { 0. } else { minf })).collect();
        Tensor::from_slice(&mask, (b, 1, tgt, tgt + offset), &self.device)?.to_dtype(self.dtype)
    }
    pub fn forward_layered(&mut self, input: &Tensor, offset: usize) -> Result<Tensor> {
        let (b, l) = input.dims2()?;
        let mut h = self.embed.forward(input)?;
        let mask = if l == 1 { None } else { Some(self.causal_mask(b, l, offset)?) };
        for layer in self.layers.iter_mut() {
            let target = &layer.device;
            h = h.to_device(target)?;
            let ln1h = layer.ln1.forward(&h)?;
            let attn = {
                #[cfg(feature = "flash-attn")]
                {
                    if layer.self_attn.use_flash_attn && matches!(target, Device::Cuda(_)) {
                        // Путь flash-attn в bf16
                        let (b_sz, seq_len, _) = ln1h.dims3()?;
                        let q = layer.self_attn.q_proj.forward(&ln1h)?;
                        let k = layer.self_attn.k_proj.forward(&ln1h)?;
                        let v = layer.self_attn.v_proj.forward(&ln1h)?;
                        let q = q.reshape((b_sz, seq_len, layer.self_attn.num_heads, layer.self_attn.head_dim))?;
                        let k = k.reshape((b_sz, seq_len, layer.self_attn.num_kv_heads, layer.self_attn.head_dim))?;
                        let v = v.reshape((b_sz, seq_len, layer.self_attn.num_kv_heads, layer.self_attn.head_dim))?;
                        // нормализации по q/k
                        let q = layer.self_attn.q_norm.forward(&q.flatten(0, 2)?)?.reshape((b_sz, seq_len, layer.self_attn.num_heads, layer.self_attn.head_dim))?;
                        let k = layer.self_attn.k_norm.forward(&k.flatten(0, 2)?)?.reshape((b_sz, seq_len, layer.self_attn.num_kv_heads, layer.self_attn.head_dim))?;
                        let (q, k) = layer.self_attn.rotary.apply(&q.transpose(1,2)?, &k.transpose(1,2)?, offset)?; // (b, heads, seq, dim)
                        if offset == 0 { layer.self_attn.kv_cache.reset(); }
                        let (k_cache, v_cache) = layer.self_attn.kv_cache.append(&k.contiguous()?, &v.transpose(1,2)?.contiguous()?)?;
                        let k_rep = candle_transformers::utils::repeat_kv(k_cache.contiguous()?, layer.self_attn.num_kv_groups)?;
                        let v_rep = candle_transformers::utils::repeat_kv(v_cache.contiguous()?, layer.self_attn.num_kv_groups)?;
                        let softmax_scale = 1.0f32 / (layer.self_attn.head_dim as f32).sqrt();
                        let q = q.to_dtype(DType::BF16)?; let k = k_rep.to_dtype(DType::BF16)?; let v = v_rep.to_dtype(DType::BF16)?;
                        let y = candle_flash_attn::flash_attn(&q.transpose(1,2)?, &k.transpose(1,2)?, &v.transpose(1,2)?, softmax_scale, seq_len > 1)?
                            .to_dtype(DType::F32)?
                            .transpose(1,2)?; // (b, seq, heads, dim)
                        let y = y.reshape((b_sz, seq_len, layer.self_attn.num_heads * layer.self_attn.head_dim))?;
                        layer.self_attn.o_proj.forward(&y)?
                    } else {
                        layer.self_attn.forward(&ln1h, mask.as_ref(), offset)?
                    }
                }
                #[cfg(not(feature = "flash-attn"))]
                {
                    layer.self_attn.forward(&ln1h, mask.as_ref(), offset)?
                }
            };
            let x = (&h + attn)?;
            let h2 = layer.ln2.forward(&x)?;
            let h2 = h2.apply(&layer.mlp)?;
            h = (x + h2)?;
        }
        let h = self.norm.forward(&h)?;
        let last_hidden = h.narrow(1, l - 1, 1)?;
        self.lm_head.forward(&last_hidden)?.squeeze(1)
    }
}


