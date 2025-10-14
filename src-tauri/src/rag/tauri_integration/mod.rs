use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tauri::Emitter;
use tauri::{self, AppHandle, State};
use tracing::{debug, error, info, instrument, trace, warn};

use crate::rag::RagServiceState;
use crate::rag::ingestion::{DocumentKind, IngestionEvent, StoredDocument};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentInfo {
    pub path: String,
    pub kind: String,
    pub chunks_count: usize,
    pub indexed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagHitResponse {
    pub text: String,
    pub score: f32,
    pub document_path: String,
    pub chunk_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagResponse {
    pub hits: Vec<RagHitResponse>,
    pub query_time_ms: u64,
}

#[tauri::command]
#[instrument(skip(app_handle, state), fields(path = %path))]
pub async fn load_document(
    path: String,
    app_handle: AppHandle,
    state: State<'_, RagServiceState>,
) -> Result<DocumentInfo, String> {
    info!("Received request to ingest document");
    let started_at = Instant::now();
    let path_buf = PathBuf::from(&path);
    if !path_buf.exists() {
        error!("Document not found on disk");
        return Err(format!("���㬥�� �� ������: {path}"));
    }

    let service = state.read().await;
    let ingestion = service.ingestion();
    drop(service);

    if let Some(stats) = ingestion.lookup_indexed(&path_buf) {
        info!("Document already indexed, returning cached metadata");
        emit_stage(&app_handle, "complete", 1.0, &path, None);
        let info = build_document_info(&path_buf, &stats);
        info!(
            elapsed_ms = started_at.elapsed().as_millis(),
            chunks = info.chunks_count,
            "Served cached document metadata"
        );
        return Ok(info);
    }

    let mut receiver = ingestion.subscribe();
    ingestion
        .queue_document(
            path_buf.clone(),
            DocumentKind::from_path(&path_buf).unwrap_or(DocumentKind::Unknown),
        )
        .await
        .map_err(|e| {
            error!(error = ?e, "Failed to queue document for ingestion");
            e.to_string()
        })?;
    debug!("Document queued for ingestion pipeline");

    while let Ok(event) = receiver.recv().await {
        match event {
            IngestionEvent::FileQueued { path: event_path } if event_path == path_buf => {
                debug!(
                    path = %event_path.display(),
                    "File queued event received"
                );
                emit_stage(&app_handle, "queued", 0.1, &path, None);
            }
            IngestionEvent::FileProcessing { path: event_path } if event_path == path_buf => {
                debug!(
                    path = %event_path.display(),
                    "File processing event received"
                );
                emit_stage(&app_handle, "processing", 0.4, &path, None);
            }
            IngestionEvent::ChunkStored {
                path: event_path, ..
            } if event_path == path_buf => {
                trace!(
                    path = %event_path.display(),
                    "Chunk stored event received"
                );
                emit_stage(&app_handle, "indexing", 0.75, &path, None);
            }
            IngestionEvent::Completed {
                processed_files, ..
            } if processed_files > 0 => {
                if ingestion.lookup_indexed(&path_buf).is_some() {
                    info!(
                        elapsed_ms = started_at.elapsed().as_millis(),
                        "Document ingestion completed successfully"
                    );
                    emit_stage(&app_handle, "complete", 1.0, &path, None);
                    break;
                }
            }
            IngestionEvent::Error {
                path: Some(event_path),
                message,
            } if event_path == path_buf => {
                error!(
                    message = %message,
                    elapsed_ms = started_at.elapsed().as_millis(),
                    "Ingestion pipeline reported error"
                );
                emit_stage(&app_handle, "error", 0.0, &path, Some(&message));
                return Err(message);
            }
            _ => {}
        }

        if ingestion.lookup_indexed(&path_buf).is_some() {
            info!("Document marked as indexed by ingestion cache");
            emit_stage(&app_handle, "complete", 1.0, &path, None);
            break;
        }
    }

    if let Some(stats) = ingestion.lookup_indexed(&path_buf) {
        let info = build_document_info(&path_buf, &stats);
        info!(
            elapsed_ms = started_at.elapsed().as_millis(),
            chunks = info.chunks_count,
            "Document ingestion flow finalised"
        );
        Ok(info)
    } else {
        error!("Ingestion succeeded but document stats not found");
        Err("���㬥�� �� �� �ந�����஢��".to_string())
    }
}

#[tauri::command]
#[instrument(skip(state), fields(query = %truncate_query(&query), top_k = top_k.unwrap_or(5)))]
pub async fn query_rag(
    query: String,
    top_k: Option<usize>,
    state: State<'_, RagServiceState>,
) -> Result<RagResponse, String> {
    let top_k = top_k.unwrap_or(5);
    info!(top_k, query_preview = %truncate_query(&query), "Processing RAG query");

    let start = std::time::Instant::now();
    let service = state.read().await;
    let result = service
        .query(crate::rag::inference::RagQuery {
            text: query.clone(),
            top_k,
            filters: std::collections::HashMap::new(),
        })
        .await
        .map_err(|e| {
            error!(error = ?e, "RAG query failed");
            e.to_string()
        })?;

    let mut hits = Vec::with_capacity(result.hits.len());
    for (idx, hit) in result.hits.into_iter().enumerate() {
        let document_path = hit
            .chunk
            .metadata
            .get("source")
            .cloned()
            .unwrap_or_default();
        debug!(
            index = idx,
            combined_score = hit.combined_score,
            vector_score = hit.vector_score,
            bm25_score = hit.bm25_score,
            rerank_score = hit.rerank_score,
            path = %document_path,
            chunk_index = hit.chunk.coordinate.index,
            preview = %truncate_snippet(&hit.chunk.text),
            "RAG hit scored"
        );
        hits.push(RagHitResponse {
            text: hit.chunk.text,
            score: hit.combined_score,
            document_path,
            chunk_index: hit.chunk.coordinate.index,
        });
    }

    let elapsed = start.elapsed();
    info!(
        hits = hits.len(),
        elapsed_ms = elapsed.as_millis(),
        "RAG query completed"
    );

    Ok(RagResponse {
        hits,
        query_time_ms: elapsed.as_millis() as u64,
    })
}

#[tauri::command]
#[instrument(skip(state))]
pub async fn list_rag_documents(
    state: State<'_, RagServiceState>,
) -> Result<Vec<DocumentInfo>, String> {
    let service = state.read().await;
    let ingestion = service.ingestion();
    let storage = service.storage();
    drop(service);

    let indexed = ingestion.indexed_documents();

    if !indexed.is_empty() {
        let mut documents = indexed
            .into_iter()
            .map(|(path, stats)| build_document_info(&path, &stats))
            .collect::<Vec<_>>();
        documents.sort_by(|a, b| a.path.cmp(&b.path));
        info!(
            count = documents.len(),
            "Returning indexed documents from ingestion cache"
        );
        return Ok(documents);
    }

    let paths = storage.list_documents().await.map_err(|e| {
        error!(error = ?e, "Failed to list documents from storage");
        e.to_string()
    })?;
    let mut fallback_docs = Vec::with_capacity(paths.len());

    for path in paths {
        if let Some(stats) = ingestion.lookup_indexed(&path) {
            fallback_docs.push(build_document_info(&path, &stats));
        } else {
            let kind = DocumentKind::from_path(&path)
                .unwrap_or(DocumentKind::Unknown)
                .as_str()
                .to_string();
            fallback_docs.push(DocumentInfo {
                path: path.to_string_lossy().into_owned(),
                kind,
                chunks_count: 0,
                indexed_at: Utc::now().to_rfc3339(),
            });
        }
    }

    fallback_docs.sort_by(|a, b| a.path.cmp(&b.path));
    info!(
        count = fallback_docs.len(),
        "Returning indexed documents from storage listing"
    );
    Ok(fallback_docs)
}

#[tauri::command]
#[instrument(skip(state), fields(path = %path))]
pub async fn delete_rag_document(
    path: String,
    state: State<'_, RagServiceState>,
) -> Result<(), String> {
    let service = state.read().await;
    let storage = service.storage();
    let ingestion = service.ingestion();
    drop(service);

    let path_buf = PathBuf::from(&path);
    storage.remove_document(&path_buf).await.map_err(|e| {
        error!(error = ?e, "Failed to remove document from storage");
        e.to_string()
    })?;
    ingestion.remove_indexed(&path_buf);
    info!("Document removed from vector store and cache");
    Ok(())
}

fn build_document_info(path: &Path, stats: &StoredDocument) -> DocumentInfo {
    let kind = DocumentKind::from_path(path)
        .unwrap_or(DocumentKind::Unknown)
        .as_str()
        .to_string();

    DocumentInfo {
        path: path.to_string_lossy().into_owned(),
        kind,
        chunks_count: stats.chunks,
        indexed_at: DateTime::<Utc>::from(stats.indexed_at).to_rfc3339(),
    }
}

fn emit_stage(
    app_handle: &AppHandle,
    stage: &str,
    progress: f32,
    path: &str,
    message: Option<&str>,
) {
    if let Some(message) = message {
        trace!(
            stage,
            progress, path, message, "Emitting rag_progress event"
        );
    } else {
        trace!(stage, progress, path, "Emitting rag_progress event");
    }
    let payload = if let Some(message) = message {
        json!({
            "stage": stage,
            "progress": progress,
            "path": path,
            "message": message
        })
    } else {
        json!({
            "stage": stage,
            "progress": progress,
            "path": path
        })
    };

    if let Err(err) = app_handle.emit("rag_progress", payload) {
        warn!("failed to emit rag_progress: {err}");
    }
}

fn truncate_query(query: &str) -> String {
    const MAX_LEN: usize = 120;
    if query.len() > MAX_LEN {
        format!("{}...", &query[..MAX_LEN])
    } else {
        query.to_string()
    }
}

fn truncate_snippet(text: &str) -> String {
    const MAX_LEN: usize = 160;
    if text.len() > MAX_LEN {
        format!("{}...", &text[..MAX_LEN])
    } else {
        text.to_string()
    }
}
