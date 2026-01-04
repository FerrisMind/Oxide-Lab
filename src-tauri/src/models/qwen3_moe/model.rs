use crate::models::qwen3::model::{
    Config as Qwen3Config, Qwen3Attention, Qwen3MLP, Qwen3RotaryEmbedding,
};
use candle::{D, DType, Device, Module, Result, Tensor};
use candle_nn::{Activation, Linear, VarBuilder, linear_no_bias, moe};
use candle_transformers::models::with_tracing::{
    Linear as TracingLinear, RmsNorm, linear_no_bias as linear_no_bias_tracing,
};
use std::sync::Arc;

fn default_use_flash_attn() -> bool {
    false
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct Config {
    pub vocab_size: usize,
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub num_hidden_layers: usize,
    pub num_attention_heads: usize,
    pub head_dim: usize,
    pub attention_bias: bool,
    pub num_key_value_heads: usize,
    pub max_position_embeddings: usize,
    pub sliding_window: Option<usize>,
    pub max_window_layers: usize,
    pub tie_word_embeddings: bool,
    pub rope_theta: f64,
    pub rms_norm_eps: f64,
    pub use_sliding_window: bool,
    pub hidden_act: Activation,
    #[serde(default = "default_use_flash_attn")]
    pub use_flash_attn: bool,
    // MoE specific configuration
    pub decoder_sparse_step: usize,
    pub moe_intermediate_size: usize,
    pub num_experts_per_tok: usize,
    pub num_experts: usize,
    pub norm_topk_prob: bool,
}

impl From<&Config> for Qwen3Config {
    fn from(val: &Config) -> Self {
        Qwen3Config {
            vocab_size: val.vocab_size,
            hidden_size: val.hidden_size,
            intermediate_size: val.intermediate_size,
            num_hidden_layers: val.num_hidden_layers,
            num_attention_heads: val.num_attention_heads,
            head_dim: val.head_dim,
            attention_bias: val.attention_bias,
            num_key_value_heads: val.num_key_value_heads,
            max_position_embeddings: val.max_position_embeddings,
            sliding_window: val.sliding_window,
            max_window_layers: val.max_window_layers,
            tie_word_embeddings: val.tie_word_embeddings,
            rope_theta: val.rope_theta,
            rms_norm_eps: val.rms_norm_eps,
            use_sliding_window: val.use_sliding_window,
            hidden_act: val.hidden_act,
            use_flash_attn: val.use_flash_attn,
        }
    }
}

// MoE configuration (from candle-transformers fused_moe.rs)
pub struct MoeCfg {
    pub hidden_size: usize,
    pub num_experts: usize,
    pub num_experts_per_tok: usize,
    pub moe_intermediate_size: usize,
    pub norm_topk_prob: bool,
    pub act: Activation,
    #[allow(dead_code)]
    pub decoder_sparse_step: Option<usize>,
}

// FusedMoe implementation (from candle-transformers fused_moe.rs)
#[derive(Debug, Clone)]
pub struct FusedMoe {
    gate: Linear,
    gate_up_w: Tensor,
    down_w: Tensor,
    w_size_n: usize,
    act: Activation,
    norm_topk_prob: bool,
    num_experts_per_tok: usize,
    #[allow(dead_code)]
    dtype: DType,
}

impl FusedMoe {
    pub fn new(cfg: &MoeCfg, vb: VarBuilder, dtype: DType) -> Result<Self> {
        let num_experts = cfg.num_experts;

        let gate = linear_no_bias(cfg.hidden_size, num_experts, vb.pp("gate"))?;

        let experts_vb = vb.pp("experts");
        let mut gate_up_experts = Vec::with_capacity(num_experts);
        let mut down_experts = Vec::with_capacity(num_experts);

        // Pack experts
        for i in 0..num_experts {
            let experts_vb = experts_vb.pp(format!("{i}").as_str());

            let (gate_up_expert, down_expert) = {
                // n x k format
                let init_ws = candle_nn::init::DEFAULT_KAIMING_NORMAL;
                let gate_expert = experts_vb.pp("gate_proj").get_with_hints(
                    (cfg.moe_intermediate_size, cfg.hidden_size),
                    "weight",
                    init_ws,
                )?;
                let up_expert = experts_vb.pp("up_proj").get_with_hints(
                    (cfg.moe_intermediate_size, cfg.hidden_size),
                    "weight",
                    init_ws,
                )?;
                let down_expert = experts_vb.pp("down_proj").get_with_hints(
                    (cfg.hidden_size, cfg.moe_intermediate_size),
                    "weight",
                    init_ws,
                )?;
                // Pack gate_proj and up_proj
                let gate_up_expert = Tensor::cat(&[&gate_expert, &up_expert], 0)?;

                (gate_up_expert, down_expert)
            };

            gate_up_experts.push(gate_up_expert);
            down_experts.push(down_expert);
        }

        let gate_up_w = Tensor::stack(&gate_up_experts, 0)?;
        let down_w = Tensor::stack(&down_experts, 0)?;
        let w_size_n = gate_up_w.dim(1)? / 2;

        Ok(Self {
            gate,
            gate_up_w,
            down_w,
            w_size_n,
            act: cfg.act,
            norm_topk_prob: cfg.norm_topk_prob,
            num_experts_per_tok: cfg.num_experts_per_tok,
            dtype,
        })
    }

    pub fn forward(&self, xs: &Tensor, is_prefill: bool) -> Result<Tensor> {
        let (batch, seq_len, hidden_dim) = xs.dims3()?;
        let xs = xs.reshape(((), hidden_dim))?;
        let (num_tokens, hidden_dim) = xs.dims2()?;

        let router_logits = self.gate.forward(&xs)?;

        let routing_weights =
            candle_nn::ops::softmax_last_dim(&router_logits.to_dtype(DType::F32)?)?;

        let topk_ids = routing_weights
            .arg_sort_last_dim(false)?
            .narrow(D::Minus1, 0, self.num_experts_per_tok)?
            .contiguous()?;

        let mut topk_weights = routing_weights.gather(&topk_ids, D::Minus1)?;

        if self.norm_topk_prob {
            topk_weights = topk_weights.broadcast_div(&topk_weights.sum_keepdim(D::Minus1)?)?;
        }

        // Note: For long-context (32K+), could use custom CUDA sort kernel
        // but for now we use the same approach for both prefill and decode
        let _ = is_prefill; // Silence unused warning, kept for future optimization
        let (expert_ids, sorted_token_ids) = topk_ids.flatten_all()?.sort_last_dim(true)?;

        // out (M, top_k, N)
        let gate_up = moe::moe_gemm(
            &xs,
            &self.gate_up_w,
            &None,
            &sorted_token_ids,
            &expert_ids,
            self.num_experts_per_tok,
            is_prefill,
        )?;

        let gate = gate_up
            .narrow(candle::D::Minus1, 0, self.w_size_n)?
            .contiguous()?;
        let up = gate_up
            .narrow(candle::D::Minus1, self.w_size_n, self.w_size_n)?
            .contiguous()?;

        // (M * top_k, N // 2)
        let down_inputs = (up * gate.apply(&self.act)?)?.reshape(((), self.w_size_n))?;

        // view(M, top_k, K) -> sum -> (M, K)
        let ys = moe::moe_gemm(
            &down_inputs,
            &self.down_w,
            &Some(topk_weights),
            &sorted_token_ids,
            &expert_ids,
            self.num_experts_per_tok,
            is_prefill,
        )?
        .reshape((num_tokens, (), hidden_dim))?
        .sum(D::Minus2)?;

        ys.reshape((batch, seq_len, hidden_dim))
    }
}

#[derive(Debug, Clone)]
struct Qwen3MLPExpert {
    gate_proj: TracingLinear,
    up_proj: TracingLinear,
    down_proj: TracingLinear,
    act_fn: Activation,
}

impl Qwen3MLPExpert {
    fn new(cfg: &Config, vb: VarBuilder) -> Result<Self> {
        Ok(Self {
            gate_proj: linear_no_bias_tracing(
                cfg.hidden_size,
                cfg.moe_intermediate_size,
                vb.pp("gate_proj"),
            )?,
            up_proj: linear_no_bias_tracing(
                cfg.hidden_size,
                cfg.moe_intermediate_size,
                vb.pp("up_proj"),
            )?,
            down_proj: linear_no_bias_tracing(
                cfg.moe_intermediate_size,
                cfg.hidden_size,
                vb.pp("down_proj"),
            )?,
            act_fn: cfg.hidden_act,
        })
    }
}

impl Module for Qwen3MLPExpert {
    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let lhs = x.apply(&self.gate_proj)?.apply(&self.act_fn)?;
        let rhs = x.apply(&self.up_proj)?;
        (lhs * rhs)?.apply(&self.down_proj)
    }
}

