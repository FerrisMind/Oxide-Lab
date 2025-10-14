use std::{
    collections::HashSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, anyhow};
use candle::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config as BertConfig};
use hf_hub::{
    Repo, RepoType,
    api::sync::{Api, ApiRepo},
};
use serde::{Deserialize, Serialize};
use tokenizers::{
    PaddingDirection, PaddingParams, PaddingStrategy, Tokenizer, TruncationDirection,
    TruncationParams, TruncationStrategy,
};
use tracing::{debug, info, instrument};

const DEFAULT_MODEL_DIR: &str = ".oxide-data/embedding-models";
const DEFAULT_MAX_LENGTH: usize = 512;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PoolingStrategy {
    Mean,
    Cls,
}

impl Default for PoolingStrategy {
    fn default() -> Self {
        Self::Mean
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddingDType {
    F32,
    Bf16,
}

impl Default for EmbeddingDType {
    fn default() -> Self {
        Self::F32
    }
}

impl EmbeddingDType {
    fn to_candle(&self) -> DType {
        match self {
            EmbeddingDType::F32 => DType::F32,
            EmbeddingDType::Bf16 => DType::BF16,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingModelConfig {
    pub model_id: String,
    #[serde(default)]
    pub revision: Option<String>,
    #[serde(default)]
    pub pooling: PoolingStrategy,
    #[serde(default)]
    pub dtype: EmbeddingDType,
    #[serde(default)]
    pub max_length: Option<usize>,
    #[serde(default)]
    pub local_dir: Option<PathBuf>,
    #[serde(default = "default_normalize")]
    pub normalize_embeddings: bool,
}

fn default_normalize() -> bool {
    true
}

impl Default for EmbeddingModelConfig {
    fn default() -> Self {
        Self {
            model_id: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            revision: Some("main".to_string()),
            pooling: PoolingStrategy::Mean,
            dtype: EmbeddingDType::F32,
            max_length: Some(DEFAULT_MAX_LENGTH),
            local_dir: None,
            normalize_embeddings: true,
        }
    }
}

pub struct EmbeddingModel {
    pub config: EmbeddingModelConfig,
    device: Device,
    tokenizer: Arc<Tokenizer>,
    model: BertModel,
    embedding_dim: usize,
    #[allow(dead_code)]
    dtype: DType,
}

impl EmbeddingModel {
    #[instrument(skip(device), fields(model = %config.model_id, revision = ?config.revision))]
    pub async fn load(config: EmbeddingModelConfig, device: Device) -> anyhow::Result<Self> {
        let cloned_device = device.clone();
        let cloned_config = config.clone();
        tokio::task::spawn_blocking(move || Self::load_blocking(cloned_config, cloned_device))
            .await?
    }

    fn load_blocking(config: EmbeddingModelConfig, device: Device) -> anyhow::Result<Self> {
        let target_dir = config
            .local_dir
            .clone()
            .unwrap_or_else(|| default_model_directory(&config.model_id));
        fs::create_dir_all(&target_dir).with_context(|| {
            format!(
                "failed to create embedding cache dir {}",
                target_dir.display()
            )
        })?;

        let files = prepare_model_files(&config, &target_dir)
            .with_context(|| "failed to download embedding model assets".to_string())?;

        let tokenizer = Tokenizer::from_file(&files.tokenizer_path)
            .map_err(anyhow::Error::msg)
            .with_context(|| {
                format!(
                    "failed to load tokenizer from {}",
                    files.tokenizer_path.display()
                )
            })?;

        let bert_config: BertConfig = {
            let mut buf = String::new();
            fs::File::open(&files.config_path)
                .with_context(|| format!("failed to open {}", files.config_path.display()))?
                .read_to_string(&mut buf)?;
            serde_json::from_str(&buf).context("failed to parse config.json as BertConfig")?
        };

        let dtype = config.dtype.to_candle();
        let vb =
            unsafe { VarBuilder::from_mmaped_safetensors(&files.weight_paths, dtype, &device)? };

        let model =
            BertModel::load(vb, &bert_config).with_context(|| "failed to construct BertModel")?;

        let embedding_dim = bert_config.hidden_size;

        info!(
            model_id = %config.model_id,
            embedding_dim,
            dtype = ?dtype,
            "Loaded safetensors embedding model"
        );

        Ok(Self {
            config,
            device,
            tokenizer: Arc::new(tokenizer),
            model,
            embedding_dim,
            dtype,
        })
    }

    #[instrument(skip(self, texts), fields(batch = texts.len()))]
    pub fn encode(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut tokenizer = (*self.tokenizer).clone();
        prepare_padding(&mut tokenizer);
        prepare_truncation(&mut tokenizer, self.config.max_length);

        let encoded = tokenizer
            .encode_batch(texts.to_vec(), true)
            .map_err(anyhow::Error::msg)
            .context("tokenization failed")?;

        let mut input_tensors = Vec::with_capacity(encoded.len());
        let mut attention_tensors = Vec::with_capacity(encoded.len());
        let mut type_tensors = Vec::with_capacity(encoded.len());

        for item in &encoded {
            let ids = Tensor::new(item.get_ids(), &self.device)?;
            let attention_mask = Tensor::new(item.get_attention_mask(), &self.device)?;
            let type_ids = if !item.get_type_ids().is_empty() {
                Tensor::new(item.get_type_ids(), &self.device)?
            } else {
                ids.zeros_like()?
            };
            input_tensors.push(ids);
            attention_tensors.push(attention_mask);
            type_tensors.push(type_ids);
        }

        let input_ids = Tensor::stack(&input_tensors, 0)?;
        let attention_mask = Tensor::stack(&attention_tensors, 0)?;
        let token_type_ids = Tensor::stack(&type_tensors, 0)?;

        let hidden_states =
            self.model
                .forward(&input_ids, &token_type_ids, Some(&attention_mask))?;

        let hidden_states = hidden_states.to_dtype(DType::F32)?;
        let attention_mask_f32 = attention_mask.to_dtype(DType::F32)?;

        let pooled = match self.config.pooling {
            PoolingStrategy::Mean => mean_pool(&hidden_states, &attention_mask_f32)?,
            PoolingStrategy::Cls => cls_pool(&hidden_states)?,
        };

        let pooled = if self.config.normalize_embeddings {
            normalize_l2(&pooled)?
        } else {
            pooled
        };

        let embeddings = pooled
            .to_dtype(DType::F32)?
            .to_vec2::<f32>()
            .context("failed to materialize embedding tensors")?;

        debug!(
            batch = texts.len(),
            dimension = self.embedding_dim,
            "Generated embeddings via safetensors backend"
        );

        Ok(embeddings)
    }

    #[allow(dead_code)]
    pub fn device(&self) -> &Device {
        &self.device
    }

    #[allow(dead_code)]
    pub fn embedding_dim(&self) -> usize {
        self.embedding_dim
    }
}

struct PreparedFiles {
    config_path: PathBuf,
    tokenizer_path: PathBuf,
    weight_paths: Vec<PathBuf>,
}

fn default_model_directory(model_id: &str) -> PathBuf {
    let safe_id = model_id.replace('/', "_");
    Path::new(DEFAULT_MODEL_DIR).join(safe_id)
}

fn prepare_model_files(
    config: &EmbeddingModelConfig,
    target_dir: &Path,
) -> anyhow::Result<PreparedFiles> {
    let revision = config
        .revision
        .clone()
        .unwrap_or_else(|| "main".to_string());
    let api = Api::new().context("failed to initialise hf-hub API")?;
    let repo = Repo::with_revision(config.model_id.clone(), RepoType::Model, revision);
    let repo_api = api.repo(repo);

    let config_path = ensure_local_copy(target_dir, "config.json", || {
        repo_api.get("config.json").map_err(|e| anyhow!(e))
    })?;

    let tokenizer_path = ensure_local_copy(target_dir, "tokenizer.json", || {
        repo_api.get("tokenizer.json").map_err(|e| anyhow!(e))
    })?;

    let weight_paths = download_safetensors(&repo_api, target_dir)?;

    Ok(PreparedFiles {
        config_path,
        tokenizer_path,
        weight_paths,
    })
}

fn ensure_local_copy<F>(target_dir: &Path, filename: &str, downloader: F) -> anyhow::Result<PathBuf>
where
    F: FnOnce() -> anyhow::Result<PathBuf>,
{
    let destination = target_dir.join(filename);
    if destination.exists() {
        return Ok(destination);
    }
    let source = downloader().with_context(|| format!("failed to download {filename} from hub"))?;
    fs::copy(&source, &destination).with_context(|| {
        format!(
            "failed to copy {} to {}",
            source.display(),
            destination.display()
        )
    })?;
    Ok(destination)
}

fn download_safetensors(repo_api: &ApiRepo, target_dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    if let Ok(index_path) = repo_api.get("model.safetensors.index.json") {
        let local_index = ensure_local_copy(target_dir, "model.safetensors.index.json", || {
            Ok::<PathBuf, anyhow::Error>(index_path.clone())
        })?;
        let index: serde_json::Value = {
            let mut buf = String::new();
            fs::File::open(&local_index)
                .context("failed to open model.safetensors.index.json")?
                .read_to_string(&mut buf)?;
            serde_json::from_str(&buf).context("failed to parse safetensors index")?
        };
        let map = index
            .get("weight_map")
            .and_then(|m| m.as_object())
            .ok_or_else(|| anyhow!("weight_map missing in safetensors index"))?;
        let mut unique: HashSet<String> = HashSet::new();
        for value in map.values() {
            if let Some(path) = value.as_str() {
                unique.insert(path.to_string());
            }
        }
        let mut collected = Vec::with_capacity(unique.len());
        for shard in unique {
            let local = ensure_local_copy(target_dir, &shard, || {
                repo_api.get(&shard).map_err(|e| anyhow!(e))
            })?;
            collected.push(local);
        }
        collected.sort();
        Ok(collected)
    } else {
        let weight = ensure_local_copy(target_dir, "model.safetensors", || {
            repo_api.get("model.safetensors").map_err(|e| anyhow!(e))
        })?;
        Ok(vec![weight])
    }
}

fn mean_pool(hidden_states: &Tensor, attention_mask: &Tensor) -> anyhow::Result<Tensor> {
    let mask = attention_mask.unsqueeze(2)?;
    let weighted = hidden_states.broadcast_mul(&mask)?;
    let sum = weighted.sum(1)?;
    let counts = mask.sum(1)?;
    let eps = Tensor::try_from(1e-6f32)?.to_device(counts.device())?;
    let counts = counts.broadcast_add(&eps)?;
    Ok(sum.broadcast_div(&counts)?)
}

fn cls_pool(hidden_states: &Tensor) -> anyhow::Result<Tensor> {
    Ok(hidden_states.narrow(1, 0, 1)?.squeeze(1)?)
}

fn normalize_l2(tensor: &Tensor) -> anyhow::Result<Tensor> {
    let squared = tensor.sqr()?.sum_keepdim(1)?;
    let eps = Tensor::try_from(1e-12f32)?.to_device(squared.device())?;
    let denom = squared.broadcast_add(&eps)?.sqrt()?;
    Ok(tensor.broadcast_div(&denom)?)
}

fn prepare_padding(tokenizer: &mut Tokenizer) {
    if let Some(padding) = tokenizer.get_padding_mut() {
        padding.strategy = PaddingStrategy::BatchLongest;
        padding.direction = PaddingDirection::Right;
    } else {
        let params = PaddingParams {
            direction: PaddingDirection::Right,
            strategy: PaddingStrategy::BatchLongest,
            ..Default::default()
        };
        tokenizer.with_padding(Some(params));
    }
}

fn prepare_truncation(tokenizer: &mut Tokenizer, max_length: Option<usize>) {
    match max_length {
        Some(max_len) => {
            let params = TruncationParams {
                max_length: max_len,
                strategy: TruncationStrategy::LongestFirst,
                stride: 0,
                direction: TruncationDirection::Right,
            };
            let _ = tokenizer.with_truncation(Some(params));
        }
        None => {
            let _ = tokenizer.with_truncation(None);
        }
    }
}
