use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::rag::ingestion::DocumentChunk;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub vector_weight: f32,
    pub bm25_weight: f32,
    pub rerank_top_k: usize,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            vector_weight: 0.6,
            bm25_weight: 0.4,
            rerank_top_k: 20,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchHit {
    pub chunk_id: Uuid,
    pub score: f32,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
struct IndexedChunk {
    chunk_id: Uuid,
    text: String,
    metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub struct HybridSearcher {
    _index_path: PathBuf,
    _config: SearchConfig,
    chunks: RwLock<Vec<IndexedChunk>>,
}

impl HybridSearcher {
    #[instrument(skip(config), fields(index = %index_dir.display()))]
    pub fn new(index_dir: PathBuf, config: SearchConfig) -> Result<Self> {
        if let Some(parent) = index_dir.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if !index_dir.exists() {
            std::fs::create_dir_all(&index_dir)?;
        }

        info!(
            ?config,
            "HybridSearcher initialised with lightweight in-memory index"
        );

        Ok(Self {
            _index_path: index_dir,
            _config: config,
            chunks: RwLock::new(Vec::new()),
        })
    }

    #[instrument(skip(self, chunk))]
    pub fn add_chunk(&self, _path: &str, chunk: &DocumentChunk) -> Result<()> {
        let mut store = self.chunks.write();
        store.push(IndexedChunk {
            chunk_id: chunk.id,
            text: chunk.text.clone(),
            metadata: chunk.metadata.clone(),
        });
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn commit(&self) -> Result<()> {
        // No-op for the in-memory placeholder implementation.
        debug!(total_chunks = self.chunks.read().len(), "Commit called");
        Ok(())
    }

    #[instrument(skip(self, query), fields(top_k = top_k))]
    pub fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchHit>> {
        let tokens = tokenize(query);
        if tokens.is_empty() {
            return Ok(Vec::new());
        }

        let store = self.chunks.read();
        let mut hits: Vec<SearchHit> = store
            .iter()
            .filter_map(|chunk| {
                let score = score_chunk(&chunk.text, &tokens);
                if score > 0.0 {
                    Some(SearchHit {
                        chunk_id: chunk.chunk_id,
                        score,
                        metadata: chunk.metadata.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(top_k);
        debug!(
            hits = hits.len(),
            "Vector/BM25 hybrid search results produced"
        );
        Ok(hits)
    }
}

fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|token| {
            token
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|token| !token.is_empty())
        .collect()
}

fn score_chunk(text: &str, tokens: &[String]) -> f32 {
    if tokens.is_empty() {
        return 0.0;
    }

    let lower = text.to_lowercase();
    let mut hits = 0usize;
    for token in tokens {
        if lower.contains(token) {
            hits += 1;
        }
    }

    if hits == 0 {
        0.0
    } else {
        hits as f32 / tokens.len() as f32
    }
}