// Qwen3 Sparse MoE Block implementation (naive, for CPU)
#[derive(Debug, Clone)]
struct Qwen3SparseMoeBlock {
    gate: TracingLinear,
    experts: Vec<Qwen3MLPExpert>,
    norm_topk_prob: bool,
    num_experts_per_tok: usize,
}

impl Qwen3SparseMoeBlock {
    fn new(cfg: &Config, vb: VarBuilder) -> Result<Self> {
        let gate = linear_no_bias_tracing(cfg.hidden_size, cfg.num_experts, vb.pp("gate"))?;
        let mut experts = Vec::with_capacity(cfg.num_experts);
        let vb_e = vb.pp("experts");
        for idx in 0..cfg.num_experts {
            let expert = Qwen3MLPExpert::new(cfg, vb_e.pp(idx))?;
            experts.push(expert)
        }
        Ok(Self {
            gate,
            experts,
            norm_topk_prob: cfg.norm_topk_prob,
            num_experts_per_tok: cfg.num_experts_per_tok,
        })
    }
}

impl Module for Qwen3SparseMoeBlock {
    fn forward(&self, xs: &Tensor) -> Result<Tensor> {
        let (b_size, seq_len, hidden_dim) = xs.dims3()?;
        let xs = xs.reshape(((), hidden_dim))?;
        let router_logits = xs.apply(&self.gate)?;
        let routing_weights = candle_nn::ops::softmax_last_dim(&router_logits)?;

        // Extract topk experts per token
        let experts_per_tok = routing_weights
            .arg_sort_last_dim(false)?
            .narrow(D::Minus1, 0, self.num_experts_per_tok)?
            .contiguous()?;
        let routing_weights = routing_weights.gather(&experts_per_tok, D::Minus1)?;

        // Extract needed data
        let routing_weights = routing_weights.to_dtype(DType::F32)?.to_vec2::<f32>()?;
        let experts_per_tok = experts_per_tok.to_vec2::<u32>()?;
        let mut top_x = vec![vec![]; self.experts.len()];
        let mut selected_experts = vec![vec![]; self.experts.len()];
        for (row_idx, (rw, expert_idxs)) in routing_weights
            .iter()
            .zip(experts_per_tok.iter())
            .enumerate()
        {
            let sum_rw = rw.iter().sum::<f32>();
            for (&rw, &expert_idx) in rw.iter().zip(expert_idxs.iter()) {
                top_x[expert_idx as usize].push(row_idx as u32);
                let rw = if self.norm_topk_prob { rw / sum_rw } else { rw };
                selected_experts[expert_idx as usize].push(rw)
            }
        }

        // Process through experts
        let mut ys = xs.zeros_like()?;
        for (expert_idx, expert_layer) in self.experts.iter().enumerate() {
            let top_x = &top_x[expert_idx];
            if top_x.is_empty() {
                continue;
            }
            let top_x = Tensor::new(top_x.as_slice(), xs.device())?;
            let selected_experts =
                Tensor::new(selected_experts[expert_idx].as_slice(), xs.device())?
                    .reshape(((), 1))?
                    .to_dtype(xs.dtype())?;

            let current_state = xs.index_select(&top_x, 0)?.reshape(((), hidden_dim))?;
            let current_hidden_states = expert_layer.forward(&current_state)?;
            let current_hidden_states = current_hidden_states.broadcast_mul(&selected_experts)?;
            ys = ys.index_add(&top_x, &current_hidden_states, 0)?;
        }

        ys.reshape((b_size, seq_len, hidden_dim))
    }
}

