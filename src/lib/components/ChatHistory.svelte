<script lang="ts">
  import { chatHistory, sortedSessions, currentSession } from '$lib/stores/chat-history';
  import { onMount } from 'svelte';
  import type { ChatSession } from '$lib/stores/chat-history';
  import PencilSimpleLine from 'phosphor-svelte/lib/PencilSimpleLine';
  import Export from 'phosphor-svelte/lib/Export';
  import TrashSimple from 'phosphor-svelte/lib/TrashSimple';
  import Plus from 'phosphor-svelte/lib/Plus';
  import DownloadSimple from 'phosphor-svelte/lib/DownloadSimple';
  import StackMinus from 'phosphor-svelte/lib/StackMinus';
  import { writeTextFile } from '@tauri-apps/plugin-fs';
  import { save } from '@tauri-apps/plugin-dialog';
  import hljs from 'highlight.js/lib/core';
  import json from 'highlight.js/lib/languages/json';
  import 'highlight.js/styles/github-dark.css';

  // Инициализация highlight.js
  hljs.registerLanguage('json', json);

  let editingSessionId: string | null = null;
  let editingTitle = '';
  let showDeleteConfirm: string | null = null;
  let showClearAllConfirm = false;
  let showExportModal = false;
  let exportData = '';
  let importInput: HTMLInputElement;
  let exportCodeElement: HTMLElement;
  let isDownloading = false;
  let isCopying = false;

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

  function handleClearAll() {
    showClearAllConfirm = true;
  }

  function confirmClearAll() {
    chatHistory.clearAll();
    showClearAllConfirm = false;
  }

  function cancelClearAll() {
    showClearAllConfirm = false;
  }

  function formatJson(jsonString: string): string {
    try {
      const parsed = JSON.parse(jsonString);
      return JSON.stringify(parsed, null, 2);
    } catch {
      return jsonString;
    }
  }

  function handleExport(sessionId: string) {
    try {
      const data = chatHistory.exportSession(sessionId);
      if (data) {
        exportData = formatJson(data);
        showExportModal = true;
        console.log('Данные экспорта подготовлены:', exportData.length, 'символов');

        // Подсвечиваем код после открытия модального окна
        setTimeout(() => {
          if (exportCodeElement) {
            hljs.highlightElement(exportCodeElement);
          }
        }, 100);
      } else {
        console.error('Не удалось получить данные для экспорта сессии:', sessionId);
      }
    } catch (error) {
      console.error('Ошибка при экспорте сессии:', error);
    }
  }

  function copyExportData() {
    if (isCopying) return;

    try {
      if (!exportData || exportData.trim() === '') {
        console.error('Нет данных для копирования');
        return;
      }

      isCopying = true;
      navigator.clipboard
        .writeText(exportData)
        .then(() => {
          console.log('Данные успешно скопированы в буфер обмена');
          // Сбрасываем состояние через короткое время
          setTimeout(() => {
            isCopying = false;
          }, 1000);
        })
        .catch((error) => {
          console.error('Ошибка при копировании:', error);
          isCopying = false;
        });
    } catch (error) {
      console.error('Ошибка при копировании данных:', error);
      isCopying = false;
    }
  }

  async function downloadExportData() {
    console.log('downloadExportData вызвана, isDownloading:', isDownloading);

    if (isDownloading) return;

    try {
      if (!exportData || exportData.trim() === '') {
        console.error('Нет данных для скачивания');
        return;
      }

      console.log(
        'Начинаем скачивание через диалоговое окно, размер данных:',
        exportData.length,
        'символов',
      );

      // Генерируем имя файла с датой и временем
      const fileName = `chat-export-${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.json`;

      console.log('Открываем диалоговое окно сохранения для файла:', fileName);

      // Показываем диалоговое окно для выбора места сохранения
      const filePath = await save({
        defaultPath: fileName,
        filters: [
          {
            name: 'JSON Files',
            extensions: ['json'],
          },
          {
            name: 'All Files',
            extensions: ['*'],
          },
        ],
      });

      // Если пользователь отменил выбор, выходим
      if (!filePath) {
        console.log('Пользователь отменил сохранение файла');
        return;
      }

      console.log('Пользователь выбрал путь для сохранения:', filePath);

      // Устанавливаем флаг загрузки
      isDownloading = true;

      // Сохраняем файл по выбранному пути
      await writeTextFile(filePath, exportData);

      console.log('Файл успешно сохранен:', filePath);
      isDownloading = false;

      // Показываем уведомление пользователю
      alert(`Файл успешно сохранен:\n${filePath}`);
    } catch (error) {
      console.error('Ошибка при сохранении файла:', error);
      isDownloading = false;

      // Показываем ошибку пользователю
      const errorMessage = error instanceof Error ? error.message : String(error);
      alert(`Ошибка при сохранении файла: ${errorMessage}`);
    }
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
      <button
        class="btn-icon"
        on:click={handleNewChat}
        title="Новый чат"
        aria-label="Создать новый чат"
      >
        <svelte:component this={Plus} size={16} weight="bold" />
      </button>
      <button
        class="btn-icon"
        on:click={handleImport}
        title="Импорт"
        aria-label="Импортировать чат из файла"
      >
        <svelte:component this={DownloadSimple} size={16} weight="bold" />
      </button>
      <button
        class="btn-icon danger"
        on:click={handleClearAll}
        title="Удалить все чаты"
        aria-label="Удалить все чаты"
      >
        <svelte:component this={StackMinus} size={16} weight="bold" />
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
        <button class="btn-new-chat" on:click={handleNewChat}>Начать новый чат</button>
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
        <div class="code-container">
          <pre><code
              bind:this={exportCodeElement}
              class="language-json"
              aria-label="Данные экспортированного чата">{exportData}</code
            ></pre>
        </div>
      </div>
      <div class="modal-footer">
        <button class="secondary" on:click={copyExportData} disabled={isCopying}>
          {isCopying ? 'Копирование...' : 'Копировать'}
        </button>
        <button
          class="primary"
          on:click={(e) => {
            console.log('Кнопка скачать нажата', e);
            downloadExportData();
          }}
          disabled={isDownloading}
          type="button"
          aria-label="Скачать экспортированные данные чата в JSON файл"
        >
          {isDownloading ? 'Скачивание...' : 'Скачать'}
        </button>
      </div>
    </div>
  </div>
{/if}

{#if showClearAllConfirm}
  <div
    class="modal-overlay"
    on:click={cancelClearAll}
    on:keydown={(e) => {
      if (e.key === 'Escape') {
        cancelClearAll();
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
          cancelClearAll();
        }
      }}
      role="dialog"
      aria-modal="true"
      aria-labelledby="clear-all-modal-title"
      tabindex="-1"
    >
      <div class="modal-header">
        <h3 id="clear-all-modal-title">Удалить все чаты</h3>
        <button class="btn-close" on:click={cancelClearAll} aria-label="Закрыть окно подтверждения">
          ×
        </button>
      </div>
      <div class="modal-body">
        <p>Вы уверены, что хотите удалить все чаты? Это действие нельзя отменить.</p>
      </div>
      <div class="modal-footer">
        <button class="secondary" on:click={cancelClearAll}>Отмена</button>
        <button class="primary danger" on:click={confirmClearAll}>Удалить все</button>
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
    --composer-row-height: 34px;
    --composer-control-radius: 12px;

    width: 100%;
    max-width: 100%;
    overflow-x: hidden;
    overflow-y: visible;
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
    overflow: visible;
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
    align-items: center;
  }

  .btn-icon {
    border: 2px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.08);
    color: #ffffff;
    border-radius: var(--composer-control-radius);
    padding: 0;
    width: var(--composer-row-height);
    height: var(--composer-row-height);
    font-size: 14px;
    font-weight: 500;
    cursor: default;
    transition:
      transform 0.2s ease,
      box-shadow 0.2s ease,
      background 0.2s ease,
      border-color 0.2s ease,
      color 0.2s ease,
      scale 0.2s ease;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    box-sizing: border-box;
    min-width: 0;
    position: relative;
    overflow: hidden;
  }

  .btn-icon::before {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.1), transparent);
    transition: left 0.5s ease;
  }

  .btn-icon:not(:disabled):hover {
    transform: none;
    background: rgba(255, 255, 255, 0.15);
    border-color: rgba(255, 255, 255, 0.2);
    box-shadow:
      0 8px 25px rgba(0, 0, 0, 0.15),
      0 4px 12px rgba(0, 0, 0, 0.1);
    scale: 1.1;
  }

  .btn-icon:not(:disabled):hover::before {
    left: 100%;
  }

  .btn-icon:not(:disabled):active {
    transform: none;
    scale: 1.05;
    box-shadow:
      0 4px 15px rgba(0, 0, 0, 0.2),
      0 2px 8px rgba(0, 0, 0, 0.15);
  }

  .btn-icon.danger {
    color: var(--danger);
    border-color: rgba(231, 76, 60, 0.3);
    background: rgba(231, 76, 60, 0.1);
  }

  .btn-icon.danger:not(:disabled):hover {
    background: rgba(231, 76, 60, 0.2);
    border-color: rgba(231, 76, 60, 0.5);
    box-shadow:
      0 8px 25px rgba(231, 76, 60, 0.3),
      0 4px 12px rgba(0, 0, 0, 0.1);
    scale: 1.1;
  }

  .btn-icon.danger:not(:disabled):active {
    background: rgba(231, 76, 60, 0.3);
    border-color: rgba(231, 76, 60, 0.6);
    scale: 1.05;
  }

  .btn-icon :global(svg) {
    width: 16px;
    height: 16px;
    color: currentColor;
    display: block;
    flex-shrink: 0;
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
  .btn-new-chat {
    border: 2px solid rgba(255, 255, 255, 0.1);
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: #ffffff;
    border-radius: var(--composer-control-radius);
    padding: 10px 16px;
    height: var(--composer-row-height);
    font-size: 14px;
    font-weight: 600;
    cursor: default;
    transition:
      transform 0.2s ease,
      box-shadow 0.2s ease,
      background 0.2s ease,
      border-color 0.2s ease,
      color 0.2s ease,
      scale 0.2s ease;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    box-sizing: border-box;
    min-width: 0;
    position: relative;
    overflow: hidden;
    box-shadow:
      0 4px 15px rgba(102, 126, 234, 0.3),
      0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .btn-new-chat::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(
      135deg,
      rgba(255, 255, 255, 0.2) 0%,
      transparent 50%,
      rgba(255, 255, 255, 0.1) 100%
    );
    opacity: 0;
    transition: opacity 0.3s ease;
  }

  .btn-new-chat:not(:disabled):hover {
    background: linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%);
    box-shadow:
      0 8px 25px rgba(102, 126, 234, 0.4),
      0 4px 12px rgba(0, 0, 0, 0.15);
    transform: none;
    scale: 1.05;
  }

  .btn-new-chat:not(:disabled):hover::before {
    opacity: 1;
  }

  .btn-new-chat:not(:disabled):active {
    transform: none;
    scale: 1.02;
    box-shadow:
      0 4px 15px rgba(102, 126, 234, 0.5),
      0 2px 8px rgba(0, 0, 0, 0.2);
  }

  .btn-new-chat:disabled {
    background: linear-gradient(135deg, #4a5568 0%, #2d3748 100%);
    border-color: transparent;
    color: rgba(255, 255, 255, 0.4);
    box-shadow: none;
    transform: none;
    cursor: not-allowed;
  }

  /* Secondary button style */
  .secondary {
    border: 2px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.08);
    color: #ffffff;
    border-radius: 8px;
    padding: 10px 16px;
    height: var(--composer-row-height);
    font-size: 14px;
    font-weight: 500;
    cursor: default;
    transition:
      transform 0.2s ease,
      box-shadow 0.2s ease,
      background 0.2s ease,
      border-color 0.2s ease,
      color 0.2s ease,
      scale 0.2s ease;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    box-sizing: border-box;
    min-width: 0;
    position: relative;
    overflow: hidden;
  }

  .secondary::before {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.1), transparent);
    transition: left 0.5s ease;
  }

  .secondary:not(:disabled):hover {
    transform: none;
    background: rgba(255, 255, 255, 0.15);
    border-color: rgba(255, 255, 255, 0.2);
    box-shadow:
      0 8px 25px rgba(0, 0, 0, 0.15),
      0 4px 12px rgba(0, 0, 0, 0.1);
    scale: 1.05;
  }

  .secondary:not(:disabled):hover::before {
    left: 100%;
  }

  .secondary:not(:disabled):active {
    transform: none;
    scale: 1.02;
    box-shadow:
      0 4px 15px rgba(0, 0, 0, 0.2),
      0 2px 8px rgba(0, 0, 0, 0.15);
  }

  .secondary:disabled {
    opacity: 0.6;
    color: rgba(255, 255, 255, 0.5);
    cursor: not-allowed;
    transform: none;
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 255, 255, 0.05);
  }

  /* Primary button style for modal */
  .primary {
    border: 2px solid transparent;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: #ffffff;
    border-radius: 8px;
    padding: 10px 16px;
    height: var(--composer-row-height);
    font-size: 14px;
    font-weight: 600;
    cursor: default;
    pointer-events: auto;
    transition:
      transform 0.2s ease,
      box-shadow 0.2s ease,
      background 0.2s ease,
      border-color 0.2s ease,
      color 0.2s ease,
      scale 0.2s ease;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    box-sizing: border-box;
    min-width: 0;
    position: relative;
    overflow: hidden;
    box-shadow:
      0 4px 15px rgba(102, 126, 234, 0.3),
      0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .primary::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(
      135deg,
      rgba(255, 255, 255, 0.2) 0%,
      transparent 50%,
      rgba(255, 255, 255, 0.1) 100%
    );
    opacity: 0;
    transition: opacity 0.3s ease;
  }

  .primary:not(:disabled):hover {
    background: linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%);
    box-shadow:
      0 8px 25px rgba(102, 126, 234, 0.4),
      0 4px 12px rgba(0, 0, 0, 0.15);
    transform: none;
    scale: 1.05;
  }

  .primary:not(:disabled):hover::before {
    opacity: 1;
  }

  .primary:not(:disabled):active {
    transform: none;
    scale: 1.02;
    box-shadow:
      0 4px 15px rgba(102, 126, 234, 0.5),
      0 2px 8px rgba(0, 0, 0, 0.2);
  }

  .primary:disabled {
    background: linear-gradient(135deg, #4a5568 0%, #2d3748 100%);
    border-color: transparent;
    color: rgba(255, 255, 255, 0.5);
    box-shadow: none;
    transform: none;
    cursor: not-allowed;
    pointer-events: none;
    opacity: 0.7;
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
    cursor: default;
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
    transition:
      opacity 0.15s ease,
      visibility 0.15s ease,
      transform 0.15s ease;
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
    border: 2px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.08);
    color: #ffffff;
    border-radius: var(--composer-control-radius);
    padding: 0;
    width: var(--composer-row-height);
    height: var(--composer-row-height);
    font-size: 14px;
    font-weight: 500;
    cursor: default;
    transition:
      transform 0.2s ease,
      box-shadow 0.2s ease,
      background 0.2s ease,
      border-color 0.2s ease,
      color 0.2s ease,
      scale 0.2s ease;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    box-sizing: border-box;
    min-width: 0;
    position: relative;
    overflow: hidden;
  }

  .btn-icon-small::before {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.1), transparent);
    transition: left 0.5s ease;
  }

  .btn-icon-small:not(:disabled):hover {
    transform: none;
    background: rgba(255, 255, 255, 0.15);
    border-color: rgba(255, 255, 255, 0.2);
    box-shadow:
      0 8px 25px rgba(0, 0, 0, 0.15),
      0 4px 12px rgba(0, 0, 0, 0.1);
    scale: 1.1;
  }

  .btn-icon-small:not(:disabled):hover::before {
    left: 100%;
  }

  .btn-icon-small:not(:disabled):active {
    transform: none;
    scale: 1.05;
    box-shadow:
      0 4px 15px rgba(0, 0, 0, 0.2),
      0 2px 8px rgba(0, 0, 0, 0.15);
  }

  .btn-icon-small.danger {
    color: var(--danger);
    border-color: rgba(231, 76, 60, 0.3);
    background: rgba(231, 76, 60, 0.1);
  }

  .btn-icon-small.danger:not(:disabled):hover {
    background: rgba(231, 76, 60, 0.2);
    border-color: rgba(231, 76, 60, 0.5);
    box-shadow:
      0 8px 25px rgba(231, 76, 60, 0.3),
      0 4px 12px rgba(0, 0, 0, 0.1);
    scale: 1.1;
  }

  .btn-icon-small.danger:not(:disabled):active {
    background: rgba(231, 76, 60, 0.3);
    border-color: rgba(231, 76, 60, 0.6);
    scale: 1.05;
  }

  .btn-icon-small :global(svg) {
    width: 16px;
    height: 16px;
    color: currentColor;
    display: block;
    flex-shrink: 0;
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
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    animation: fadeIn 0.2s ease-out;
  }

  .modal {
    background: rgba(30, 30, 30, 0.95);
    border-radius: 16px;
    border: 2px solid rgba(255, 255, 255, 0.1);
    max-width: 700px;
    width: 90%;
    max-height: 80vh;
    overflow: hidden;
    box-shadow:
      0 20px 40px rgba(0, 0, 0, 0.3),
      0 8px 25px rgba(0, 0, 0, 0.2),
      0 4px 12px rgba(0, 0, 0, 0.15);
    animation: slideIn 0.3s ease-out;
    backdrop-filter: blur(10px);
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(-20px) scale(0.95);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    background: rgba(30, 30, 30, 0.95);
    border-radius: 16px 16px 0 0;
  }

  .modal-header h3 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: #ffffff;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  }

  .btn-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.08);
    color: #ffffff;
    cursor: default;
    font-size: 18px;
    font-weight: 600;
    transition:
      transform 0.2s ease,
      box-shadow 0.2s ease,
      background 0.2s ease,
      border-color 0.2s ease,
      color 0.2s ease,
      scale 0.2s ease;
    position: relative;
    overflow: hidden;
  }

  .btn-close::before {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.1), transparent);
    transition: left 0.5s ease;
  }

  .btn-close:hover {
    background: rgba(255, 255, 255, 0.15);
    border-color: rgba(255, 255, 255, 0.2);
    box-shadow:
      0 8px 25px rgba(0, 0, 0, 0.15),
      0 4px 12px rgba(0, 0, 0, 0.1);
    scale: 1.1;
  }

  .btn-close:hover::before {
    left: 100%;
  }

  .btn-close:active {
    scale: 1.05;
    box-shadow:
      0 4px 15px rgba(0, 0, 0, 0.2),
      0 2px 8px rgba(0, 0, 0, 0.15);
  }

  .modal-body {
    padding: 16px 20px;
    background: rgba(30, 30, 30, 0.95);
  }

  .code-container {
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-radius: 12px;
    background: rgba(20, 20, 20, 0.8);
    overflow: hidden;
    box-shadow:
      0 4px 12px rgba(0, 0, 0, 0.2),
      inset 0 1px 0 rgba(255, 255, 255, 0.05);
  }

  .code-container pre {
    margin: 0;
    padding: 16px;
    background: transparent;
    overflow-x: auto;
    max-height: 400px;
    overflow-y: auto;
  }

  .code-container code {
    font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', 'Monaco', monospace;
    font-size: 13px;
    line-height: 1.6;
    color: #ffffff;
    background: transparent;
    display: block;
    white-space: pre;
  }

  .modal-footer {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
    padding: 16px 20px;
    background: rgba(30, 30, 30, 0.95);
    border-radius: 0 0 16px 16px;
  }

  /* Light theme adjustments */
  @media (prefers-color-scheme: light) {
    .modal {
      background: rgba(255, 255, 255, 0.95);
      border-color: rgba(0, 0, 0, 0.1);
    }

    .modal-header {
      background: rgba(255, 255, 255, 0.95);
      border-radius: 16px 16px 0 0;
      padding: 16px 20px;
    }

    .modal-header h3 {
      color: #1a1a1a;
    }

    .modal-body {
      background: rgba(255, 255, 255, 0.95);
      padding: 16px 20px;
    }

    .code-container {
      background: rgba(248, 248, 248, 0.8);
      border-color: rgba(0, 0, 0, 0.1);
    }

    .code-container code {
      color: #1a1a1a;
    }

    .modal-footer {
      background: rgba(255, 255, 255, 0.95);
      border-radius: 0 0 16px 16px;
      padding: 16px 20px;
    }

    .btn-close {
      background: rgba(0, 0, 0, 0.05);
      border-color: rgba(0, 0, 0, 0.1);
      color: #1a1a1a;
      border-radius: 8px;
    }

    .btn-close:hover {
      background: rgba(0, 0, 0, 0.1);
      border-color: rgba(0, 0, 0, 0.2);
    }

    .secondary {
      background: rgba(0, 0, 0, 0.05);
      border-color: rgba(0, 0, 0, 0.1);
      color: #1a1a1a;
      border-radius: 8px;
    }

    .secondary:disabled {
      background: rgba(0, 0, 0, 0.03);
      border-color: rgba(0, 0, 0, 0.05);
      color: rgba(26, 26, 26, 0.5);
      opacity: 0.6;
    }

    .primary:disabled {
      background: linear-gradient(135deg, #4a5568 0%, #2d3748 100%);
      border-color: transparent;
      color: rgba(255, 255, 255, 0.5);
      opacity: 0.7;
    }

    .primary {
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
      border-color: transparent;
      color: #ffffff;
      border-radius: 8px;
    }

    .primary:hover {
      background: linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%);
    }
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

    .btn-icon-small :global(svg) {
      width: 16px;
      height: 16px;
      color: currentColor;
      display: block;
      flex-shrink: 0;
    }

    .edit-input {
      background: #2d2d2d;
      border-color: #3a3a3a;
    }

    .edit-input::placeholder {
      color: #bdbdbd;
      opacity: 1;
    }

    .btn-new-chat {
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
      border-color: rgba(255, 255, 255, 0.1);
      color: #ffffff;
      box-shadow:
        0 4px 15px rgba(102, 126, 234, 0.3),
        0 2px 8px rgba(0, 0, 0, 0.1);
    }

    .btn-new-chat:not(:disabled):hover {
      background: linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%);
      box-shadow:
        0 8px 25px rgba(102, 126, 234, 0.4),
        0 4px 12px rgba(0, 0, 0, 0.15);
    }

    .btn-new-chat:not(:disabled):active {
      box-shadow:
        0 4px 15px rgba(102, 126, 234, 0.5),
        0 2px 8px rgba(0, 0, 0, 0.2);
    }

    .btn-new-chat:disabled {
      background: linear-gradient(135deg, #4a5568 0%, #2d3748 100%);
      border-color: transparent;
      color: rgba(255, 255, 255, 0.4);
      box-shadow: none;
    }

    /* Modal dark theme styles - уже применены в основном стиле */
    .modal {
      background: rgba(30, 30, 30, 0.95);
      border-color: rgba(255, 255, 255, 0.1);
    }

    .modal-header {
      background: rgba(30, 30, 30, 0.95);
      border-radius: 16px 16px 0 0;
      padding: 16px 20px;
    }

    .modal-header h3 {
      color: #ffffff;
    }

    .modal-body {
      background: rgba(30, 30, 30, 0.95);
      padding: 16px 20px;
    }

    .code-container {
      background: rgba(20, 20, 20, 0.8);
      border-color: rgba(255, 255, 255, 0.1);
    }

    .modal-footer {
      background: rgba(30, 30, 30, 0.95);
      border-radius: 0 0 16px 16px;
      padding: 16px 20px;
    }

    .btn-close {
      background: rgba(255, 255, 255, 0.08);
      border-color: rgba(255, 255, 255, 0.1);
      color: #ffffff;
      border-radius: 8px;
    }

    .btn-close:hover {
      background: rgba(255, 255, 255, 0.15);
      border-color: rgba(255, 255, 255, 0.2);
    }

    .secondary {
      background: rgba(255, 255, 255, 0.08);
      border-color: rgba(255, 255, 255, 0.1);
      color: #ffffff;
      border-radius: 8px;
    }

    .secondary:disabled {
      background: rgba(255, 255, 255, 0.03);
      border-color: rgba(255, 255, 255, 0.05);
      color: rgba(255, 255, 255, 0.5);
      opacity: 0.6;
    }

    .primary:disabled {
      background: linear-gradient(135deg, #4a5568 0%, #2d3748 100%);
      border-color: transparent;
      color: rgba(255, 255, 255, 0.5);
      opacity: 0.7;
    }

    .primary {
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
      border-color: transparent;
      color: #ffffff;
      border-radius: 8px;
    }

    .primary:hover {
      background: linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%);
    }
  }
</style>
