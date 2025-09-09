use candle_transformers::generation::{LogitsProcessor, Sampling};
use crate::core::config::SamplingOptions;

pub fn build_logits_processor_from_options(options: &SamplingOptions) -> (LogitsProcessor, String) {
    let seed = options.effective_seed();
    if options.temperature <= 0.0 {
        return (LogitsProcessor::from_sampling(seed, Sampling::ArgMax), "ArgMax".to_string());
    }
    match (options.top_k, options.top_p) {
        (None, None) => (
            LogitsProcessor::from_sampling(seed, Sampling::All { temperature: options.temperature }),
            format!("All(temp={:.3})", options.temperature),
        ),
        (Some(k), None) => (
            LogitsProcessor::from_sampling(seed, Sampling::TopK { k, temperature: options.temperature }),
            format!("TopK(k={}, temp={:.3})", k, options.temperature),
        ),
        (None, Some(p)) => (
            LogitsProcessor::from_sampling(seed, Sampling::TopP { p, temperature: options.temperature }),
            format!("TopP(p={:.3}, temp={:.3})", p, options.temperature),
        ),
        (Some(k), Some(p)) => (
            LogitsProcessor::from_sampling(seed, Sampling::TopKThenTopP { k, p, temperature: options.temperature }),
            format!("TopKThenTopP(k={}, p={:.3}, temp={:.3})", k, p, options.temperature),
        ),
    }
}

pub fn build_logits_processor(
    temperature: f64,
    top_k: Option<usize>,
    top_p: Option<f64>,
    seed: u64,
) -> (LogitsProcessor, String) {
    if temperature <= 0.0 {
        return (LogitsProcessor::from_sampling(seed, Sampling::ArgMax), "ArgMax".to_string());
    }
    match (top_k, top_p) {
        (None, None) => (
            LogitsProcessor::from_sampling(seed, Sampling::All { temperature }),
            format!("All(temp={:.3})", temperature),
        ),
        (Some(k), None) => (
            LogitsProcessor::from_sampling(seed, Sampling::TopK { k, temperature }),
            format!("TopK(k={}, temp={:.3})", k, temperature),
        ),
        (None, Some(p)) => (
            LogitsProcessor::from_sampling(seed, Sampling::TopP { p, temperature }),
            format!("TopP(p={:.3}, temp={:.3})", p, temperature),
        ),
        (Some(k), Some(p)) => (
            LogitsProcessor::from_sampling(seed, Sampling::TopKThenTopP { k, p, temperature }),
            format!("TopKThenTopP(k={}, p={:.3}, temp={:.3})", k, p, temperature),
        ),
    }
}