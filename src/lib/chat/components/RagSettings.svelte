<script lang="ts">
  import { listen } from '@tauri-apps/api/event';
  import { deleteRagDocument, listRagDocuments, type DocumentInfo } from '$lib/api/rag';

  let documents = $state<DocumentInfo[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);

  async function loadDocuments() {
    if (loading) return;
    loading = true;
    error = null;
    try {
      documents = await listRagDocuments();
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err ?? 'Unknown error');
      error = message;
    } finally {
      loading = false;
    }
  }

  async function handleDelete(path: string) {
    if (!path) return;
    try {
      await deleteRagDocument(path);
      await loadDocuments();
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err ?? 'Unknown error');
      error = message;
    }
  }

  function formatDate(iso: string): string {
    if (!iso) return '';
    const date = new Date(iso);
    if (Number.isNaN(date.getTime())) return iso;
    return date.toLocaleString();
  }

  $effect(() => {
    void loadDocuments();
  });

  $effect(() => {
    const unlistenPromise = listen<{
      stage?: string;
      path?: string;
      message?: string;
    }>('rag_progress', (event) => {
      const payload = event?.payload ?? {};
      if (payload.stage === 'complete') {
        void loadDocuments();
      }
      if (payload.stage === 'error' && payload.message) {
        error = payload.message;
      }
    });

    return () => {
      unlistenPromise
        .then((fn) => fn())
        .catch(() => {});
    };
  });
</script>

<section class="rag-settings">
  <header class="rag-settings__header">
    <div>
      <h3>Indexed Documents</h3>
      <p class="rag-settings__subtitle">Manage your Retrieval-Augmented documents.</p>
    </div>
    <button
      type="button"
      class="rag-settings__refresh"
      onclick={() => void loadDocuments()}
      disabled={loading}
    >
      {#if loading}
        Refreshing...
      {:else}
        Refresh
      {/if}
    </button>
  </header>

  {#if error}
    <div class="rag-settings__error">{error}</div>
  {/if}

  {#if loading && documents.length === 0}
    <div class="rag-settings__empty">Loading documents...</div>
  {:else if documents.length === 0}
    <div class="rag-settings__empty">No documents indexed yet.</div>
  {:else}
    <ul class="rag-settings__list">
      {#each documents as doc}
        <li class="rag-settings__item">
          <div class="rag-settings__meta">
            <span class="rag-settings__name" title={doc.path}>{doc.path}</span>
            <span class="rag-settings__info">
              <span class="rag-settings__tag">{doc.kind}</span>
              <span class="rag-settings__tag">{doc.chunks_count} chunks</span>
              <span class="rag-settings__date">Indexed {formatDate(doc.indexed_at)}</span>
            </span>
          </div>
          <button
            type="button"
            class="rag-settings__delete"
            onclick={() => void handleDelete(doc.path)}
          >
            Delete
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .rag-settings {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 12px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 12px;
  }

  .rag-settings__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
  }

  .rag-settings__header h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.92);
  }

  .rag-settings__subtitle {
    margin: 0;
    font-size: 12px;
    color: rgba(255, 255, 255, 0.5);
  }

  .rag-settings__refresh {
    appearance: none;
    border: 1px solid rgba(255, 255, 255, 0.2);
    background: rgba(255, 255, 255, 0.08);
    color: rgba(255, 255, 255, 0.85);
    padding: 6px 12px;
    border-radius: 999px;
    font-size: 12px;
    cursor: pointer;
    transition: background 0.2s ease;
  }

  .rag-settings__refresh:disabled {
    opacity: 0.6;
    cursor: default;
  }

  .rag-settings__refresh:not(:disabled):hover {
    background: rgba(255, 255, 255, 0.16);
  }

  .rag-settings__error {
    padding: 8px 12px;
    border-radius: 8px;
    background: rgba(196, 85, 85, 0.15);
    border: 1px solid rgba(196, 85, 85, 0.4);
    color: rgba(255, 205, 178, 0.95);
    font-size: 12px;
  }

  .rag-settings__empty {
    padding: 12px;
    font-size: 12px;
    color: rgba(255, 255, 255, 0.6);
    background: rgba(255, 255, 255, 0.04);
    border-radius: 8px;
  }

  .rag-settings__list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .rag-settings__item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.08);
  }

  .rag-settings__meta {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }

  .rag-settings__name {
    font-size: 13px;
    color: rgba(255, 255, 255, 0.88);
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 220px;
  }

  .rag-settings__info {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    font-size: 11px;
    color: rgba(255, 255, 255, 0.6);
  }

  .rag-settings__tag {
    padding: 2px 6px;
    border-radius: 999px;
    background: rgba(102, 126, 234, 0.18);
    color: rgba(226, 198, 255, 0.9);
  }

  .rag-settings__date {
    color: rgba(255, 255, 255, 0.45);
  }

  .rag-settings__delete {
    appearance: none;
    border: none;
    background: rgba(196, 85, 85, 0.18);
    color: rgba(255, 205, 178, 0.95);
    border-radius: 999px;
    padding: 6px 12px;
    font-size: 12px;
    cursor: pointer;
    transition: background 0.2s ease;
  }

  .rag-settings__delete:hover {
    background: rgba(196, 85, 85, 0.28);
  }

  @media (max-width: 720px) {
    .rag-settings__header {
      flex-direction: column;
      align-items: flex-start;
    }

    .rag-settings__name {
      max-width: 100%;
      white-space: normal;
    }
  }
</style>
