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

        // Use tensor operations instead of to_vec1() for GPU efficiency
        let max_val = logits.max(0).map_err(|e| e.to_string())?;
        let threshold_scalar = self.temperature * min_p.ln();
        let threshold = (&max_val + threshold_scalar as f64).map_err(|e| e.to_string())?;

        // Create mask: logits >= threshold
        let mask = logits.ge(&threshold).map_err(|e| e.to_string())?;

        // Create neg_infinity tensor for masked values
        let neg_inf = Tensor::new(f32::NEG_INFINITY, logits.device())
            .map_err(|e| e.to_string())?
            .broadcast_as(logits.shape())
            .map_err(|e| e.to_string())?;

        // Apply mask: keep original where true, neg_infinity where false
        let result = mask
            .where_cond(logits, &neg_inf)
            .map_err(|e| e.to_string())?;

        if self.log_prints < 5 {
            // For logging, we still need to materialize some values
            let max_f32: f32 = max_val.to_scalar().map_err(|e| e.to_string())?;
            let threshold_f32 = max_f32 + threshold_scalar;
            let delta = max_f32 - threshold_f32;

            // Count kept tokens (requires CPU sync, done only for logging)
            let kept: u32 = mask
                .to_dtype(candle::DType::U32)
                .map_err(|e| e.to_string())?
                .sum_all()
                .map_err(|e| e.to_string())?
                .to_scalar()
                .map_err(|e| e.to_string())?;
            let total = logits.elem_count();

            log_infer!(
                "min_p applied: p={:.3}, temp={:.3}, max={:.4}, threshold={:.4}, delta={:.4}, kept={} of {}",
                min_p,
                self.temperature,
                max_f32,
                threshold_f32,
                delta,
                kept,
                total
            );
            self.log_prints += 1;
        }

        Ok(result)
    }
}
