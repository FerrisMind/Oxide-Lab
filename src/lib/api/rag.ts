import { invoke } from '@tauri-apps/api/core';

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

  const result = await invoke<any>('load_document', { path });
  return normalizeDocumentInfo(result);
}

export async function listRagDocuments(): Promise<DocumentInfo[]> {
  const result = await invoke<any>('list_rag_documents');
  if (!Array.isArray(result)) {
    return [];
  }
  return result.map((doc) => normalizeDocumentInfo(doc));
}

export async function deleteRagDocument(path: string): Promise<void> {
  if (!path) {
    return;
  }
  await invoke('delete_rag_document', { path });
}

export async function queryRag(query: string, topK = 5): Promise<RagResponse> {
  const top_k = Math.max(1, Math.floor(topK ?? 5));
  const result = await invoke<any>('query_rag', { query, top_k });
  return normalizeRagResponse(result);
}
