use aes_gcm::aead::{Aead, KeyInit};
use rand::{RngCore, rngs::OsRng};
use std::collections::HashMap;
use std::mem::transmute;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use anyhow::{Context, anyhow};
use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose::STANDARD_NO_PAD};
use parking_lot::Mutex;
use rusqlite::ffi::{sqlite3, sqlite3_api_routines, sqlite3_auto_extension};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};
#[cfg(feature = "rag-qdrant")]
use serde_json::Value;
use sqlite_vec::sqlite3_vec_init;
#[cfg(feature = "rag-qdrant")]
use tokio::task::JoinHandle;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use crate::rag::ingestion::DocumentKind;

use super::EmbeddingCache;

type SerializedEmbedding = (Option<Vec<u8>>, Option<Vec<u8>>);

static SQLITE_VEC_REGISTRATION: OnceLock<()> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub backend: VectorBackend,
    pub data_dir: PathBuf,
    pub sqlite: Option<SqliteConfig>,
    pub qdrant: Option<QdrantConfig>,
    pub index_encryption_key: Option<Vec<u8>>,
    pub quantize_vectors: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: VectorBackend::Sqlite,
            data_dir: PathBuf::from(".oxide-data/rag"),
            sqlite: Some(SqliteConfig::default()),
            qdrant: None,
            index_encryption_key: None,
            quantize_vectors: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VectorBackend {
    Sqlite,
    Qdrant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqliteConfig {
    pub filename: PathBuf,
    pub enable_vss: bool,
}

impl Default for SqliteConfig {
    fn default() -> Self {
        Self {
            filename: PathBuf::from("rag.db"),
            enable_vss: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantConfig {
    pub url: String,
    pub api_key: Option<String>,
    pub collection_name: String,
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1:6333".into(),
            api_key: None,
            collection_name: "oxide_rag".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DocumentPayload {
    pub path: PathBuf,
    pub kind: DocumentKind,
    pub fingerprint: Option<String>,
    pub chunks: Vec<PayloadChunk>,
}

#[derive(Debug, Clone)]
pub struct PayloadChunk {
    pub chunk: crate::rag::ingestion::DocumentChunk,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct VectorQuery {
    pub embedding: Vec<f32>,
    pub top_k: usize,
    pub filters: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct StoredChunk {
    pub chunk: crate::rag::ingestion::DocumentChunk,
    pub embedding: Vec<f32>,
    pub score: f32,
}

#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn persist(&self, payload: DocumentPayload) -> anyhow::Result<()>;
    async fn remove_document(&self, path: &Path) -> anyhow::Result<()>;
    async fn vector_search(&self, query: &VectorQuery) -> anyhow::Result<Vec<StoredChunk>>;
    async fn list_documents(&self) -> anyhow::Result<Vec<PathBuf>>;
    async fn get_chunk(&self, chunk_id: Uuid) -> anyhow::Result<Option<StoredChunk>>;
}

pub struct VectorStoreFactory {
    cache: EmbeddingCache,
}

impl VectorStoreFactory {
    pub fn new(cache: EmbeddingCache) -> Self {
        Self { cache }
    }

    #[instrument(skip(self, config), fields(backend = ?config.backend))]
    pub async fn build(&self, config: &StorageConfig) -> anyhow::Result<Box<dyn VectorStore>> {
        tokio::fs::create_dir_all(&config.data_dir).await?;
        debug!(data_dir = %config.data_dir.display(), "Initialising vector store backend");
        match config.backend {
            VectorBackend::Sqlite => {
                let sqlite_cfg = config.sqlite.clone().unwrap_or_default();
                let path = config.data_dir.join(sqlite_cfg.filename);
                let store = SqliteVectorStore::new(
                    path,
                    config.quantize_vectors,
                    config.index_encryption_key.clone(),
                    self.cache.clone(),
                )
                .context("initialising sqlite vector store")?;
                Ok(Box::new(store))
            }
            VectorBackend::Qdrant => {
                #[cfg(feature = "rag-qdrant")]
                {
                    let qdrant_cfg = config.qdrant.clone().unwrap_or_default();
                    let store = QdrantVectorStore::new(qdrant_cfg, self.cache.clone()).await?;
                    Ok(Box::new(store))
                }
                #[cfg(not(feature = "rag-qdrant"))]
                {
                    anyhow::bail!("Qdrant backend requested but feature `rag-qdrant` is disabled")
                }
            }
        }
    }
}

fn register_sqlite_vec_extension() -> anyhow::Result<()> {
    SQLITE_VEC_REGISTRATION.get_or_init(|| unsafe {
        let init_fn: unsafe extern "C" fn(
            *mut sqlite3,
            *mut *const i8,
            *const sqlite3_api_routines,
        ) -> i32 = transmute(sqlite3_vec_init as unsafe extern "C" fn() -> ());
        let _ = sqlite3_auto_extension(Some(init_fn));
    });
    Ok(())
}

struct SqliteVectorStore {
    connection: Arc<Mutex<Connection>>,
    quantize: bool,
    encryptor: Option<Encryptor>,
    cache: EmbeddingCache,
    vector_dimension: Arc<Mutex<Option<usize>>>,
}

impl SqliteVectorStore {
    #[instrument(skip(cache, encryption_key), fields(path = %path.display(), quantize = quantize))]
    fn new(
        path: PathBuf,
        quantize: bool,
        encryption_key: Option<Vec<u8>>,
        cache: EmbeddingCache,
    ) -> anyhow::Result<Self> {
        info!(path = %path.display(), "Opening SQLite vector store");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        register_sqlite_vec_extension()?;
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        initialise_schema(&conn)?;

        let existing_dim = read_stored_dimension(&conn)?;

        Ok(Self {
            connection: Arc::new(Mutex::new(conn)),
            quantize,
            encryptor: encryption_key.map(Encryptor::new).transpose()?,
            cache,
            vector_dimension: Arc::new(Mutex::new(existing_dim)),
        })
    }

    async fn with_conn<F, R>(&self, func: F) -> anyhow::Result<R>
    where
        F: FnOnce(&mut Connection) -> anyhow::Result<R> + Send + 'static,
        R: Send + 'static,
    {
        let connection = Arc::clone(&self.connection);
        tokio::task::spawn_blocking(move || {
            let mut conn = connection.lock();
            func(&mut conn)
        })
        .await?
    }
}

#[async_trait]
impl VectorStore for SqliteVectorStore {
    #[instrument(skip(self, payload), fields(path = %payload.path.display(), chunks = payload.chunks.len()))]
    async fn persist(&self, payload: DocumentPayload) -> anyhow::Result<()> {
        info!(
            path = %payload.path.display(),
            kind = ?payload.kind,
            chunk_count = payload.chunks.len(),
            "Persisting document into SQLite store"
        );
        let quantize = self.quantize;
        let encryptor = self.encryptor.clone();
        let cache = self.cache.clone();
        let vector_dimension = Arc::clone(&self.vector_dimension);
        self.with_conn(move |conn| {
            let DocumentPayload {
                path,
                kind,
                fingerprint,
                chunks,
            } = payload;
            let tx = conn.transaction()?;
            let doc_id = upsert_document(
                &tx,
                &path,
                kind,
                fingerprint,
            )?;

            for payload_chunk in chunks {
                debug!(chunk_id = %payload_chunk.chunk.id, index = payload_chunk.chunk.coordinate.index, "Writing chunk row");
                let serialized_metadata =
                    serialize_metadata(&payload_chunk.chunk.metadata, encryptor.as_ref())?;
                let (embedding_blob, quantized_blob) =
                    prepare_embedding_payload(&payload_chunk.embedding, quantize, encryptor.as_ref())?;

                tx.execute(
                    r#"
                    INSERT OR REPLACE INTO chunks (
                        id,
                        document_id,
                        chunk_index,
                        start_token,
                        end_token,
                        token_count,
                        text,
                        metadata,
                        embedding,
                        quantized
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                    "#,
                    params![
                        payload_chunk.chunk.id.to_string(),
                        doc_id,
                        payload_chunk.chunk.coordinate.index as i64,
                        payload_chunk.chunk.coordinate.start_token as i64,
                        payload_chunk.chunk.coordinate.end_token as i64,
                        payload_chunk.chunk.token_count as i64,
                        payload_chunk.chunk.text,
                        serialized_metadata,
                        embedding_blob,
                        quantized_blob,
                    ],
                )?;

                let rowid: i64 = tx.query_row(
                    "SELECT rowid FROM chunks WHERE id = ?1",
                    params![payload_chunk.chunk.id.to_string()],
                    |row| row.get(0),
                )?;

                let embedding_dim = payload_chunk.embedding.len();
                if embedding_dim == 0 {
                    anyhow::bail!("пустой вектор в chunk {}", payload_chunk.chunk.id);
                }

                let current_dim = {
                    let guard = vector_dimension.lock();
                    *guard
                };

                if let Some(dim) = current_dim {
                    if dim != embedding_dim {
                        anyhow::bail!(format!(
                            "Несовпадающая размерность embedding: ожидалось {dim}, получено {embedding_dim}"
                        ));
                    }
                } else {
                    ensure_vector_table_tx(&tx, embedding_dim)?;
                    let mut guard = vector_dimension.lock();
                    *guard = Some(embedding_dim);
                }

                upsert_vector_embedding(&tx, rowid, &payload_chunk.embedding)?;

                cache.insert(
                    cache_key(&path, payload_chunk.chunk.coordinate.index),
                    payload_chunk.embedding.clone(),
                );
            }
            tx.commit()?;
            info!(path = %path.display(), "SQLite transaction committed");
            Ok(())
        })
        .await
    }

    #[instrument(skip(self), fields(path = %path.display()))]
    async fn remove_document(&self, path: &Path) -> anyhow::Result<()> {
        info!(path = %path.display(), "Removing document from SQLite store");
        let path = path.to_path_buf();
        self.with_conn(move |conn| {
            if chunk_index_exists(conn)? {
                conn.execute(
                    r#"
                    DELETE FROM chunk_index
                    WHERE rowid IN (
                        SELECT rowid FROM chunks
                        WHERE document_id = (SELECT id FROM documents WHERE path = ?1)
                    )
                    "#,
                    params![path.to_string_lossy()],
                )?;
            }
            conn.execute(
                "DELETE FROM chunks WHERE document_id = (SELECT id FROM documents WHERE path = ?1)",
                params![path.to_string_lossy()],
            )?;
            conn.execute(
                "DELETE FROM documents WHERE path = ?1",
                params![path.to_string_lossy()],
            )?;
            Ok(())
        })
        .await
    }

    #[instrument(skip(self, query), fields(top_k = query.top_k, filter_count = query.filters.len()))]
    async fn vector_search(&self, query: &VectorQuery) -> anyhow::Result<Vec<StoredChunk>> {
        let query = query.clone();
        let top_k = query.top_k;
        let filter_count = query.filters.len();
        debug!(top_k, filter_count, "Executing vector search");
        let encryptor = self.encryptor.clone();
        let quantize = self.quantize;
        let started = std::time::Instant::now();
        let results = self
            .with_conn(move |conn| {
                if chunk_index_exists(conn)? {
                    vector_search_with_index(conn, &query, encryptor.as_ref(), quantize)
                } else {
                    brute_force_vector_search(conn, &query, encryptor.as_ref(), quantize)
                }
            })
            .await?;
        debug!(
            top_k,
            filter_count,
            result_count = results.len(),
            elapsed_ms = started.elapsed().as_millis(),
            "Vector search completed"
        );
        Ok(results)
    }

    #[instrument(skip(self))]
    async fn list_documents(&self) -> anyhow::Result<Vec<PathBuf>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare("SELECT path FROM documents")?;
            let rows = stmt
                .query_map([], |row| row.get::<_, String>(0))
                .context("query documents")?;
            let mut paths = Vec::new();
            for row in rows {
                paths.push(PathBuf::from(row?));
            }
            info!(count = paths.len(), "Fetched document paths");
            Ok(paths)
        })
        .await
    }

    #[instrument(skip(self), fields(chunk_id = %chunk_id))]
    async fn get_chunk(&self, chunk_id: Uuid) -> anyhow::Result<Option<StoredChunk>> {
        let id = chunk_id.to_string();
        let encryptor = self.encryptor.clone();
        let quantize = self.quantize;
        self.with_conn(move |conn| {
            let mut stmt = conn.prepare(
                r#"
                SELECT
                    c.id,
                    c.chunk_index,
                    c.start_token,
                    c.end_token,
                    c.token_count,
                    c.text,
                    c.metadata,
                    c.embedding,
                    c.quantized
                FROM chunks c
                WHERE c.id = ?1
                "#,
            )?;

            let row_opt = stmt
                .query_row(params![id], |row| {
                    let metadata_raw: String = row.get(6)?;
                    let metadata = deserialize_metadata(&metadata_raw, encryptor.as_ref())
                        .map_err(map_anyhow_to_rusqlite)?;
                    let chunk = crate::rag::ingestion::DocumentChunk {
                        id: Uuid::parse_str(row.get::<_, String>(0)?.as_str()).unwrap_or(chunk_id),
                        coordinate: crate::rag::ingestion::ChunkCoordinate {
                            section: metadata
                                .get("section")
                                .and_then(|s| s.parse::<usize>().ok())
                                .unwrap_or_default(),
                            index: row.get::<_, i64>(1)? as usize,
                            start_token: row.get::<_, i64>(2)? as usize,
                            end_token: row.get::<_, i64>(3)? as usize,
                        },
                        text: row.get(5)?,
                        token_count: row.get::<_, i64>(4)? as usize,
                        metadata,
                    };

                    let embedding_blob: Option<Vec<u8>> = row.get(7)?;
                    let quantized_blob: Option<Vec<u8>> = row.get(8)?;
                    let embedding = restore_embedding(
                        embedding_blob,
                        quantized_blob,
                        encryptor.as_ref(),
                        quantize,
                    )
                    .map_err(map_anyhow_to_rusqlite)?;

                    Ok(StoredChunk {
                        chunk,
                        embedding,
                        score: 1.0,
                    })
                })
                .optional()?;
            debug!(found = row_opt.is_some(), "Chunk lookup finished");
            Ok(row_opt)
        })
        .await
    }
}

#[cfg(feature = "rag-qdrant")]
struct QdrantVectorStore {
    client: qdrant_client::client::QdrantClient,
    collection_name: String,
    cache: EmbeddingCache,
    background: JoinHandle<anyhow::Result<()>>,
}

#[cfg(feature = "rag-qdrant")]
impl QdrantVectorStore {
    async fn new(config: QdrantConfig, cache: EmbeddingCache) -> anyhow::Result<Self> {
        let mut client_config = qdrant_client::client::QdrantClientConfig::from_url(&config.url);
        if let Some(key) = &config.api_key {
            client_config.api_key = Some(key.clone());
        }
        let client = qdrant_client::client::QdrantClient::new(Some(client_config))?;
        let collection_name = config.collection_name.clone();
        let client_clone = client.clone();
        let collection_name_clone = collection_name.clone();
        let background = tokio::spawn(async move {
            client_clone
                .create_collection(&qdrant_client::qdrant::CreateCollection {
                    collection_name: collection_name_clone,
                    vectors_config: Some(qdrant_client::qdrant::VectorsConfig {
                        config: Some(qdrant_client::qdrant::vectors_config::Config::Params(
                            qdrant_client::qdrant::VectorParams {
                                size: 1024,
                                distance: qdrant_client::qdrant::Distance::Cosine as i32,
                                ..Default::default()
                            },
                        )),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await
                .ok();
            Ok(())
        });
        Ok(Self {
            client,
            collection_name,
            cache,
            background,
        })
    }
}

#[cfg(feature = "rag-qdrant")]
#[async_trait]
impl VectorStore for QdrantVectorStore {
    async fn persist(&self, payload: DocumentPayload) -> anyhow::Result<()> {
        self.background.await??;
        let points: Vec<qdrant_client::qdrant::PointStruct> = payload
            .chunks
            .iter()
            .map(|chunk| {
                self.cache.insert(
                    cache_key(&payload.path, chunk.chunk.coordinate.index),
                    chunk.embedding.clone(),
                );
                qdrant_client::qdrant::PointStruct::new(
                    chunk.chunk.id.to_string(),
                    chunk.embedding.clone(),
                    [
                        (
                            "path".into(),
                            Value::String(payload.path.to_string_lossy().into()),
                        ),
                        (
                            "chunk_index".into(),
                            Value::Number(chunk.chunk.coordinate.index.into()),
                        ),
                        (
                            "token_count".into(),
                            Value::Number(chunk.chunk.token_count.into()),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                )
            })
            .collect();
        self.client
            .upsert_points_blocking(&self.collection_name, points, None)
            .await?;
        Ok(())
    }

    async fn remove_document(&self, path: &Path) -> anyhow::Result<()> {
        self.client
            .delete(
                &self.collection_name,
                &qdrant_client::qdrant::DeletePoints {
                    points_selector: Some(qdrant_client::qdrant::PointsSelector::Filter(
                        qdrant_client::qdrant::Filter {
                            must: vec![qdrant_client::qdrant::Condition {
                                condition_one_of: Some(
                                    qdrant_client::qdrant::condition::ConditionOneOf::Field(
                                        qdrant_client::qdrant::FieldCondition {
                                            key: "path".into(),
                                            r#match: Some(
                                                qdrant_client::qdrant::r#match::Match::Text(
                                                    qdrant_client::qdrant::MatchText {
                                                        text: path.to_string_lossy().into(),
                                                    },
                                                ),
                                            ),
                                            ..Default::default()
                                        },
                                    ),
                                ),
                            }],
                            ..Default::default()
                        },
                    )),
                    ..Default::default()
                },
            )
            .await?;
        Ok(())
    }

    async fn vector_search(&self, query: &VectorQuery) -> anyhow::Result<Vec<StoredChunk>> {
        let started = std::time::Instant::now();
        let result = self
            .client
            .search_points(
                &self.collection_name,
                query.embedding.clone(),
                query.top_k as u64,
                None,
            )
            .await?;
        let mut out = Vec::new();
        for scored_point in result.result {
            let chunk_id = Uuid::parse_str(&scored_point.id)?;
            if let Some(embedding) = self
                .cache
                .lookup(&chunk_id.to_string())
                .map(|v| (*v).clone())
            {
                let metadata = scored_point
                    .payload
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_string()))
                    .collect::<HashMap<_, _>>();
                out.push(StoredChunk {
                    chunk: crate::rag::ingestion::DocumentChunk {
                        id: chunk_id,
                        coordinate: crate::rag::ingestion::ChunkCoordinate {
                            section: metadata
                                .get("section")
                                .and_then(|s| s.parse().ok())
                                .unwrap_or_default(),
                            index: metadata
                                .get("chunk_index")
                                .and_then(|s| s.parse().ok())
                                .unwrap_or_default(),
                            start_token: 0,
                            end_token: 0,
                        },
                        text: String::new(),
                        token_count: metadata
                            .get("token_count")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or_default(),
                        metadata,
                    },
                    embedding,
                    score: scored_point.score,
                });
            }
        }
        debug!(
            top_k = query.top_k,
            filter_count = query.filters.len(),
            result_count = out.len(),
            elapsed_ms = started.elapsed().as_millis(),
            "Qdrant vector search completed"
        );
        Ok(out)
    }

    async fn list_documents(&self) -> anyhow::Result<Vec<PathBuf>> {
        let response = self
            .client
            .scroll(
                &self.collection_name,
                None,
                None,
                None,
                &["path".into()],
                None,
                None,
            )
            .await?;
        let mut paths = Vec::new();
        for point in response.result {
            if let Some(Value::String(path)) = point.payload.get("path") {
                paths.push(PathBuf::from(path));
            }
        }
        Ok(paths)
    }

    async fn get_chunk(&self, chunk_id: Uuid) -> anyhow::Result<Option<StoredChunk>> {
        if let Some(embedding) = self.cache.lookup(&chunk_id.to_string()) {
            return Ok(Some(StoredChunk {
                chunk: crate::rag::ingestion::DocumentChunk {
                    id: chunk_id,
                    coordinate: crate::rag::ingestion::ChunkCoordinate {
                        section: 0,
                        index: 0,
                        start_token: 0,
                        end_token: 0,
                    },
                    text: String::new(),
                    token_count: 0,
                    metadata: HashMap::new(),
                },
                embedding: (*embedding).clone(),
                score: 1.0,
            }));
        }
        Ok(None)
    }
}

fn initialise_schema(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;
        CREATE TABLE IF NOT EXISTS documents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT UNIQUE NOT NULL,
            kind TEXT NOT NULL,
            fingerprint TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS chunks (
            id TEXT PRIMARY KEY,
            document_id INTEGER NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
            chunk_index INTEGER NOT NULL,
            start_token INTEGER NOT NULL,
            end_token INTEGER NOT NULL,
            token_count INTEGER NOT NULL,
            text TEXT NOT NULL,
            metadata TEXT NOT NULL,
            embedding BLOB,
            quantized BLOB,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_chunks_document_id ON chunks(document_id);

        CREATE TABLE IF NOT EXISTS vector_metadata (
            key TEXT PRIMARY KEY,
            value INTEGER NOT NULL
        );
        "#,
    )?;
    Ok(())
}

fn upsert_document(
    conn: &Connection,
    path: &Path,
    kind: DocumentKind,
    fingerprint: Option<String>,
) -> anyhow::Result<i64> {
    conn.execute(
        r#"
        INSERT INTO documents(path, kind, fingerprint)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(path) DO UPDATE SET
            kind = excluded.kind,
            fingerprint = excluded.fingerprint,
            updated_at = CURRENT_TIMESTAMP
        "#,
        params![path.to_string_lossy(), format!("{kind:?}"), fingerprint],
    )?;
    let id: i64 = conn.query_row(
        "SELECT id FROM documents WHERE path = ?1",
        params![path.to_string_lossy()],
        |row| row.get(0),
    )?;
    Ok(id)
}

#[derive(Clone)]
struct Encryptor {
    cipher: aes_gcm::Aes256Gcm,
}

impl Encryptor {
    fn new(key: Vec<u8>) -> anyhow::Result<Self> {
        let key_bytes = ensure_key_len(key)?;
        let cipher = aes_gcm::Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| anyhow!(e))?;
        Ok(Self { cipher })
    }

    fn encrypt(&self, plaintext: &[u8]) -> anyhow::Result<Vec<u8>> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        #[allow(deprecated)]
        let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);
        let mut ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| anyhow!(e))?;
        let mut out = nonce_bytes.to_vec();
        out.append(&mut ciphertext);
        Ok(out)
    }

    fn decrypt(&self, ciphertext: &[u8]) -> anyhow::Result<Vec<u8>> {
        if ciphertext.len() < 12 {
            anyhow::bail!("ciphertext too short");
        }
        let (nonce_bytes, payload) = ciphertext.split_at(12);
        #[allow(deprecated)]
        let nonce = aes_gcm::Nonce::from_slice(nonce_bytes);
        self.cipher.decrypt(nonce, payload).map_err(|e| anyhow!(e))
    }
}

fn ensure_key_len(mut key: Vec<u8>) -> anyhow::Result<[u8; 32]> {
    if key.len() == 32 {
        return Ok(key.try_into().unwrap());
    }
    if key.len() > 32 {
        key.truncate(32);
        return Ok(key.try_into().unwrap());
    }
    let mut padded = [0u8; 32];
    padded[..key.len()].copy_from_slice(&key);
    Ok(padded)
}

fn prepare_embedding_payload(
    embedding: &[f32],
    quantize: bool,
    encryptor: Option<&Encryptor>,
) -> anyhow::Result<SerializedEmbedding> {
    let mut raw = Vec::new();
    for value in embedding {
        raw.extend_from_slice(&value.to_le_bytes());
    }
    let encrypted = match encryptor {
        Some(enc) => Some(enc.encrypt(&raw)?),
        None => Some(raw),
    };

    let quantized = if quantize {
        let (quant, scale) = quantize_vector(embedding);
        let mut payload = Vec::with_capacity(4 + quant.len());
        payload.extend_from_slice(&scale.to_le_bytes());
        payload.extend(quant.iter().map(|v| *v as u8));
        Some(payload)
    } else {
        None
    };

    Ok((encrypted, quantized))
}

fn restore_embedding(
    encrypted: Option<Vec<u8>>,
    quantized: Option<Vec<u8>>,
    encryptor: Option<&Encryptor>,
    quantize: bool,
) -> anyhow::Result<Vec<f32>> {
    if let Some(encrypted) = encrypted {
        let bytes = match encryptor {
            Some(enc) => enc.decrypt(&encrypted)?,
            None => encrypted,
        };
        bytes
            .chunks_exact(4)
            .map(|chunk| Ok(f32::from_le_bytes(chunk.try_into().unwrap())))
            .collect::<anyhow::Result<Vec<_>>>()
    } else if quantize {
        if let Some(blob) = quantized {
            if blob.len() < 4 {
                anyhow::bail!("quantized blob too short");
            }
            let (scale_bytes, payload) = blob.split_at(4);
            let scale = f32::from_le_bytes(scale_bytes.try_into().unwrap());
            Ok(payload
                .iter()
                .map(|value| (*value as i8 as f32) * scale)
                .collect())
        } else {
            anyhow::bail!("missing embedding data");
        }
    } else {
        anyhow::bail!("no embedding data stored");
    }
}

fn metadata_matches_filters(
    metadata: &HashMap<String, String>,
    filters: &HashMap<String, String>,
) -> bool {
    filters.iter().all(|(key, expected)| {
        metadata
            .get(key)
            .map(|value| value == expected)
            .unwrap_or(false)
    })
}

fn serialize_metadata(
    metadata: &HashMap<String, String>,
    encryptor: Option<&Encryptor>,
) -> anyhow::Result<String> {
    if let Some(enc) = encryptor {
        let bytes = serde_json::to_vec(metadata)?;
        let encrypted = enc.encrypt(&bytes)?;
        Ok(format!("enc:{}", STANDARD_NO_PAD.encode(encrypted)))
    } else {
        Ok(serde_json::to_string(metadata)?)
    }
}

fn deserialize_metadata(
    raw: &str,
    encryptor: Option<&Encryptor>,
) -> anyhow::Result<HashMap<String, String>> {
    if let Some(enc) = encryptor {
        let payload = if let Some(stripped) = raw.strip_prefix("enc:") {
            stripped
        } else {
            warn!(
                "Найдена нешифрованная метадата при активном шифровании, выполняю парсинг без дешифровки"
            );
            return Ok(serde_json::from_str(raw)?);
        };
        let bytes = STANDARD_NO_PAD
            .decode(payload.as_bytes())
            .map_err(|e| anyhow!(e))?;
        let decrypted = enc.decrypt(&bytes)?;
        Ok(serde_json::from_slice(&decrypted)?)
    } else {
        Ok(serde_json::from_str(raw)?)
    }
}

fn ensure_vector_table_tx(tx: &Transaction<'_>, dimension: usize) -> anyhow::Result<()> {
    if dimension == 0 {
        anyhow::bail!("некорректная размерность вектора (0)");
    }

    if chunk_index_exists(tx)? {
        if let Some(existing) = read_stored_dimension(tx)? {
            anyhow::ensure!(
                existing == dimension,
                "embedding dimension mismatch: stored {existing}, incoming {dimension}"
            );
        } else {
            tx.execute(
                "INSERT OR REPLACE INTO vector_metadata(key, value) VALUES ('chunk_index_dim', ?1)",
                params![dimension as i64],
            )?;
        }
        return Ok(());
    }

    tx.execute(
        &format!(
            "CREATE VIRTUAL TABLE IF NOT EXISTS chunk_index USING vec0(embedding float[{dimension}])"
        ),
        [],
    )?;
    tx.execute(
        "INSERT OR REPLACE INTO vector_metadata(key, value) VALUES ('chunk_index_dim', ?1)",
        params![dimension as i64],
    )?;
    Ok(())
}

fn upsert_vector_embedding(
    tx: &Transaction<'_>,
    rowid: i64,
    embedding: &[f32],
) -> anyhow::Result<()> {
    let json = serde_json::to_string(embedding)?;
    tx.execute(
        "INSERT OR REPLACE INTO chunk_index(rowid, embedding) VALUES (?1, ?2)",
        params![rowid, json],
    )?;
    Ok(())
}

fn chunk_index_exists(conn: &Connection) -> anyhow::Result<bool> {
    let exists: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE name = 'chunk_index' LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()?;
    Ok(exists.is_some())
}

fn read_stored_dimension(conn: &Connection) -> anyhow::Result<Option<usize>> {
    let value: Option<i64> = conn
        .query_row(
            "SELECT value FROM vector_metadata WHERE key = 'chunk_index_dim'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    Ok(value.map(|v| v as usize))
}

fn oversample_limit(top_k: usize) -> usize {
    let base = top_k.saturating_mul(4);
    base.max(top_k + 8).max(16)
}

fn vector_search_with_index(
    conn: &Connection,
    query: &VectorQuery,
    encryptor: Option<&Encryptor>,
    quantize: bool,
) -> anyhow::Result<Vec<StoredChunk>> {
    let limit = oversample_limit(query.top_k);
    let embedding_json = serde_json::to_string(&query.embedding)?;
    let mut stmt = conn.prepare(
        r#"
        SELECT
            c.id,
            c.chunk_index,
            c.start_token,
            c.end_token,
            c.token_count,
            c.text,
            c.metadata,
            c.embedding,
            c.quantized,
            sub.distance
        FROM (
            SELECT
                rowid,
                distance
            FROM chunk_index
            WHERE embedding MATCH ?1
            ORDER BY distance
            LIMIT ?2
        ) AS sub
        JOIN chunks c ON c.rowid = sub.rowid
        ORDER BY sub.distance
        "#,
    )?;

    let rows = stmt.query_map(params![embedding_json, limit as i64], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, Option<Vec<u8>>>(7)?,
            row.get::<_, Option<Vec<u8>>>(8)?,
            row.get::<_, f32>(9)?,
        ))
    })?;

    let mut scored: Vec<StoredChunk> = Vec::new();
    for row in rows {
        let (
            id_str,
            chunk_index,
            start_token,
            end_token,
            token_count,
            text,
            metadata_raw,
            embedding_blob,
            quantized_blob,
            _distance,
        ) = row?;

        let metadata = match deserialize_metadata(&metadata_raw, encryptor) {
            Ok(meta) => meta,
            Err(err) => {
                warn!("Не удалось расшифровать/десериализовать метаданные блока {id_str}: {err:?}");
                continue;
            }
        };

        if !metadata_matches_filters(&metadata, &query.filters) {
            continue;
        }

        let embedding = match restore_embedding(embedding_blob, quantized_blob, encryptor, quantize)
        {
            Ok(vec) => vec,
            Err(err) => {
                warn!("Не удалось восстановить embedding для блока {id_str}: {err:?}");
                continue;
            }
        };

        let chunk = crate::rag::ingestion::DocumentChunk {
            id: Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::new_v4()),
            coordinate: crate::rag::ingestion::ChunkCoordinate {
                section: metadata
                    .get("section")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or_default(),
                index: chunk_index as usize,
                start_token: start_token as usize,
                end_token: end_token as usize,
            },
            text,
            token_count: token_count as usize,
            metadata,
        };

        let score = cosine_similarity(&query.embedding, &embedding);
        scored.push(StoredChunk {
            chunk,
            embedding,
            score,
        });
    }

    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored.truncate(query.top_k);
    Ok(scored)
}