// MLP or MoE decision enum
#[derive(Debug, Clone)]
enum Qwen3FeedForward {
    Mlp(Qwen3MLP),
    NaiveMoE(Qwen3SparseMoeBlock),
    FusedMoE(FusedMoe),
}

impl Qwen3FeedForward {
    fn forward(&self, xs: &Tensor, is_prefill: bool) -> Result<Tensor> {
        match self {
            Self::Mlp(m) => m.forward(xs),
            Self::NaiveMoE(m) => m.forward(xs),
            Self::FusedMoE(m) => m.forward(xs, is_prefill),
        }
    }
}

#[derive(Debug, Clone)]
struct DecoderLayer {
    self_attn: Qwen3Attention,
    feed_forward: Qwen3FeedForward,
    ln1: RmsNorm,
    ln2: RmsNorm,
}

impl DecoderLayer {
    fn new(
        layer_idx: usize,
        cfg: &Config,
        rotary: Arc<Qwen3RotaryEmbedding>,
        vb: VarBuilder,
    ) -> Result<Self> {
        let self_attn = Qwen3Attention::new(&cfg.into(), rotary, vb.pp("self_attn"))?;

        let moe_cfg = MoeCfg {
            hidden_size: cfg.hidden_size,
            num_experts: cfg.num_experts,
            num_experts_per_tok: cfg.num_experts_per_tok,
            moe_intermediate_size: cfg.moe_intermediate_size,
            norm_topk_prob: cfg.norm_topk_prob,
            act: cfg.hidden_act,
            decoder_sparse_step: None,
        };

        // Decide whether to use MoE or regular MLP based on layer_idx and decoder_sparse_step
        let feed_forward =
            if cfg.num_experts > 0 && (layer_idx + 1).is_multiple_of(cfg.decoder_sparse_step) {
                if cfg!(feature = "cuda") {
                    // Use fused MoE kernel on CUDA
                    Qwen3FeedForward::FusedMoE(FusedMoe::new(&moe_cfg, vb.pp("mlp"), vb.dtype())?)
                } else {
                    Qwen3FeedForward::NaiveMoE(Qwen3SparseMoeBlock::new(cfg, vb.pp("mlp"))?)
                }
            } else {
                Qwen3FeedForward::Mlp(Qwen3MLP::new(&cfg.into(), vb.pp("mlp"))?)
            };

        let ln1 = RmsNorm::new(cfg.hidden_size, cfg.rms_norm_eps, vb.pp("input_layernorm"))?;
        let ln2 = RmsNorm::new(
            cfg.hidden_size,
            cfg.rms_norm_eps,
            vb.pp("post_attention_layernorm"),
        )?;

        Ok(Self {
            self_attn,
            feed_forward,
            ln1,
            ln2,
        })
    }

