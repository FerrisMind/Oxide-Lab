use candle_core::{Device, Tensor, DType, Result as CandleResult};
use candle_nn::VarBuilder;
use candle_transformers::models::qwen::{Config, Model as QwenModel};
use candle_transformers::models::qwen::tokenizer::Tokenizer;
use candle_transformers::generation::LogitsProcessor;
use anyhow::Result;
use serde_json;
use std::path::Path;
use std::fs;

pub struct CandleLLM {
    model: Option<QwenModel>,
    tokenizer: Option<Tokenizer>,
    device: Device,
}

impl CandleLLM {
    pub fn new(device: Device) -> Self {
        Self { model: None, tokenizer: None, device }
    }

    pub fn load_model(&mut self, model_dir: &str) -> Result<()> {
        let dir_path = Path::new(model_dir);
        let config_path = dir_path.join("config.json");
        let tokenizer_path = dir_path.join("tokenizer.json");

        // Load config
        let config_str = fs::read_to_string(&config_path)?;
        let config: Config = serde_json::from_str(&config_str)?;

        // Load tokenizer
        self.tokenizer = Some(Tokenizer::from_file(&tokenizer_path, &self.device)?);

        // Collect safetensors files
        let mut safetensors_paths = vec![];
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("safetensors") {
                safetensors_paths.push(path);
            }
        }

        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&safetensors_paths, DType::F16, &self.device)? };
        self.model = Some(QwenModel::load(&vb, &config)?);
        Ok(())
    }

    pub fn generate(&self, prompt: &str, max_tokens: usize, temperature: f64) -> Result<String> {
        if self.model.is_none() || self.tokenizer.is_none() {
            return Err(anyhow::anyhow!("Model or tokenizer not loaded"));
        }
        let model = self.model.as_ref().unwrap();
        let tokenizer = self.tokenizer.as_ref().unwrap();

        // Tokenize
        let tokens = tokenizer.encode(prompt, true)?.get_ids().to_vec();
        let mut generated_tokens = tokens.clone();

        let eos_token_id = 151643; // EOS token for Qwen

        for _ in 0..max_tokens {
            if generated_tokens.last() == Some(&eos_token_id) { break; }

            let seq_len = generated_tokens.len();
            let input = Tensor::new(&generated_tokens, &self.device)?.unsqueeze(0)?;
            let position_ids = Tensor::arange(0u32, seq_len as u32, &self.device)?.unsqueeze(0)?;
            let logits = model.forward(&input, &position_ids)?;

            // Last token logits
            let last_logits = logits.narrow(1, (seq_len - 1) as i64, 1)?.squeeze(1)?.squeeze(0)?;

            let mut logits_processor = LogitsProcessor::new(temperature, 1.0, true);
            let processed_logits = logits_processor.process(&last_logits, seq_len - 1)?;

            let next_token_id = if temperature > 0.0 {
                let probs = candle_nn::ops::softmax(&processed_logits, 0)?;
                let next_token = candle_nn::ops::multinomial(&probs.unsqueeze(0)?, 1)?;
                next_token.squeeze(1)?.to_scalar::<u32>()? as usize
            } else {
                processed_logits.argmax(0)?.to_scalar::<u32>()? as usize
            };

            generated_tokens.push(next_token_id);
            if next_token_id == eos_token_id { break; }
        }

        let output_tokens = &generated_tokens[tokens.len()..];
        let generated = tokenizer.decode(output_tokens)?;
        Ok(generated.trim().to_string())
    }
}