fn brute_force_vector_search(
    conn: &Connection,
    query: &VectorQuery,
    encryptor: Option<&Encryptor>,
    quantize: bool,
) -> anyhow::Result<Vec<StoredChunk>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT
            c.id,
            c.chunk_index,
            c.start_token,
            c.end_token,
            c.token_count,
            c.text,
            c.metadata,
            c.embedding,
            c.quantized
        FROM chunks c
        INNER JOIN documents d ON d.id = c.document_id
        ORDER BY c.id
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, i64>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, String>(6)?,
            row.get::<_, Option<Vec<u8>>>(7)?,
            row.get::<_, Option<Vec<u8>>>(8)?,
        ))
    })?;

    let mut scored = Vec::new();
    for row in rows {
        let (
            id_str,
            chunk_index,
            start_token,
            end_token,
            token_count,
            text,
            metadata_raw,
            embedding_blob,
            quantized_blob,
        ) = row?;

        let metadata = match deserialize_metadata(&metadata_raw, encryptor) {
            Ok(meta) => meta,
            Err(err) => {
                warn!("Не удалось расшифровать/десериализовать метаданные блока {id_str}: {err:?}");
                continue;
            }
        };

        if !metadata_matches_filters(&metadata, &query.filters) {
            continue;
        }

        let embedding = match restore_embedding(embedding_blob, quantized_blob, encryptor, quantize)
        {
            Ok(vec) => vec,
            Err(err) => {
                warn!("Не удалось восстановить embedding для блока {id_str}: {err:?}");
                continue;
            }
        };

        let chunk = crate::rag::ingestion::DocumentChunk {
            id: Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::new_v4()),
            coordinate: crate::rag::ingestion::ChunkCoordinate {
                section: metadata
                    .get("section")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or_default(),
                index: chunk_index as usize,
                start_token: start_token as usize,
                end_token: end_token as usize,
            },
            text,
            token_count: token_count as usize,
            metadata,
        };

        let score = cosine_similarity(&query.embedding, &embedding);
        scored.push(StoredChunk {
            chunk,
            embedding,
            score,
        });
    }

    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored.truncate(query.top_k);
    Ok(scored)
}

fn quantize_vector(v: &[f32]) -> (Vec<i8>, f32) {
    let mut max = 1e-12f32;
    for value in v {
        max = max.max(value.abs());
    }
    let scale = max / 127.0;
    let quantized = v
        .iter()
        .map(|value| (value / scale).clamp(-127.0, 127.0) as i8)
        .collect();
    (quantized, scale)
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a = (a.iter().map(|x| x * x).sum::<f32>()).sqrt();
    let norm_b = (b.iter().map(|x| x * x).sum::<f32>()).sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

fn cache_key(path: &Path, chunk_index: usize) -> String {
    format!("{}#{chunk_index}", path.to_string_lossy())
}

fn map_anyhow_to_rusqlite(err: anyhow::Error) -> rusqlite::Error {
    let boxed: Box<dyn std::error::Error + Send + Sync> = err.into();
    rusqlite::Error::ToSqlConversionFailure(boxed)
}