    fn forward(&mut self, x: &Tensor, mask: Option<&Tensor>, offset: usize) -> Result<Tensor> {
        let h = self.ln1.forward(x)?;
        let h = self.self_attn.forward(&h, mask, offset)?;
        let x = (x + h)?;
        let h2 = self.ln2.forward(&x)?;
        let h2 = self.feed_forward.forward(&h2, mask.is_some())?;
        x + h2
    }

    fn clear_kv_cache(&mut self) {
        self.self_attn.clear_kv_cache();
    }
}

#[derive(Debug, Clone)]
pub struct Model {
    embed_tokens: candle_nn::Embedding,
    layers: Vec<DecoderLayer>,
    norm: RmsNorm,
    device: Device,
    dtype: DType,
}

impl Model {
    pub fn new(cfg: &Config, vb: VarBuilder) -> Result<Self> {
        let embed_tokens =
            candle_nn::embedding(cfg.vocab_size, cfg.hidden_size, vb.pp("model.embed_tokens"))?;
        let rotary = Arc::new(Qwen3RotaryEmbedding::new(
            vb.dtype(),
            &cfg.into(),
            vb.device(),
        )?);
        let mut layers = Vec::with_capacity(cfg.num_hidden_layers);
        let vb_l = vb.pp("model.layers");
        for i in 0..cfg.num_hidden_layers {
            layers.push(DecoderLayer::new(i, cfg, rotary.clone(), vb_l.pp(i))?);
        }
        Ok(Self {
            embed_tokens,
            layers,
            norm: RmsNorm::new(cfg.hidden_size, cfg.rms_norm_eps, vb.pp("model.norm"))?,
            device: vb.device().clone(),
            dtype: vb.dtype(),
        })
    }

    fn clear_kv_cache(&mut self) {
        for l in &mut self.layers {
            l.clear_kv_cache();
        }
    }

    fn causal_mask(
        &self,
        b: usize,
        tgt: usize,
        offset: usize,
        sw: Option<usize>,
    ) -> Result<Tensor> {
        let minf = f32::NEG_INFINITY;
        let mask: Vec<_> = (0..tgt)
            .flat_map(|i| {
                (0..(tgt + offset)).map(move |j| {
                    let past_ok = j <= i + offset;
                    let sw_ok = match sw {
                        Some(w) => (i + offset) as i64 - j as i64 <= w as i64,
                        None => true,
                    };
                    if past_ok && sw_ok { 0. } else { minf }
                })
            })
            .collect();
        Tensor::from_slice(&mask, (b, 1, tgt, tgt + offset), &self.device)?.to_dtype(self.dtype)
    }

    pub fn forward(&mut self, input: &Tensor, offset: usize) -> Result<Tensor> {
        let (b, l) = input.dims2()?;
        let mut h = self.embed_tokens.forward(input)?;

        let causal = if l == 1 {
            None
        } else {
            Some(self.causal_mask(b, l, offset, None)?)
        };

        for layer in &mut self.layers {
            h = layer.forward(&h, causal.as_ref(), offset)?;
        }
        self.norm.forward(&h)
    }
}

#[derive(Debug, Clone)]
pub struct ModelForCausalLM {
    base: Model,
    lm_head: TracingLinear,
}

impl ModelForCausalLM {
    pub fn new(cfg: &Config, vb: VarBuilder) -> Result<Self> {
        let base = Model::new(cfg, vb.clone())?;
        let lm_head = if cfg.tie_word_embeddings {
            TracingLinear::from_weights(base.embed_tokens.embeddings().clone(), None)
        } else {
            linear_no_bias_tracing(cfg.hidden_size, cfg.vocab_size, vb.pp("lm_head"))?
        };
        Ok(Self { base, lm_head })
    }

    pub fn forward(&mut self, input: &Tensor, offset: usize) -> Result<Tensor> {
        let (_, l) = input.dims2()?;
        self.base
            .forward(input, offset)?
            .narrow(1, l - 1, 1)?
            .apply(&self.lm_head)
    }

    pub fn clear_kv_cache(&mut self) {
        self.base.clear_kv_cache();
    }
}
