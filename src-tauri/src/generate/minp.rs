use crate::log_infer;
use candle::Tensor;

pub struct MinPFilter {
    min_p: Option<f32>,
    temperature: f32,
    log_prints: usize,
}

impl MinPFilter {
    pub fn new(min_p: Option<f64>, temperature: f64) -> Self {
        Self {
            min_p: min_p.and_then(|v| {
                if (0.0..=1.0).contains(&v) {
                    Some(v as f32)
                } else {
                    None
                }
            }),
            temperature: temperature as f32,
            log_prints: 0,
        }
    }

    pub fn apply(&mut self, logits: &Tensor) -> Result<Tensor, String> {
        let min_p = match self.min_p {
            Some(v) => v,
            None => return Ok(logits.clone()),
        };
        if self.temperature <= 0.0 {
            if self.log_prints < 5 {
                log_infer!("min_p ignored because temperature <= 0");
                self.log_prints += 1;
            }
            return Ok(logits.clone());
        }
        let vals: Vec<f32> = logits.to_vec1::<f32>().map_err(|e| e.to_string())?;
        if vals.is_empty() {
            return Ok(logits.clone());
        }
        let mut max_val = f32::NEG_INFINITY;
        let mut second_val = f32::NEG_INFINITY;
        for &v in &vals {
            if v > max_val {
                second_val = max_val;
                max_val = v;
            } else if v > second_val {
                second_val = v;
            }
        }
        let threshold = max_val + self.temperature * (min_p.ln());
        let mut kept = 0usize;
        let masked: Vec<f32> = vals
            .into_iter()
            .map(|v| {
                if v >= threshold {
                    kept += 1;
                    v
                } else {
                    f32::NEG_INFINITY
                }
            })
            .collect();
        if self.log_prints < 5 {
            let delta = max_val - threshold;
            let gap12 = max_val - second_val;
            log_infer!(
                "min_p applied: p={:.3}, temp={:.3}, max={:.4}, threshold={:.4}, delta={:.4}, gap12={:.4}, kept={} of {}",
                min_p,
                self.temperature,
                max_val,
                threshold,
                delta,
                gap12,
                kept,
                masked.len()
            );
            self.log_prints += 1;
        }
        Tensor::new(masked.as_slice(), logits.device()).map_err(|e| e.to_string())
    }
}
