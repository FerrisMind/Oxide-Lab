<script lang="ts">
  import { chatHistory, sortedSessions, currentSession } from '$lib/stores/chat-history';
  import { onMount } from 'svelte';
  import type { ChatSession } from '$lib/stores/chat-history';
  import PencilSimpleLine from 'phosphor-svelte/lib/PencilSimpleLine';
  import Export from 'phosphor-svelte/lib/Export';
  import TrashSimple from 'phosphor-svelte/lib/TrashSimple';
  import Plus from 'phosphor-svelte/lib/Plus';
  import UploadSimple from 'phosphor-svelte/lib/UploadSimple';

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

<section class="chat-history">
  <div class="chat-history-header">
    <h3>История чатов</h3>
    <div class="header-actions">
      <button class="btn-icon" on:click={handleNewChat} title="Новый чат" aria-label="Создать новый чат">
        <Plus size={20} weight="regular" />
      </button>
      <button class="btn-icon" on:click={handleImport} title="Импорт" aria-label="Импортировать чат из файла">
        <UploadSimple size={20} weight="regular" />
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
        <button class="primary" on:click={handleNewChat}>Начать новый чат</button>
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
                <PencilSimpleLine size={16} weight="bold" />
              </button>
              <button
                class="btn-icon-small"
                on:click={() => handleExport(session.id)}
                title="Экспорт"
                aria-label={`Экспортировать чат "${session.title}"`}
              >
                <Export size={16} weight="bold" />
              </button>
              <button
                class="btn-icon-small danger"
                on:click={() => confirmDelete(session.id)}
                title="Удалить"
                aria-label={`Удалить чат "${session.title}"`}
              >
                <TrashSimple size={16} weight="bold" />
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
                <button class="secondary" on:click={() => (showDeleteConfirm = null)}>
                  Отмена
                </button>
                <button class="primary danger" on:click={() => handleDelete(session.id)}>
                  Удалить
                </button>
              </div>
            </div>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</section>

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
        <button class="secondary" on:click={copyExportData}>Копировать</button>
        <button class="primary" on:click={downloadExportData}>Скачать</button>
      </div>
    </div>
  </div>
{/if}

