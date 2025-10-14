mod embeddings;
mod search;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use candle::Device;
use candle_nn::{linear, Linear, Module, VarBuilder};
use candle_transformers::models::bert::{BertModel, Config as BertConfig, DTYPE};
use hf_hub::{api::sync::Api, Repo, RepoType};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokenizers::{EncodeInput, Tokenizer};
use uuid::Uuid;

use crate::core::device::select_device;
use crate::core::types::DevicePreference;
use crate::rag::ingestion::{DocumentChunk, DocumentKind};
use crate::rag::storage::{DocumentPayload, EmbeddingCache, PayloadChunk, VectorQuery, VectorStore};

use self::embeddings::{EmbeddingModel, EmbeddingModelConfig};
use self::search::{HybridSearcher, SearchConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossEncoderConfig {
    pub model_id: String,
    pub revision: Option<String>,
    pub weight: f32,
}

impl Default for CrossEncoderConfig {
    fn default() -> Self {
        Self {
            model_id: "cross-encoder/ms-marco-MiniLM-L-6-v2".into(),
            revision: None,
            weight: 0.2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    pub embedding: EmbeddingModelConfig,
    pub search: SearchConfig,
    pub device_preference: DevicePreference,
    pub max_vector_results: usize,
    pub cross_encoder: Option<CrossEncoderConfig>,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            embedding: EmbeddingModelConfig::default(),
            search: SearchConfig::default(),
            device_preference: DevicePreference::Cpu,
            max_vector_results: 64,
            cross_encoder: Some(CrossEncoderConfig::default()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RagQuery {
    pub text: String,
    pub top_k: usize,
    pub filters: HashMap<String, String>,
}

impl RagQuery {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            top_k: 10,
            filters: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RagHit {
    pub chunk: DocumentChunk,
    pub vector_score: f32,
    pub bm25_score: f32,
    pub rerank_score: Option<f32>,
    pub combined_score: f32,
}

#[derive(Debug, Clone)]
pub struct RagResult {
    pub hits: Vec<RagHit>,
}

#[async_trait]
pub trait DocumentIndexer: Send + Sync {
    async fn index_document(
        &self,
        path: &std::path::Path,
        kind: DocumentKind,
        chunks: Vec<DocumentChunk>,
    ) -> anyhow::Result<()>;
}

pub struct RagEngine {
    storage: Arc<dyn VectorStore>,
    embedder: Arc<Mutex<EmbeddingModel>>,
    searcher: Arc<HybridSearcher>,
    cache: EmbeddingCache,
    config: InferenceConfig,
    reranker: Option<Arc<Mutex<CrossEncoder>>>,
}

impl RagEngine {
    pub async fn new(
        storage: Arc<dyn VectorStore>,
        cache: EmbeddingCache,
        workspace_dir: PathBuf,
        config: InferenceConfig,
    ) -> anyhow::Result<Self> {
        let device_preference = config.device_preference.clone();
        let device = select_device(Some(device_preference));
        let embedder = EmbeddingModel::load(config.embedding.clone(), device.clone()).await?;
        let embedder = Arc::new(Mutex::new(embedder));
        let search_dir = workspace_dir.join("rag_index");
        let searcher = Arc::new(HybridSearcher::new(search_dir, config.search.clone())?);
        let reranker = if let Some(rerank_config) = config.cross_encoder.clone() {
            Some(Arc::new(Mutex::new(
                CrossEncoder::load(rerank_config, device).await?,
            )))
        } else {
            None
        };
        Ok(Self {
            storage,
            embedder,
            searcher,
            cache,
            config,
            reranker,
        })
    }

    pub async fn query(&self, query: RagQuery) -> anyhow::Result<RagResult> {
        let embedding = self.embed(vec![query.text.clone()]).await?;
        let embedding = embedding
            .into_iter()
            .next()
            .context("embedding generation failed")?;

        let vector_results = self
            .storage
            .vector_search(&VectorQuery {
                embedding: embedding.clone(),
                top_k: self.config.max_vector_results.max(query.top_k),
                filters: query.filters.clone(),
            })
            .await?;
        let bm25_hits = self
            .searcher
            .search(&query.text, self.config.max_vector_results)?;

        let mut aggregated: HashMap<Uuid, AggregatedHit> = HashMap::new();

        for stored in vector_results {
            let entry = aggregated
                .entry(stored.chunk.id)
                .or_insert_with(|| AggregatedHit::from_chunk(stored.chunk.clone()));
            entry.vector_score = stored.score;
            entry.embedding = Some(stored.embedding);
        }

        for hit in bm25_hits {
            let stored = self.storage.get_chunk(hit.chunk_id).await?;
            let entry = aggregated.entry(hit.chunk_id).or_insert_with(|| {
                stored
                    .clone()
                    .map(|stored| {
                        let mut hit = AggregatedHit::from_chunk(stored.chunk);
                        hit.embedding = Some(stored.embedding);
                        hit
                    })
                    .unwrap_or_else(|| {
                        AggregatedHit::from_chunk(DocumentChunk {
                            id: hit.chunk_id,
                            coordinate: crate::rag::ingestion::ChunkCoordinate {
                                section: hit
                                    .metadata
                                    .get("section")
                                    .and_then(|s| s.parse().ok())
                                    .unwrap_or_default(),
                                index: hit
                                    .metadata
                                    .get("chunk_index")
                                    .and_then(|s| s.parse().ok())
                                    .unwrap_or_default(),
                                start_token: 0,
                                end_token: 0,
                            },
                            text: String::new(),
                            token_count: 0,
                            metadata: hit.metadata.clone(),
                        })
                    })
            });
            if entry.embedding.is_none() {
                if let Some(stored) = stored.clone() {
                    entry.embedding = Some(stored.embedding);
                }
            }
            entry.bm25_score = hit.score;
        }

        let vector_max = aggregated
            .values()
            .map(|entry| entry.vector_score)
            .fold(0.0f32, f32::max);
        let bm25_max = aggregated
            .values()
            .map(|entry| entry.bm25_score)
            .fold(0.0f32, f32::max);

        let rerank_candidates = self.config.search.rerank_top_k.min(query.top_k * 2);
        let mut scored_hits: Vec<_> = aggregated
            .into_values()
            .map(|mut entry| {
                let vector_norm = if vector_max > 0.0 {
                    entry.vector_score / vector_max
                } else {
                    0.0
                };
                let bm25_norm = if bm25_max.abs() > 1e-9 {
                    entry.bm25_score / bm25_max
                } else {
                    0.0
                };
                entry.combined = self.config.search.vector_weight * vector_norm
                    + self.config.search.bm25_weight * bm25_norm;
                entry
            })
            .collect();

        scored_hits.sort_by(|a, b| {
            b.combined
                .partial_cmp(&a.combined)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let rerank_slice = scored_hits.iter_mut().take(rerank_candidates);
        if let Some(reranker) = &self.reranker {
            for candidate in rerank_slice {
                if candidate.chunk.text.is_empty() {
                    continue;
                }
                let rerank_score = self
                    .score_with_cross_encoder(&query.text, &candidate.chunk.text, reranker)
                    .await?;
                candidate.rerank = Some(rerank_score);
                candidate.combined = (1.0 - self
                    .config
                    .cross_encoder
                    .as_ref()
                    .map(|cfg| cfg.weight)
                    .unwrap_or(0.0))
                    * candidate.combined
                    + rerank_score
                        * self
                            .config
                            .cross_encoder
                            .as_ref()
                            .map(|cfg| cfg.weight)
                            .unwrap_or(0.0);
            }
        }

        scored_hits.sort_by(|a, b| {
            b.combined
                .partial_cmp(&a.combined)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let hits = scored_hits
            .into_iter()
            .take(query.top_k)
            .map(|entry| RagHit {
                chunk: entry.chunk,
                vector_score: entry.vector_score,
                bm25_score: entry.bm25_score,
                rerank_score: entry.rerank,
                combined_score: entry.combined,
            })
            .collect();

        Ok(RagResult { hits })
    }

    async fn embed(&self, texts: Vec<String>) -> anyhow::Result<Vec<Vec<f32>>> {
        let mut results: Vec<Option<Vec<f32>>> = vec![None; texts.len()];
        let mut missing = Vec::new();

        for (idx, text) in texts.iter().enumerate() {
            let key = cache_key_for_text(text);
            if let Some(cached) = self.cache.lookup(&key) {
                results[idx] = Some((*cached).clone());
            } else {
                missing.push((idx, text.clone(), key));
            }
        }

        if !missing.is_empty() {
            let embedder = Arc::clone(&self.embedder);
            let compute_texts: Vec<String> = missing.iter().map(|(_, text, _)| text.clone()).collect();
            let embeddings = tokio::task::spawn_blocking(move || {
                let embedder = embedder.blocking_lock();
                embedder.encode(&compute_texts)
            })
            .await??;

            for ((idx, _text, key), embedding) in missing.into_iter().zip(embeddings.into_iter()) {
                self.cache.insert(key.clone(), embedding.clone());
                results[idx] = Some(embedding);
            }
        }

        Ok(results.into_iter().map(|item| item.unwrap()).collect())
    }

    async fn score_with_cross_encoder(
        &self,
        query: &str,
        passage: &str,
        reranker: &Arc<Mutex<CrossEncoder>>,
    ) -> anyhow::Result<f32> {
        let reranker = Arc::clone(reranker);
        let query_owned = query.to_string();
        let passage_owned = passage.to_string();
        tokio::task::spawn_blocking(move || {
            let reranker = reranker.blocking_lock();
            reranker.score(&query_owned, &passage_owned)
        })
        .await?
    }
}

#[async_trait]
impl DocumentIndexer for RagEngine {
    async fn index_document(
        &self,
        path: &std::path::Path,
        kind: DocumentKind,
        chunks: Vec<DocumentChunk>,
    ) -> anyhow::Result<()> {
        let texts: Vec<String> = chunks
            .iter()
            .map(|chunk| chunk.text.clone())
            .collect();

        let embeddings = self.embed(texts).await?;
        let mut payload_chunks = Vec::new();
        for (chunk, embedding) in chunks.into_iter().zip(embeddings.into_iter()) {
            self.searcher
                .add_chunk(path.to_string_lossy().as_ref(), &chunk)?;
            payload_chunks.push(PayloadChunk { chunk, embedding });
        }

        self.searcher.commit()?;

        let payload = DocumentPayload {
            path: path.to_path_buf(),
            kind,
            fingerprint: None,
            chunks: payload_chunks,
        };
        self.storage.persist(payload).await
    }
}

struct AggregatedHit {
    chunk: DocumentChunk,
    vector_score: f32,
    bm25_score: f32,
    rerank: Option<f32>,
    combined: f32,
    embedding: Option<Vec<f32>>,
}

impl AggregatedHit {
    fn from_chunk(chunk: DocumentChunk) -> Self {
        Self {
            chunk,
            vector_score: 0.0,
            bm25_score: 0.0,
            rerank: None,
            combined: 0.0,
            embedding: None,
        }
    }
}

struct CrossEncoder {
    bert: BertModel,
    classifier: Linear,
    tokenizer: Tokenizer,
    device: Device,
}

impl CrossEncoder {
    async fn load(config: CrossEncoderConfig, device: Device) -> anyhow::Result<Self> {
        tokio::task::spawn_blocking(move || Self::load_sync(config, device)).await?
    }

    fn load_sync(config: CrossEncoderConfig, device: Device) -> anyhow::Result<Self> {
        let revision = config.revision.unwrap_or_else(|| "main".into());
        let repo = Repo::with_revision(config.model_id.clone(), RepoType::Model, revision);
        let api = Api::new()?;
        let repo = api.repo(repo);
        let config_path = repo.get("config.json")?;
        let tokenizer_path = repo.get("tokenizer.json")?;
        let weights_path = repo
            .get("model.safetensors")
            .or_else(|_| repo.get("pytorch_model.bin"))?;

        let config_contents = std::fs::read_to_string(&config_path)?;
        let raw_config: serde_json::Value = serde_json::from_str(&config_contents)?;
        let num_labels = raw_config
            .get("num_labels")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as usize;
        let bert_config: BertConfig = serde_json::from_value(raw_config)?;
        let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(anyhow::Error::msg)?;
        let vb = if weights_path
            .extension()
            .and_then(|ext| ext.to_str())
            == Some("bin")
        {
            VarBuilder::from_pth(weights_path, DTYPE, &device)?
        } else {
            unsafe { VarBuilder::from_mmaped_safetensors(&[weights_path], DTYPE, &device)? }
        };
        let bert = BertModel::load(vb.pp("bert"), &bert_config)?;
        let classifier = linear(
            bert_config.hidden_size,
            num_labels,
            vb.pp("classifier"),
        )?;
        Ok(Self {
            bert,
            classifier,
            tokenizer,
            device,
        })
    }

    fn score(&self, query: &str, passage: &str) -> anyhow::Result<f32> {
        let encoding = self
            .tokenizer
            .encode(EncodeInput::Dual(query.into(), passage.into()), true)
            .map_err(anyhow::Error::msg)?;
        let token_ids =
            candle::Tensor::new(encoding.get_ids().to_vec(), &self.device)?.unsqueeze(0)?;
        let type_ids =
            candle::Tensor::new(encoding.get_type_ids().to_vec(), &self.device)?.unsqueeze(0)?;
        let attention_mask = candle::Tensor::new(
            encoding.get_attention_mask().to_vec(),
            &self.device,
        )?
        .unsqueeze(0)?;

        let outputs = self
            .bert
            .forward(&token_ids, &type_ids, Some(&attention_mask))?;
        let outputs_vec = outputs.to_vec3::<f32>()?;
        let cls_vec = outputs_vec
            .first()
            .and_then(|doc| doc.first())
            .ok_or_else(|| anyhow::anyhow!("empty cross-encoder output"))?;
        let cls = candle::Tensor::new(cls_vec.clone(), &self.device)?.unsqueeze(0)?;
        let logits = self.classifier.forward(&cls)?;
        let raw = logits
            .squeeze(0)?
            .to_vec1::<f32>()?
            .into_iter()
            .next()
            .unwrap_or(0.0);
        Ok(1.0 / (1.0 + (-raw).exp()))
    }
}

fn cache_key_for_text(text: &str) -> String {
    let hash = blake3::hash(text.as_bytes());
    format!("query:{}", hash)
}









