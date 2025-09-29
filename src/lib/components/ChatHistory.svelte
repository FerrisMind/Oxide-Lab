<script lang="ts">
  import { chatHistory, sortedSessions, currentSession } from '$lib/stores/chat-history';
  import { onMount } from 'svelte';
  import type { ChatSession } from '$lib/stores/chat-history';

  let editingSessionId: string | null = null;
  let editingTitle = '';
  let showDeleteConfirm: string | null = null;
  let showExportModal = false;
  let exportData = '';
  let importInput: HTMLInputElement;

  function handleNewChat() {
    const sessionId = chatHistory.createSession();
    console.log('Создан новый чат:', sessionId);
    // Сессия автоматически станет активной и загрузится в Chat.svelte
  }

  function handleLoadSession(sessionId: string) {
    console.log('Переключаемся на сессию:', sessionId);
    chatHistory.loadSession(sessionId);
  }

  function startEditing(session: ChatSession) {
    editingSessionId = session.id;
    editingTitle = session.title;
  }

  function saveEdit() {
    if (editingSessionId && editingTitle.trim()) {
      chatHistory.renameSession(editingSessionId, editingTitle.trim());
    }
    editingSessionId = null;
    editingTitle = '';
  }

  function cancelEdit() {
    editingSessionId = null;
    editingTitle = '';
  }

  function confirmDelete(sessionId: string) {
    showDeleteConfirm = sessionId;
  }

  function handleDelete(sessionId: string) {
    chatHistory.deleteSession(sessionId);
    showDeleteConfirm = null;
  }

  function handleExport(sessionId: string) {
    const data = chatHistory.exportSession(sessionId);
    if (data) {
      exportData = data;
      showExportModal = true;
    }
  }

  function copyExportData() {
    navigator.clipboard.writeText(exportData);
  }

  function downloadExportData() {
    const blob = new Blob([exportData], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `chat-export-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }

  function handleImport() {
    importInput.click();
  }

  function onImportFile(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (e) => {
      const content = e.target?.result as string;
      if (chatHistory.importSession(content)) {
        alert('Чат успешно импортирован!');
      } else {
        alert('Ошибка импорта чата. Проверьте формат файла.');
      }
    };
    reader.readAsText(file);
    target.value = ''; // Очистка input
  }

  function formatDate(timestamp: number): string {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) {
      return 'Сегодня';
    } else if (diffDays === 1) {
      return 'Вчера';
    } else if (diffDays < 7) {
      return `${diffDays} дней назад`;
    } else {
      return date.toLocaleDateString('ru-RU');
    }
  }

  onMount(() => {
    // Если нет текущей сессии, создаем новую
    if (!$currentSession) {
      handleNewChat();
    }
  });
</script>

<div class="chat-history">
  <div class="chat-history-header">
    <h3>История чатов</h3>
    <div class="header-actions">
      <button class="btn-icon" on:click={handleNewChat} title="Новый чат" aria-label="Создать новый чат">
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          aria-hidden="true"
        >
          <path d="M12 5v14M5 12h14" />
        </svg>
      </button>
      <button class="btn-icon" on:click={handleImport} title="Импорт" aria-label="Импортировать чат из файла">
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          aria-hidden="true"
        >
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12" />
        </svg>
      </button>
    </div>
  </div>

  <input
    type="file"
    accept=".json"
    bind:this={importInput}
    on:change={onImportFile}
    style="display: none;"
  />

  <div class="sessions-list">
    {#if $sortedSessions.length === 0}
      <div class="empty-state">
        <p>Нет сохраненных чатов</p>
        <button class="btn-primary" on:click={handleNewChat}>Начать новый чат</button>
      </div>
    {:else}
      {#each $sortedSessions as session (session.id)}
        <div
          class="session-item"
          class:active={$currentSession?.id === session.id}
          on:click={() => handleLoadSession(session.id)}
          on:keydown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') {
              e.preventDefault();
              handleLoadSession(session.id);
            }
          }}
          role="button"
          tabindex="0"
          aria-label={`Переключиться на чат: ${session.title}`}
        >
          <div class="session-content">
            {#if editingSessionId === session.id}
              <input
                type="text"
                class="edit-input"
                bind:value={editingTitle}
                on:blur={saveEdit}
                on:keydown={(e) => {
                  if (e.key === 'Enter') saveEdit();
                  if (e.key === 'Escape') cancelEdit();
                }}
                on:click|stopPropagation
                aria-label="Новое название чата"
              />
            {:else}
              <div class="session-info">
                <div class="session-title">{session.title}</div>
                <div class="session-meta">
                  <span class="session-date">{formatDate(session.updatedAt)}</span>
                  <span class="session-count">{session.messages.length} сообщений</span>
                </div>
              </div>
            {/if}

            <div 
              class="session-actions" 
              on:click|stopPropagation
              on:keydown={(e) => e.stopPropagation()}
              role="toolbar" 
              aria-label="Действия с чатом"
              tabindex="-1"
            >
              <button
                class="btn-icon-small"
                on:click={() => startEditing(session)}
                title="Переименовать"
                aria-label={`Переименовать чат "${session.title}"`}
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true">
                  <path
                    d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"
                  />
                </svg>
              </button>
              <button
                class="btn-icon-small"
                on:click={() => handleExport(session.id)}
                title="Экспорт"
                aria-label={`Экспортировать чат "${session.title}"`}
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true">
                  <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3" />
                </svg>
              </button>
              <button
                class="btn-icon-small danger"
                on:click={() => confirmDelete(session.id)}
                title="Удалить"
                aria-label={`Удалить чат "${session.title}"`}
              >
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true">
                  <path
                    d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"
                  />
                </svg>
              </button>
            </div>
          </div>

          {#if showDeleteConfirm === session.id}
            <div 
              class="delete-confirm" 
              on:click|stopPropagation
              on:keydown={(e) => e.stopPropagation()}
              role="alertdialog"
              aria-labelledby="delete-confirm-title"
              aria-describedby="delete-confirm-desc"
              tabindex="-1"
            >
              <p id="delete-confirm-desc">Удалить этот чат?</p>
              <div class="confirm-actions">
                <button class="btn-secondary" on:click={() => (showDeleteConfirm = null)}>
                  Отмена
                </button>
                <button class="btn-danger" on:click={() => handleDelete(session.id)}>
                  Удалить
                </button>
              </div>
            </div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>

{#if showExportModal}
  <div 
    class="modal-overlay" 
    on:click={() => (showExportModal = false)}
    on:keydown={(e) => {
      if (e.key === 'Escape') {
        showExportModal = false;
      }
    }}
    role="presentation"
  >
    <div 
      class="modal" 
      on:click|stopPropagation
      on:keydown={(e) => {
        e.stopPropagation();
        if (e.key === 'Escape') {
          showExportModal = false;
        }
      }}
      role="dialog"
      aria-modal="true"
      aria-labelledby="export-modal-title"
      tabindex="-1"
    >
      <div class="modal-header">
        <h3 id="export-modal-title">Экспорт чата</h3>
        <button 
          class="btn-close" 
          on:click={() => (showExportModal = false)}
          aria-label="Закрыть окно экспорта"
        >
          ×
        </button>
      </div>
      <div class="modal-body">
        <textarea 
          readonly 
          value={exportData} 
          rows="10"
          aria-label="Данные экспортированного чата"
        ></textarea>
      </div>
      <div class="modal-footer">
        <button class="btn-secondary" on:click={copyExportData}>Копировать</button>
        <button class="btn-primary" on:click={downloadExportData}>Скачать</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .chat-history {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-bg-primary, #ffffff);
    border-right: 1px solid var(--color-border, #e0e0e0);
  }

  .chat-history-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem;
    border-bottom: 1px solid var(--color-border, #e0e0e0);
  }

  .chat-history-header h3 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
    color: var(--color-text-primary, #1a1a1a);
  }

  .header-actions {
    display: flex;
    gap: 0.5rem;
  }

  .btn-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    padding: 0;
    border: none;
    background: transparent;
    border-radius: 6px;
    cursor: pointer;
    color: var(--color-text-secondary, #666);
    transition: all 0.2s;
  }

  .btn-icon:hover {
    background: var(--color-bg-hover, #f5f5f5);
    color: var(--color-text-primary, #1a1a1a);
  }

  .sessions-list {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 2rem 1rem;
    text-align: center;
  }

  .empty-state p {
    margin: 0 0 1rem 0;
    color: var(--color-text-secondary, #666);
  }

  .session-item {
    margin-bottom: 0.5rem;
    padding: 0.75rem;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
    background: var(--color-bg-secondary, #f9f9f9);
    border: 1px solid transparent;
  }

  .session-item:hover {
    background: var(--color-bg-hover, #f0f0f0);
  }

  .session-item.active {
    background: var(--color-accent-bg, #e3f2fd);
    border-color: var(--color-accent, #2196f3);
  }

  .session-content {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 0.5rem;
  }

  .session-info {
    flex: 1;
    min-width: 0;
  }

  .session-title {
    font-weight: 500;
    font-size: 0.95rem;
    color: var(--color-text-primary, #1a1a1a);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-bottom: 0.25rem;
  }

  .session-meta {
    display: flex;
    gap: 0.5rem;
    font-size: 0.75rem;
    color: var(--color-text-secondary, #666);
  }

  .session-actions {
    display: flex;
    gap: 0.25rem;
    opacity: 0;
    transition: opacity 0.2s;
  }

  .session-item:hover .session-actions {
    opacity: 1;
  }

  .btn-icon-small {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    padding: 0;
    border: none;
    background: transparent;
    border-radius: 4px;
    cursor: pointer;
    color: var(--color-text-secondary, #666);
    transition: all 0.2s;
  }

  .btn-icon-small:hover {
    background: var(--color-bg-hover, #e0e0e0);
    color: var(--color-text-primary, #1a1a1a);
  }

  .btn-icon-small.danger:hover {
    background: #fee;
    color: #c00;
  }

  .edit-input {
    width: 100%;
    padding: 0.25rem 0.5rem;
    border: 1px solid var(--color-accent, #2196f3);
    border-radius: 4px;
    font-size: 0.95rem;
    outline: none;
  }

  .delete-confirm {
    margin-top: 0.5rem;
    padding: 0.75rem;
    background: var(--color-bg-primary, #fff);
    border: 1px solid var(--color-border, #e0e0e0);
    border-radius: 6px;
  }

  .delete-confirm p {
    margin: 0 0 0.75rem 0;
    font-size: 0.9rem;
  }

  .confirm-actions {
    display: flex;
    gap: 0.5rem;
  }

  .btn-primary,
  .btn-secondary,
  .btn-danger {
    flex: 1;
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 6px;
    font-size: 0.9rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-primary {
    background: var(--color-accent, #2196f3);
    color: white;
  }

  .btn-primary:hover {
    background: var(--color-accent-hover, #1976d2);
  }

  .btn-secondary {
    background: var(--color-bg-secondary, #f5f5f5);
    color: var(--color-text-primary, #1a1a1a);
  }

  .btn-secondary:hover {
    background: var(--color-bg-hover, #e0e0e0);
  }

  .btn-danger {
    background: #f44336;
    color: white;
  }

  .btn-danger:hover {
    background: #d32f2f;
  }

  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal {
    background: var(--color-bg-primary, #fff);
    border-radius: 12px;
    width: 90%;
    max-width: 600px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.3);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1.5rem;
    border-bottom: 1px solid var(--color-border, #e0e0e0);
  }

  .modal-header h3 {
    margin: 0;
    font-size: 1.25rem;
  }

  .btn-close {
    background: none;
    border: none;
    font-size: 2rem;
    line-height: 1;
    cursor: pointer;
    color: var(--color-text-secondary, #666);
    padding: 0;
    width: 32px;
    height: 32px;
  }

  .modal-body {
    flex: 1;
    padding: 1.5rem;
    overflow-y: auto;
  }

  .modal-body textarea {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid var(--color-border, #e0e0e0);
    border-radius: 6px;
    font-family: monospace;
    font-size: 0.85rem;
    resize: vertical;
  }

  .modal-footer {
    display: flex;
    gap: 0.75rem;
    padding: 1.5rem;
    border-top: 1px solid var(--color-border, #e0e0e0);
  }
</style>
