//! High-level coordinator for the Retrieval-Augmented Generation (RAG) stack.
//!
//! The module exposes a modular three-layer architecture as required by the
//! project specification:
//! - ingestion (`ingestion::IngestionPipeline`)
//! - storage (`storage::VectorStore`)
//! - inference (`inference::RagEngine`)
//!
//! Each submodule provides a focused API while this module wires the building
//! blocks together and offers a cohesive entry-point for Tauri commands as well
//! as background services.

pub mod inference;
pub mod ingestion;
pub mod storage;
pub mod tauri_integration;

use std::sync::Arc;
use tokio::sync::RwLock;

use chrono::{DateTime, Utc};

use inference::{DocumentIndexer, InferenceConfig, RagEngine, RagQuery, RagResult};
use ingestion::{FileIngestionConfig, IngestionPipeline, IngestionStats};
use std::path::{Path, PathBuf};
use storage::{CacheConfig, EmbeddingCache, StorageConfig, VectorStore, VectorStoreFactory};

pub type RagServiceState = Arc<RwLock<RagService>>;

/// Shared configuration knobs for the RAG subsystem.
#[derive(Debug, Clone)]
pub struct RagConfig {
    pub workspace_dir: PathBuf,
    pub cache: CacheConfig,
    pub storage: StorageConfig,
    pub ingestion: FileIngestionConfig,
    pub inference: InferenceConfig,
}

impl RagConfig {
    pub fn new<P: AsRef<Path>>(workspace_dir: P) -> Self {
        Self {
            workspace_dir: workspace_dir.as_ref().to_path_buf(),
            cache: CacheConfig::default(),
            storage: StorageConfig::default(),
            ingestion: FileIngestionConfig::default(),
            inference: InferenceConfig::default(),
        }
    }
}

/// Central handle that keeps references to each layer.
pub struct RagService {
    storage: Arc<dyn VectorStore>,
    ingestion: Arc<IngestionPipeline>,
    engine: Arc<RagEngine>,
    stats: Arc<RwLock<IngestionStats>>,
    last_loaded: Arc<RwLock<Option<DateTime<Utc>>>>,
}

impl RagService {
    pub async fn initialise(config: RagConfig) -> anyhow::Result<Self> {
        let cache = EmbeddingCache::new(config.cache.clone());
        let factory = VectorStoreFactory::new(cache.clone());
        let storage = factory.build(&config.storage).await?;
        let storage = Arc::from(storage);

        let stats = Arc::new(RwLock::new(IngestionStats::default()));

        let engine = Arc::new(
            RagEngine::new(
                Arc::clone(&storage),
                cache,
                config.workspace_dir.clone(),
                config.inference.clone(),
            )
            .await?,
        );

        let ingestion = Arc::new(
            IngestionPipeline::initialise(
                config.workspace_dir.clone(),
                config.ingestion.clone(),
                Arc::clone(&engine) as Arc<dyn DocumentIndexer>,
                Arc::clone(&stats),
            )
            .await?,
        );

        Ok(Self {
            storage,
            ingestion,
            engine,
            stats,
            last_loaded: Arc::new(RwLock::new(None)),
        })
    }

    pub fn ingestion(&self) -> Arc<IngestionPipeline> {
        Arc::clone(&self.ingestion)
    }

    pub fn engine(&self) -> Arc<RagEngine> {
        Arc::clone(&self.engine)
    }

    pub fn storage(&self) -> Arc<dyn VectorStore> {
        Arc::clone(&self.storage)
    }

    pub fn stats(&self) -> Arc<RwLock<IngestionStats>> {
        Arc::clone(&self.stats)
    }

    pub fn last_loaded(&self) -> Arc<RwLock<Option<DateTime<Utc>>>> {
        Arc::clone(&self.last_loaded)
    }

    pub async fn query(&self, query: RagQuery) -> anyhow::Result<RagResult> {
        self.engine.query(query).await
    }
}
