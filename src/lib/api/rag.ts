import { invoke } from '@tauri-apps/api/core';

const ragLog = (...args: unknown[]) => console.debug('[RAG]', ...args);
const snippet = (value: string, max = 160) =>
  typeof value === 'string' && value.length > max ? `${value.slice(0, max)}...` : value;

export type DocumentInfo = {
  path: string;
  kind: string;
  chunks_count: number;
  indexed_at: string;
};

export type RagHitResponse = {
  text: string;
  score: number;
  document_path: string;
  chunk_index: number;
};

export type RagResponse = {
  hits: RagHitResponse[];
  query_time_ms: number;
};

function normalizeDocumentInfo(payload: any): DocumentInfo {
  return {
    path: String(payload?.path ?? ''),
    kind: String(payload?.kind ?? 'unknown'),
    chunks_count: Number(
      payload?.chunks_count ?? payload?.chunksCount ?? payload?.chunks ?? 0,
    ),
    indexed_at: String(payload?.indexed_at ?? payload?.indexedAt ?? ''),
  };
}

function normalizeHit(payload: any): RagHitResponse {
  return {
    text: String(payload?.text ?? ''),
    score: Number(payload?.score ?? payload?.combined_score ?? 0),
    document_path: String(payload?.document_path ?? payload?.documentPath ?? ''),
    chunk_index: Number(payload?.chunk_index ?? payload?.chunkIndex ?? 0),
  };
}

function normalizeRagResponse(payload: any): RagResponse {
  return {
    hits: Array.isArray(payload?.hits)
      ? payload.hits.map((hit: any) => normalizeHit(hit))
      : [],
    query_time_ms: Number(payload?.query_time_ms ?? payload?.queryTimeMs ?? 0),
  };
}

export async function loadRagDocument(path: string): Promise<DocumentInfo> {
  if (!path) {
    throw new Error('Document path is required');
  }

  ragLog('Invoking load_document', { path });
  try {
    const result = await invoke<any>('load_document', { path });
    const normalized = normalizeDocumentInfo(result);
    ragLog('Document ingestion completed', {
      path: normalized.path,
      kind: normalized.kind,
      chunks: normalized.chunks_count,
    });
    return normalized;
  } catch (error) {
    ragLog('load_document failed', {
      path,
      error: error instanceof Error ? error.message : String(error),
    });
    throw error;
  }
}

export async function listRagDocuments(): Promise<DocumentInfo[]> {
  ragLog('Invoking list_rag_documents');
  try {
    const result = await invoke<any>('list_rag_documents');
    if (!Array.isArray(result)) {
      ragLog('list_rag_documents returned non-array payload');
      return [];
    }
    const documents = result.map((doc) => normalizeDocumentInfo(doc));
    ragLog('Received indexed documents', { count: documents.length });
    return documents;
  } catch (error) {
    ragLog('list_rag_documents failed', {
      error: error instanceof Error ? error.message : String(error),
    });
    throw error;
  }
}

export async function deleteRagDocument(path: string): Promise<void> {
  if (!path) {
    return;
  }
  ragLog('Invoking delete_rag_document', { path });
  try {
    await invoke('delete_rag_document', { path });
    ragLog('Document deleted', { path });
  } catch (error) {
    ragLog('delete_rag_document failed', {
      path,
      error: error instanceof Error ? error.message : String(error),
    });
    throw error;
  }
}

export async function queryRag(query: string, topK = 5): Promise<RagResponse> {
  const top_k = Math.max(1, Math.floor(topK ?? 5));
  ragLog('Invoking query_rag', { topK: top_k, queryPreview: snippet(query) });
  try {
    const result = await invoke<any>('query_rag', { query, top_k });
    const normalized = normalizeRagResponse(result);
    ragLog('Received RAG response', {
      hitCount: normalized.hits.length,
      queryTimeMs: normalized.query_time_ms,
    });
    if (normalized.hits.length > 0) {
      const topHit = normalized.hits[0];
      ragLog('Top RAG hit', {
        documentPath: topHit.document_path,
        score: topHit.score,
        preview: snippet(topHit.text),
      });
    }
    return normalized;
  } catch (error) {
    ragLog('query_rag failed', {
      topK: top_k,
      error: error instanceof Error ? error.message : String(error),
    });
    throw error;
  }
}
