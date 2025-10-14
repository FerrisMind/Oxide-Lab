use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::Context;
use blake3::Hasher;
use dashmap::DashMap;

use lopdf::Document;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio::sync::{Mutex, broadcast};
use tokio::time::Instant;
use walkdir::WalkDir;

use tracing::{debug, error, info, instrument, trace, warn};

use super::chunking::{DocumentKind, SemanticChunker};
use crate::rag::inference::DocumentIndexer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchedDirectory {
    pub path: PathBuf,
    pub recursive: bool,
    #[serde(default)]
    pub default_kind: Option<DocumentKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileIngestionConfig {
    pub watch_roots: Vec<WatchedDirectory>,
    #[serde(default = "default_polling_interval_ms")]
    pub polling_interval_ms: u64,
    #[serde(default = "default_concurrency")]
    pub max_concurrency: usize,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default)]
    pub chunking: super::chunking::ChunkingConfig,
}

fn default_polling_interval_ms() -> u64 {
    1500
}

fn default_concurrency() -> usize {
    4
}

fn default_batch_size() -> usize {
    16
}

impl Default for FileIngestionConfig {
    fn default() -> Self {
        Self {
            watch_roots: Vec::new(),
            polling_interval_ms: default_polling_interval_ms(),
            max_concurrency: default_concurrency(),
            batch_size: default_batch_size(),
            chunking: super::chunking::ChunkingConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IngestionEvent {
    Started {
        timestamp: SystemTime,
    },
    FileQueued {
        path: PathBuf,
    },
    FileProcessing {
        path: PathBuf,
    },
    FileSkipped {
        path: PathBuf,
        reason: String,
    },
    ChunkStored {
        path: PathBuf,
        chunk_id: uuid::Uuid,
        chunk_index: usize,
    },
    Completed {
        timestamp: SystemTime,
        processed_files: usize,
        processed_chunks: usize,
    },
    Error {
        path: Option<PathBuf>,
        message: String,
    },
}

#[derive(Debug, Clone, Default)]
pub struct IngestionStats {
    pub last_run: Option<SystemTime>,
    pub processed_files: usize,
    pub processed_chunks: usize,
    pub skipped_files: usize,
    pub last_duration: Option<Duration>,
}

struct FileFingerprint {
    modified: SystemTime,
    _len: u64,
    hash: blake3::Hash,
}

struct QueuedFile {
    path: PathBuf,
    kind: DocumentKind,
}

#[derive(Clone, Debug)]
pub struct StoredDocument {
    pub indexed_at: SystemTime,
    pub chunks: usize,
}

struct IngestionInner {
    root_dir: PathBuf,
    config: FileIngestionConfig,
    chunker: SemanticChunker,
    indexer: Arc<dyn DocumentIndexer>,
    stats: Arc<tokio::sync::RwLock<IngestionStats>>,
    event_tx: broadcast::Sender<IngestionEvent>,
    fingerprints: DashMap<PathBuf, FileFingerprint>,
    queue_tx: mpsc::Sender<QueuedFile>,
    queue_rx: Mutex<mpsc::Receiver<QueuedFile>>,
    shutdown: tokio::sync::watch::Sender<bool>,
    shutdown_recv: tokio::sync::watch::Receiver<bool>,
    indexed_documents: DashMap<PathBuf, StoredDocument>,
    queued: DashMap<PathBuf, ()>,
}

#[derive(Clone)]
pub struct IngestionPipeline {
    inner: Arc<IngestionInner>,
}

impl IngestionPipeline {
    #[instrument(skip(indexer, stats), fields(root = %root_dir.display(), watchers = config.watch_roots.len()))]
    pub async fn initialise(
        root_dir: PathBuf,
        config: FileIngestionConfig,
        indexer: Arc<dyn DocumentIndexer>,
        stats: Arc<tokio::sync::RwLock<IngestionStats>>,
    ) -> anyhow::Result<Self> {
        let chunker = SemanticChunker::new(config.chunking.clone())
            .context("initialising semantic chunker failed")?;
        let (event_tx, _) = broadcast::channel(256);
        let (queue_tx, queue_rx) = mpsc::channel(config.batch_size * 4);
        let (shutdown, shutdown_recv) = tokio::sync::watch::channel(false);

        let inner = Arc::new(IngestionInner {
            root_dir,
            config,
            chunker,
            indexer,
            stats,
            event_tx,
            fingerprints: DashMap::new(),
            queue_tx,
            queue_rx: Mutex::new(queue_rx),
            shutdown,
            shutdown_recv,
            indexed_documents: DashMap::new(),
            queued: DashMap::new(),
        });

        let pipeline = Self { inner };
        pipeline.spawn_tasks().await;

        Ok(pipeline)
    }

    async fn spawn_tasks(&self) {
        for worker_id in 0..self.inner.config.max_concurrency.max(1) {
            self.spawn_worker(worker_id);
        }
        self.spawn_scanner();
    }

    fn spawn_worker(&self, worker_id: usize) {
        let inner = self.inner.clone();
        tokio::spawn(async move {
            let shutdown_rx = inner.shutdown_recv.clone();
            loop {
                let maybe_job = {
                    let mut rx = inner.queue_rx.lock().await;
                    rx.recv().await
                };
                match maybe_job {
                    Some(job) => {
                        if let Err(err) = inner.process_file(job).await {
                            let _ = inner.event_tx.send(IngestionEvent::Error {
                                path: Some(err.path.clone()),
                                message: err.error.to_string(),
                            });
                        }
                    }
                    None => break,
                }
                if *shutdown_rx.borrow() {
                    break;
                }
            }
            tracing::debug!("ingestion worker {} terminated", worker_id);
        });
    }

    fn spawn_scanner(&self) {
        let inner = self.inner.clone();
        tokio::spawn(async move {
            let interval = Duration::from_millis(inner.config.polling_interval_ms);
            let mut ticker = tokio::time::interval(interval);
            inner
                .event_tx
                .send(IngestionEvent::Started {
                    timestamp: SystemTime::now(),
                })
                .ok();
            let mut shutdown_rx = inner.shutdown_recv.clone();
            loop {
                let scan_start = Instant::now();
                if let Err(scan_err) = inner.scan_once().await {
                    tracing::error!("ingestion scan failed: {scan_err:?}");
                    inner
                        .event_tx
                        .send(IngestionEvent::Error {
                            path: None,
                            message: scan_err.to_string(),
                        })
                        .ok();
                }
                {
                    let mut stats = inner.stats.write().await;
                    stats.last_run = Some(SystemTime::now());
                    stats.last_duration = Some(scan_start.elapsed());
                }
                let mut should_break = false;
                tokio::select! {
                    _ = ticker.tick() => {}
                    changed = shutdown_rx.changed() => {
                        if changed.is_ok() && *shutdown_rx.borrow() {
                            should_break = true;
                        }
                    }
                }
                if should_break {
                    break;
                }
            }
            tracing::debug!("ingestion scanner terminated");
        });
    }

    pub fn subscribe(&self) -> broadcast::Receiver<IngestionEvent> {
        self.inner.event_tx.subscribe()
    }
    #[instrument(skip(self))]
    pub async fn trigger_rescan(&self) -> anyhow::Result<()> {
        info!("Manual rescan triggered");
        self.inner.scan_once().await
    }

    #[instrument(skip(self), fields(path = %path.display(), kind = ?kind))]
    pub async fn queue_document(&self, path: PathBuf, kind: DocumentKind) -> anyhow::Result<()> {
        if self.inner.queued.insert(path.clone(), ()).is_none() {
            self.inner
                .queue_tx
                .clone()
                .send(QueuedFile { path, kind })
                .await
                .map_err(|e| anyhow::anyhow!("failed to queue document: {e}"))?;
            info!("Queued document for ingestion");
        } else {
            debug!("Document already queued, skip duplicate request");
        }

        Ok(())
    }

    pub fn lookup_indexed(&self, path: &Path) -> Option<StoredDocument> {
        self.inner
            .indexed_documents
            .get(path)
            .map(|entry| entry.value().clone())
    }

    pub fn indexed_documents(&self) -> Vec<(PathBuf, StoredDocument)> {
        self.inner
            .indexed_documents
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    pub fn remove_indexed(&self, path: &Path) {
        self.inner.indexed_documents.remove(path);
        self.inner.queued.remove(path);
    }

    pub async fn shutdown(&self) {
        let _ = self.inner.shutdown.send(true);
    }
}

struct ProcessError {
    path: PathBuf,
    error: anyhow::Error,
}

impl IngestionInner {
    #[instrument(skip(self))]
    async fn scan_once(&self) -> anyhow::Result<()> {
        info!("Scanning watch roots");
        let mut queued_files = 0usize;

        for root in &self.config.watch_roots {
            if !root.path.exists() {
                debug!(root = %root.path.display(), "Skipping missing watch root");
                continue;
            }
            let walker = WalkDir::new(&root.path).follow_links(false);
            for entry in walker {
                let entry = match entry {
                    Ok(e) => e,
                    Err(err) => {
                        warn!("walkdir error: {err:?}");
                        continue;
                    }
                };
                if entry.file_type().is_file() {
                    if let Some(ext) = entry.path().extension().and_then(|s| s.to_str())
                        && !is_supported_extension(ext)
                    {
                        continue;
                    }
                    let path = entry.path().to_path_buf();
                    let fingerprint = match fingerprint(&path).await {
                        Ok(fp) => fp,
                        Err(err) => {
                            warn!(path = %path.display(), "fingerprint error: {err:?}");
                            continue;
                        }
                    };

                    let is_new_or_changed = self
                        .fingerprints
                        .get(&path)
                        .map(|fp| {
                            fp.hash != fingerprint.hash || fp.modified != fingerprint.modified
                        })
                        .unwrap_or(true);

                    if is_new_or_changed {
                        debug!(path = %path.display(), "Detected new or updated file");
                        self.event_tx
                            .send(IngestionEvent::FileQueued { path: path.clone() })
                            .ok();
                        self.queue_tx
                            .send(QueuedFile {
                                path: path.clone(),
                                kind: detect_kind(&path),
                            })
                            .await
                            .ok();
                        self.fingerprints.insert(path, fingerprint);
                        queued_files += 1;
                    }
                }
                if !root.recursive {
                    debug!(root = %root.path.display(), "Non-recursive watch root processed");
                    break;
                }
            }
        }
        info!(queued_files, "Scan cycle completed");
        Ok(())
    }

    #[instrument(skip(self, job), fields(path = %job.path.display(), kind = ?job.kind))]
    async fn process_file(&self, job: QueuedFile) -> Result<(), ProcessError> {
        self.event_tx
            .send(IngestionEvent::FileProcessing {
                path: job.path.clone(),
            })
            .ok();

        let job_path = job.path.clone();
        let started_at = std::time::Instant::now();
        let result = self.process_file_inner(job).await;
        self.queued.remove(&job_path);

        match result {
            Ok(_) => {
                info!(
                    elapsed_ms = started_at.elapsed().as_millis(),
                    "Completed document processing"
                );
                Ok(())
            }
            Err(error) => {
                error!(
                    error = ?error,
                    elapsed_ms = started_at.elapsed().as_millis(),
                    "Document processing failed"
                );
                Err(ProcessError {
                    path: job_path,
                    error,
                })
            }
        }
    }

    #[instrument(skip(self, job), fields(path = %job.path.display(), kind = ?job.kind))]
    async fn process_file_inner(&self, job: QueuedFile) -> anyhow::Result<()> {
        let text = extract_text(&job.path, job.kind)
            .await
            .with_context(|| format!("failed to extract text from {:?}", job.path))?;
        let text_len = text.len();

        let mut metadata = HashMap::new();
        metadata.insert(
            "path".to_string(),
            job.path
                .strip_prefix(&self.root_dir)
                .unwrap_or(&job.path)
                .to_string_lossy()
                .to_string(),
        );
        metadata.insert("source".into(), job.path.to_string_lossy().into_owned());
        metadata.insert("kind".into(), format!("{:?}", job.kind));

        let chunks = self.chunker.chunk(job.kind, &text, &metadata);
        let chunk_count = chunks.len();
        debug!(text_len, chunk_count, "Chunking produced segments");

        if chunks.is_empty() {
            info!("Skipping document because no chunks were generated");
            self.event_tx
                .send(IngestionEvent::FileSkipped {
                    path: job.path.clone(),
                    reason: "no chunks generated".into(),
                })
                .ok();
            let mut stats = self.stats.write().await;
            stats.skipped_files += 1;
            return Ok(());
        }

        self.indexer
            .index_document(&job.path, job.kind, chunks.clone())
            .await
            .with_context(|| format!("failed to index {:?}", job.path))?;
        info!(chunk_count, "Document indexed");

        self.indexed_documents.insert(
            job.path.clone(),
            StoredDocument {
                indexed_at: SystemTime::now(),
                chunks: chunk_count,
            },
        );

        for (idx, chunk) in chunks.iter().enumerate() {
            trace!(chunk_index = idx, chunk_id = %chunk.id, "Emitting chunk stored event");
            self.event_tx
                .send(IngestionEvent::ChunkStored {
                    path: job.path.clone(),
                    chunk_id: chunk.id,
                    chunk_index: idx,
                })
                .ok();
        }

        {
            let mut stats = self.stats.write().await;
            stats.processed_files += 1;
            stats.processed_chunks += chunk_count;
            trace!(
                total_files = stats.processed_files,
                total_chunks = stats.processed_chunks,
                "Updated ingestion statistics"
            );
        }

        Ok(())
    }
}

async fn fingerprint(path: &Path) -> anyhow::Result<FileFingerprint> {
    use tokio::fs::File;
    use tokio::io::{AsyncReadExt, BufReader};

    let metadata = tokio::fs::metadata(path).await?;
    let file = File::open(path).await?;
    let mut reader = BufReader::with_capacity(64 * 1024, file);
    let mut buffer = vec![0u8; 64 * 1024];
    let mut hasher = Hasher::new();

    loop {
        let read = reader.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(FileFingerprint {
        modified: metadata.modified().unwrap_or(SystemTime::now()),
        _len: metadata.len(),
        hash: hasher.finalize(),
    })
}

async fn extract_text(path: &Path, kind: DocumentKind) -> anyhow::Result<String> {
    match kind {
        DocumentKind::Pdf => extract_pdf_text(path),
        DocumentKind::Docx => extract_docx_text(path),
        DocumentKind::Markdown
        | DocumentKind::PlainText
        | DocumentKind::Code
        | DocumentKind::Unknown => Ok(tokio::fs::read_to_string(path).await?),
    }
}

fn extract_pdf_text(path: &Path) -> anyhow::Result<String> {
    let mut doc = Document::load(path)?;
    if doc.is_encrypted() {
        doc.decrypt("")?;
    }
    let page_numbers: Vec<u32> = doc
        .page_iter()
        .enumerate()
        .map(|(idx, _)| (idx as u32) + 1)
        .collect();
    Ok(doc.extract_text(&page_numbers)?)
}

fn extract_docx_text(path: &Path) -> anyhow::Result<String> {
    use quick_xml::Reader;
    use quick_xml::events::Event;
    use std::io::Read as _;
    use zip::read::ZipArchive;

    let file = std::fs::File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut document_xml = archive
        .by_name("word/document.xml")
        .context("document.xml missing in docx")?;
    let mut xml = String::new();
    document_xml.read_to_string(&mut xml)?;

    let mut reader = Reader::from_str(&xml);
    reader.trim_text(true);
    let mut buf = Vec::new();
    let mut out = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Text(e)) => {
                out.push_str(&e.unescape().unwrap_or_default());
                out.push(' ');
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow::anyhow!("docx xml parse error: {e}")),
            _ => {}
        }
        buf.clear();
    }

    Ok(out)
}

fn is_supported_extension(ext: &str) -> bool {
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "pdf"
            | "txt"
            | "md"
            | "markdown"
            | "docx"
            | "rs"
            | "py"
            | "js"
            | "ts"
            | "java"
            | "cpp"
            | "c"
            | "go"
            | "cs"
            | "swift"
            | "kt"
    )
}

fn detect_kind(path: &Path) -> DocumentKind {
    match path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
    {
        Some(ext) if ext == "pdf" => DocumentKind::Pdf,
        Some(ext) if ext == "txt" => DocumentKind::PlainText,
        Some(ext) if ext == "docx" => DocumentKind::Docx,
        Some(ext) if ext == "md" || ext == "markdown" => DocumentKind::Markdown,
        Some(ext)
            if matches!(
                ext.as_str(),
                "rs" | "py" | "js" | "ts" | "cpp" | "c" | "java" | "go" | "cs" | "swift" | "kt"
            ) =>
        {
            DocumentKind::Code
        }
        _ => DocumentKind::Unknown,
    }
}
