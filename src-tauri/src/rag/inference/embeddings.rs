use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use candle::Device;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingModelConfig {
    pub model_id: String,
    #[serde(default)]
    pub revision: Option<String>,
    #[serde(default = "default_pooling")] 
    pub pooling: PoolingStrategy,
}

fn default_pooling() -> PoolingStrategy {
    PoolingStrategy::Mean
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoolingStrategy {
    Mean,
    Cls,
}

impl Default for EmbeddingModelConfig {
    fn default() -> Self {
        Self {
            model_id: "intfloat/e5-base-v2".to_string(),
            revision: Some("main".to_string()),
            pooling: default_pooling(),
        }
    }
}

pub struct EmbeddingModel {
    #[allow(dead_code)]
    pub config: EmbeddingModelConfig,
    device: Device,
    #[allow(dead_code)]
    model_dir: PathBuf,
}

impl EmbeddingModel {
    pub async fn load(config: EmbeddingModelConfig, device: Device) -> anyhow::Result<Self> {
        let model_dir = PathBuf::from(".oxide-data/embedding-models");
        tokio::fs::create_dir_all(&model_dir)
            .await
            .context("failed to create embedding model directory")?;

        Ok(Self {
            config,
            device,
            model_dir,
        })
    }

    pub fn encode(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>> {
        let dimension = 768;
        let mut out = Vec::with_capacity(texts.len());
        for (idx, _) in texts.iter().enumerate() {
            let mut vec = vec![0f32; dimension];
            if dimension > 0 {
                vec[idx % dimension] = 1.0;
            }
            out.push(vec);
        }
        Ok(out)
    }

    #[allow(dead_code)]
    pub fn device(&self) -> &Device {
        &self.device
    }
}

pub type SharedEmbeddingModel = Arc<Mutex<EmbeddingModel>>;

