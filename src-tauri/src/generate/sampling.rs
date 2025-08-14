use candle_transformers::generation::{LogitsProcessor, Sampling};

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


