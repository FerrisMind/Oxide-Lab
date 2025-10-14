//! Простая in-memory реализация кеша эмбеддингов, использующая LRU выселение.
//! Кеш служит общим слоем между инференсом и хранилищем векторов: он хранит
//! недавно вычисленные эмбеддинги, чтобы избежать повторных обращений к модели.
//!
//! Для целей текущего проекта достаточно RAM-кеша без потерь, однако позже
//! можно дополнить реализацией на диске. Параметры кеша управляются через
//! `CacheConfig` и прокидываются из `RagConfig`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

const DEFAULT_TTL_SECS: u64 = 60 * 60;
const DEFAULT_MAX_ENTRIES: usize = 10_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(default = "default_ttl_secs")]
    pub ttl_secs: u64,
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,
    #[serde(default = "default_cache_dir")]
    pub metadata_dir: PathBuf,
}

fn default_ttl_secs() -> u64 {
    DEFAULT_TTL_SECS
}

fn default_max_entries() -> usize {
    DEFAULT_MAX_ENTRIES
}

fn default_cache_dir() -> PathBuf {
    PathBuf::from(".oxide-data/rag/cache")
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            ttl_secs: default_ttl_secs(),
            max_entries: default_max_entries(),
            metadata_dir: default_cache_dir(),
        }
    }
}

#[derive(Debug, Clone)]
struct CacheEntry {
    embedding: Arc<Vec<f32>>,
    expires_at: Option<SystemTime>,
    last_used: SystemTime,
}

#[derive(Debug, Clone)]
pub struct EmbeddingCache {
    inner: Arc<RwLock<HashMap<String, CacheEntry>>>,
    ttl: Option<Duration>,
    max_entries: usize,
}

impl EmbeddingCache {
    pub fn new(config: CacheConfig) -> Self {
        let ttl = if config.ttl_secs == 0 {
            None
        } else {
            Some(Duration::from_secs(config.ttl_secs))
        };
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            ttl,
            max_entries: config.max_entries.max(1),
        }
    }

    pub fn insert(&self, key: String, embedding: Vec<f32>) {
        let expires_at = self.ttl.map(|ttl| SystemTime::now() + ttl);
        let entry = CacheEntry {
            embedding: Arc::new(embedding),
            expires_at,
            last_used: SystemTime::now(),
        };

        let mut guard = self.inner.write();

        if guard.len() >= self.max_entries {
            Self::evict_lru(&mut guard);
        }

        guard.insert(key, entry);
    }

    pub fn lookup(&self, key: &str) -> Option<Arc<Vec<f32>>> {
        let mut guard = self.inner.write();
        if let Some(entry) = guard.get_mut(key) {
            if let Some(expiry) = entry.expires_at
                && SystemTime::now() > expiry
            {
                guard.remove(key);
                return None;
            }
            entry.last_used = SystemTime::now();
            return Some(Arc::clone(&entry.embedding));
        }
        None
    }

    fn evict_lru(guard: &mut HashMap<String, CacheEntry>) {
        if guard.is_empty() {
            return;
        }

        if let Some((key, _)) = guard
            .iter()
            .min_by_key(|(_, entry)| entry.last_used)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            guard.remove(&key);
        }
    }
}