<style>
  /* ===== Chat History Panel - LoaderPanel Style ===== */
  
  .chat-history {
    --control-radius: 10px;
    --control-padding-y: 8px;
    --control-padding-x: 12px;
    --focus-ring: 0 0 0 3px rgb(179 205 224 / 0.15);

    width: 100%;
    max-width: 100%;
    overflow-x: hidden;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--card);
    border-radius: 14px;
    padding: 16px;
  }

  .chat-history-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border-color);
  }

  .chat-history-header h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text);
  }

  .header-actions {
    display: flex;
    gap: 8px;
  }

  .btn-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    border-radius: var(--control-radius);
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-icon:hover {
    background: var(--accent);
    color: white;
  }

  .sessions-list {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px 20px;
    text-align: center;
    color: var(--muted);
  }

  .empty-state p {
    margin: 0 0 16px 0;
    font-size: 14px;
  }

  /* Primary button style matching LoaderPanel */
  .primary {
    background: var(--accent-2);
    border: none;
    border-radius: 12px;
    padding: 10px 14px;
    cursor: pointer;
    color: #3a2f4f;
    font-weight: 600;
    font-size: 14px;
    transition: all 0.2s ease;
  }

  .primary:hover {
    background: color-mix(in srgb, var(--accent-2) 85%, black 10%);
  }

  .primary.danger {
    background: #ffb3b3;
    color: #3a1f1f;
  }

  .primary.danger:hover {
    background: color-mix(in srgb, #ffb3b3 85%, black 10%);
  }

  /* Secondary button style */
  .secondary {
    background: transparent;
    border: 1px solid var(--border-color);
    border-radius: var(--control-radius);
    padding: var(--control-padding-y) var(--control-padding-x);
    color: var(--text);
    font-size: 14px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .secondary:hover {
    background: var(--accent-light);
    border-color: var(--accent);
  }

  /* Session items styled like LoaderPanel fields */
  .session-item {
    display: grid;
    gap: 8px;
    margin: 6px 0;
    padding: 12px;
    border: 1px solid var(--border-color);
    border-radius: var(--control-radius);
    background: var(--card);
    cursor: pointer;
    transition: all 0.2s ease;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
    position: relative;
    overflow: hidden;
  }

  .session-item::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 3px;
    background: transparent;
    transition: background 0.2s ease;
  }

  .session-item:hover {
    border-color: var(--accent);
    background: color-mix(in srgb, var(--card) 95%, var(--accent) 5%);
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .session-item:hover::before {
    background: var(--accent);
  }

  .session-item.active {
    border-color: var(--accent);
    background: color-mix(in srgb, var(--card) 90%, var(--accent) 10%);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .session-item.active::before {
    background: var(--accent);
  }

  .session-content {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .session-info {
    flex: 1;
    min-width: 0;
  }

  .session-title {
    font-size: 14px;
    font-weight: 500;
    color: var(--text);
    margin-bottom: 4px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .session-meta {
    display: flex;
    gap: 8px;
    font-size: 12px;
    color: var(--muted);
  }

  .session-date {
    white-space: nowrap;
  }

  .session-count {
    white-space: nowrap;
  }

  .session-actions {
    display: flex;
    gap: 6px;
    opacity: 0;
    visibility: hidden;
    transition: opacity 0.15s ease, visibility 0.15s ease, transform 0.15s ease;
    transform: translateY(-2px);
  }

  .session-item:hover .session-actions {
    opacity: 1;
    visibility: visible;
    transform: translateY(0);
  }

  .session-item.active .session-actions {
    opacity: 1;
    visibility: visible;
    transform: translateY(0);
  }

  .btn-icon-small {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: 1px solid var(--border-color);
    border-radius: var(--control-radius);
    background: var(--card);
    color: var(--text);
    cursor: pointer;
    transition: all 0.2s ease;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    position: relative;
    padding: 0;
  }

  .btn-icon-small :global(svg) {
    width: 16px;
    height: 16px;
    color: currentColor;
  }

  .btn-icon-small:hover {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
    transform: translateY(-1px);
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
  }

  .btn-icon-small.danger {
    color: var(--danger);
    border-color: var(--danger);
  }

  .btn-icon-small.danger:hover {
    background: var(--danger);
    color: white;
    border-color: var(--danger);
  }

  /* Edit input matching LoaderPanel field inputs */
  .edit-input {
    border: 1px solid var(--border-color);
    background: #fcfbfa;
    color: var(--text);
    border-radius: var(--control-radius);
    padding: var(--control-padding-y) var(--control-padding-x);
    outline: none;
    width: 100%;
    box-sizing: border-box;
    font-size: 14px;
  }

  .edit-input:focus {
    border-color: var(--accent);
    box-shadow: var(--focus-ring);
  }

  .edit-input::placeholder {
    color: #5a5a5a;
    opacity: 0.9;
  }

  /* Delete confirmation styled like LoaderPanel */
  .delete-confirm {
    margin-top: 8px;
    padding: var(--control-padding-y) var(--control-padding-x);
    border-radius: var(--control-radius);
    background: color-mix(in srgb, #ffb3b3 20%, transparent 80%);
    border: 1px solid #ffb3b3;
  }

  .delete-confirm p {
    margin: 0 0 12px 0;
    font-size: 14px;
    color: #3a1f1f;
    font-weight: 500;
  }

  .confirm-actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }

  /* Modal styling matching LoaderPanel aesthetic */
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
    background: var(--card);
    border-radius: 14px;
    border: 1px solid var(--border-color);
    max-width: 500px;
    width: 90%;
    max-height: 80vh;
    overflow: hidden;
    box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px;
    border-bottom: 1px solid var(--border-color);
  }

  .modal-header h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text);
  }

  .btn-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    font-size: 18px;
    transition: all 0.2s ease;
  }

  .btn-close:hover {
    background: var(--accent);
    color: white;
  }

  .modal-body {
    padding: 16px;
  }

  .modal-body textarea {
    width: 100%;
    padding: var(--control-padding-y) var(--control-padding-x);
    border: 1px solid var(--border-color);
    border-radius: var(--control-radius);
    background: #fcfbfa;
    color: var(--text);
    font-family: monospace;
    font-size: 12px;
    resize: vertical;
    outline: none;
    box-sizing: border-box;
  }

  .modal-body textarea:focus {
    border-color: var(--accent);
    box-shadow: var(--focus-ring);
  }

  .modal-footer {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    padding: 16px;
    border-top: 1px solid var(--border-color);
  }

  /* Dark theme adjustments */
  @media (prefers-color-scheme: dark) {
    .session-item {
      background: #2d2d2d;
      border-color: #3a3a3a;
      box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
    }

    .session-item:hover {
      background: color-mix(in srgb, #2d2d2d 95%, var(--accent) 5%);
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
    }

    .session-item.active {
      background: color-mix(in srgb, #2d2d2d 90%, var(--accent) 10%);
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
    }

    .btn-icon-small {
      background: #2d2d2d;
      border-color: #3a3a3a;
      color: var(--text);
      box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
    }

    .btn-icon-small:hover {
      box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
    }

    .btn-icon-small :global(svg) {
      width: 16px;
      height: 16px;
      color: currentColor;
    }
    
    .edit-input {
      background: #2d2d2d;
      border-color: #3a3a3a;
    }
    
    .edit-input::placeholder {
      color: #bdbdbd;
      opacity: 1;
    }
    
    .modal-body textarea {
      background: #2d2d2d;
      border-color: #3a3a3a;
    }
  }
</style>
