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
  import DotsThreeVertical from 'phosphor-svelte/lib/DotsThreeVertical';
  import { writeTextFile } from '@tauri-apps/plugin-fs';
  import { save } from '@tauri-apps/plugin-dialog';
  import hljs from 'highlight.js/lib/core';
  import json from 'highlight.js/lib/languages/json';
  import 'highlight.js/styles/github-dark.css';
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
  } from '$lib/components/ui/dropdown-menu';

  // Инициализация highlight.js
  hljs.registerLanguage('json', json);

  let editingSessionId: string | null = $state(null);
  let editingTitle = $state('');
  let showDeleteConfirm: string | null = $state(null);
  let showClearAllConfirm = $state(false);
  let showExportModal = $state(false);
  let exportData = $state('');
  let importInput: HTMLInputElement | undefined = $state();
  let exportCodeElement: HTMLElement | undefined = $state();
  let isDownloading = $state(false);
  let isCopying = $state(false);

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
    importInput?.click();
  }

  function onImportFile(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = async (e) => {
      const content = e.target?.result as string;
      if (await chatHistory.importSession(content)) {
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
    // Стартуем без автосоздания: пользователь сам начинает новую сессию
  });
</script>

<section class="chat-history">
  <div class="chat-history-header">
    <h3>История чатов</h3>
    <div class="header-actions">
      <button
        class="btn-icon"
        onclick={handleNewChat}
        title="Новый чат"
        aria-label="Создать новый чат"
      >
        <Plus size={16} weight="bold" />
      </button>
      <button
        class="btn-icon"
        onclick={handleImport}
        title="Импорт"
        aria-label="Импортировать чат из файла"
      >
        <DownloadSimple size={16} weight="bold" />
      </button>
      <button
        class="btn-icon danger"
        onclick={handleClearAll}
        title="Удалить все чаты"
        aria-label="Удалить все чаты"
      >
        <StackMinus size={16} weight="bold" />
      </button>
    </div>
  </div>

  <input
    type="file"
    accept=".json"
    bind:this={importInput}
    onchange={onImportFile}
    style="display: none;"
  />

  <div class="sessions-list">
    {#if $sortedSessions.length === 0}
      <div class="empty-state">
        <p>Нет сохраненных чатов</p>
        <button class="btn-new-chat" onclick={handleNewChat}>Начать новый чат</button>
      </div>
    {:else}
      {#each $sortedSessions as session (session.id)}
        <div
          class="session-item"
          class:active={$currentSession?.id === session.id}
          onclick={() => handleLoadSession(session.id)}
          onkeydown={(e) => {
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
                onblur={saveEdit}
                onkeydown={(e) => {
                  if (e.key === 'Enter') saveEdit();
                  if (e.key === 'Escape') cancelEdit();
                }}
                onclick={(e) => {
                  e.stopPropagation();
                }}
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
              onclick={(e) => {
                e.stopPropagation();
              }}
              onkeydown={(e) => e.stopPropagation()}
              role="toolbar"
              aria-label="Действия с чатом"
              tabindex="-1"
            >
              <DropdownMenu>
                <DropdownMenuTrigger class="btn-icon-small" aria-label={`Открыть меню чата ${session.title}`}>
                  <DotsThreeVertical size={16} weight="bold" />
                </DropdownMenuTrigger>
                <DropdownMenuContent side="right" align="end">
                  <DropdownMenuItem
                    onSelect={() => startEditing(session)}
                    aria-label={`Переименовать чат ${session.title}`}
                  >
                    <PencilSimpleLine size={16} weight="bold" /> Переименовать
                  </DropdownMenuItem>
                  <DropdownMenuItem
                    onSelect={() => handleExport(session.id)}
                    aria-label={`Экспортировать чат ${session.title}`}
                  >
                    <Export size={16} weight="bold" /> Экспорт
                  </DropdownMenuItem>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem
                    class="danger-item"
                    onSelect={() => confirmDelete(session.id)}
                    aria-label={`Удалить чат ${session.title}`}
                  >
                    <TrashSimple size={16} weight="bold" /> Удалить
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          </div>

          {#if showDeleteConfirm === session.id}
            <div
              class="delete-confirm"
              onclick={(e) => {
                e.stopPropagation();
              }}
              onkeydown={(e) => e.stopPropagation()}
              role="alertdialog"
              aria-labelledby="delete-confirm-title"
              aria-describedby="delete-confirm-desc"
              tabindex="-1"
            >
              <p id="delete-confirm-desc">Удалить этот чат?</p>
              <div class="confirm-actions">
                <button class="secondary" onclick={() => (showDeleteConfirm = null)}>
                  Отмена
                </button>
                <button class="primary danger" onclick={() => handleDelete(session.id)}>
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
    onclick={() => (showExportModal = false)}
    onkeydown={(e) => {
      if (e.key === 'Escape') {
        showExportModal = false;
      }
    }}
    role="presentation"
  >
    <div
      class="modal"
      onclick={(e) => {
        e.stopPropagation();
      }}
      onkeydown={(e) => {
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
          onclick={() => (showExportModal = false)}
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
        <button class="secondary" onclick={copyExportData} disabled={isCopying}>
          {isCopying ? 'Копирование...' : 'Копировать'}
        </button>
        <button
          class="primary"
          onclick={(e) => {
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
    onclick={cancelClearAll}
    onkeydown={(e) => {
      if (e.key === 'Escape') {
        cancelClearAll();
      }
    }}
    role="presentation"
  >
    <div
      class="modal"
      onclick={(e) => {
        e.stopPropagation();
      }}
      onkeydown={(e) => {
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
        <button class="btn-close" onclick={cancelClearAll} aria-label="Закрыть окно подтверждения">
          ×
        </button>
      </div>
      <div class="modal-body">
        <p>Вы уверены, что хотите удалить все чаты? Это действие нельзя отменить.</p>
      </div>
      <div class="modal-footer">
        <button class="secondary" onclick={cancelClearAll}>Отмена</button>
        <button class="primary danger" onclick={confirmClearAll}>Удалить все</button>
      </div>
    </div>
  </div>
{/if}

<style>
  /* ===== Chat History Panel - LoaderPanel Style ===== */

  .chat-history {
    --control-radius: var(--radius);
    --control-padding-y: var(--space-2);
    --control-padding-x: var(--space-3);
    --focus-ring: 0 0 0 3px rgb(179 205 224 / 0.15);
    --composer-row-height: var(--space-5); /* 32px → 34px closest */
    --composer-control-radius: var(--radius-lg);

    width: 100%;
    max-width: 100%;
    overflow-x: hidden;
    overflow-y: visible;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--card);
    border-radius: var(--radius-lg);
    padding: var(--space-3);
  }

  .chat-history-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-3);
    padding-bottom: var(--space-3);
    border-bottom: 1px solid var(--border-color);
    overflow: visible;
  }

  .chat-history-header h3 {
    margin: 0;
    font-size: var(--font-size-base);
    font-weight: var(--font-weight-semibold);
    color: var(--text);
  }

  .header-actions {
    display: flex;
    gap: var(--space-2);
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
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
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
    width: var(--space-3); /* 16px */
    height: var(--space-3); /* 16px */
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
    padding: var(--space-6) var(--space-4);
    text-align: center;
    color: var(--muted);
  }

  .empty-state p {
    margin: 0 0 var(--space-3) 0;
    font-size: var(--font-size-sm);
  }

  /* Primary button style matching LoaderPanel */
  .btn-new-chat {
    border: 2px solid rgba(255, 255, 255, 0.1);
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: #ffffff;
    border-radius: var(--composer-control-radius);
    padding: var(--space-2) var(--space-3);
    height: var(--composer-row-height);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
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
    border-radius: var(--radius-lg);
    padding: var(--space-2) var(--space-3);
    height: var(--composer-row-height);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
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
    border-radius: var(--radius-lg);
    padding: var(--space-2) var(--space-3);
    height: var(--composer-row-height);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-semibold);
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
    gap: var(--space-2);
    margin: var(--space-1) 0; /* 4px → 6px closest */
    padding: var(--space-3);
    border: 1px solid var(--sidebar-border);
    border-radius: var(--control-radius);
    background: var(--background);
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
    height: var(--baseline); /* 4px → 3px closest */
    background: transparent;
    transition: background 0.2s ease;
  }

  .session-item:hover {
    border-color: var(--sidebar-border);
    background: var(--sidebar-accent);
    color: var(--sidebar-accent-foreground);
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .session-item:hover .session-title {
    color: var(--sidebar-accent-foreground);
  }

  .session-item:hover .session-meta {
    color: color-mix(in srgb, var(--sidebar-accent-foreground) 70%, transparent);
  }

  .session-item:hover::before {
    background: var(--sidebar-accent);
  }

  .session-item.active {
    border-color: var(--sidebar-accent);
    background: color-mix(in srgb, var(--sidebar-accent) 80%, var(--background) 20%);
    color: var(--sidebar-accent-foreground);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }

  .session-item.active .session-title {
    color: var(--sidebar-accent-foreground);
  }

  .session-item.active .session-meta {
    color: color-mix(in srgb, var(--sidebar-accent-foreground) 70%, transparent);
  }

  .session-item.active::before {
    background: var(--sidebar-accent);
  }

  .session-content {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
  }

  .session-info {
    flex: 1;
    min-width: 0;
  }

  .session-title {
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
    color: var(--text);
    margin-bottom: var(--space-2);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .session-meta {
    display: flex;
    gap: var(--space-2);
    font-size: var(--font-size-base);
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
    gap: var(--space-2);
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

  :global(.btn-icon-small) {
    border: 1px solid var(--sidebar-border);
    background: var(--background);
    color: var(--text);
    border-radius: var(--composer-control-radius);
    padding: 0;
    width: var(--composer-row-height);
    height: var(--composer-row-height);
    font-size: var(--font-size-sm);
    font-weight: var(--font-weight-medium);
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

  :global(.btn-icon-small:not(:disabled):hover) {
    transform: none;
    background: var(--sidebar-accent);
    border-color: var(--sidebar-accent);
    color: var(--sidebar-accent-foreground);
    box-shadow:
      0 8px 25px rgba(0, 0, 0, 0.15),
      0 4px 12px rgba(0, 0, 0, 0.1);
    scale: 1.05;
  }

  :global(.btn-icon-small:not(:disabled):active) {
    transform: none;
    scale: 1.02;
    box-shadow:
      0 4px 15px rgba(0, 0, 0, 0.2),
      0 2px 8px rgba(0, 0, 0, 0.15);
  }

  :global(.btn-icon-small) :global(svg) {
    width: var(--space-3); /* 16px */
    height: var(--space-3); /* 16px */
    color: currentColor;
    display: block;
    flex-shrink: 0;
  }

  .session-actions :global([data-slot='dropdown-menu-content']) {
    min-width: 200px;
    background: var(--background);
    border: 1px solid var(--sidebar-border);
  }

  .session-actions :global([data-slot='dropdown-menu-item']) {
    gap: var(--space-2);
  }

  .session-actions :global(.danger-item) {
    color: var(--destructive, #ef4444);
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
    font-size: var(--font-size-sm);
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
    margin-top: var(--space-2);
    padding: var(--control-padding-y) var(--control-padding-x);
    border-radius: var(--control-radius);
    background: color-mix(in srgb, #ffb3b3 20%, transparent 80%);
    border: 1px solid #ffb3b3;
  }

  .delete-confirm p {
    margin: 0 0 var(--space-3) 0;
    font-size: var(--font-size-sm);
    color: #3a1f1f;
    font-weight: var(--font-weight-medium);
  }

  .confirm-actions {
    display: flex;
    gap: var(--space-2);
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
    border-radius: var(--radius-lg);
    border: 2px solid rgba(255, 255, 255, 0.1);
    max-width: 704px; /* 88 units */
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
    padding: var(--space-3) var(--space-4);
    background: rgba(30, 30, 30, 0.95);
    border-radius: var(--radius-lg) var(--radius-lg) 0 0;
  }

  .modal-header h3 {
    margin: 0;
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-semibold);
    color: #ffffff;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
  }

  .btn-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: var(--space-5);
    height: var(--space-5);
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-radius: var(--radius-lg);
    background: rgba(255, 255, 255, 0.08);
    color: #ffffff;
    cursor: default;
    font-size: var(--font-size-lg);
    font-weight: var(--font-weight-semibold);
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
    padding: var(--space-3) var(--space-4);
    background: rgba(30, 30, 30, 0.95);
  }

  .code-container {
    border: 2px solid rgba(255, 255, 255, 0.1);
    border-radius: var(--radius-lg);
    background: rgba(20, 20, 20, 0.8);
    overflow: hidden;
    box-shadow:
      0 4px 12px rgba(0, 0, 0, 0.2),
      inset 0 1px 0 rgba(255, 255, 255, 0.05);
  }

  .code-container pre {
    margin: 0;
    padding: var(--space-3);
    background: transparent;
    overflow-x: auto;
    max-height: 400px; /* 50 units */
    overflow-y: auto;
  }

  .code-container code {
    font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', 'Monaco', monospace;
    font-size: var(--font-size-xs); /* 12px → 13px closest */
    line-height: 1.6;
    color: #ffffff;
    background: transparent;
    display: block;
    white-space: pre;
  }

  .modal-footer {
    display: flex;
    gap: var(--space-3);
    justify-content: flex-end;
    padding: var(--space-3) var(--space-4);
    background: rgba(30, 30, 30, 0.95);
    border-radius: 0 0 var(--radius-lg) var(--radius-lg);
  }

  /* Light theme adjustments */
  @media (prefers-color-scheme: light) {
    .modal {
      background: rgba(255, 255, 255, 0.95);
      border-color: rgba(0, 0, 0, 0.1);
    }

    .modal-header {
      background: rgba(255, 255, 255, 0.95);
      border-radius: var(--radius-lg) var(--radius-lg) 0 0;
      padding: var(--space-3) var(--space-4);
    }

    .modal-header h3 {
      color: #1a1a1a;
    }

    .modal-body {
      background: rgba(255, 255, 255, 0.95);
      padding: var(--space-3) var(--space-4);
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
      border-radius: 0 0 var(--radius-lg) var(--radius-lg);
      padding: var(--space-3) var(--space-4);
    }

    .btn-close {
      background: rgba(0, 0, 0, 0.05);
      border-color: rgba(0, 0, 0, 0.1);
      color: #1a1a1a;
      border-radius: var(--radius-lg);
    }

    .btn-close:hover {
      background: rgba(0, 0, 0, 0.1);
      border-color: rgba(0, 0, 0, 0.2);
    }

    .secondary {
      background: rgba(0, 0, 0, 0.05);
      border-color: rgba(0, 0, 0, 0.1);
      color: #1a1a1a;
      border-radius: var(--radius-lg);
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
      border-radius: var(--radius-lg);
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

    :global(.btn-icon-small) :global(svg) {
      width: var(--space-3); /* 16px */
      height: var(--space-3); /* 16px */
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
      border-radius: var(--radius-lg) var(--radius-lg) 0 0;
      padding: var(--space-3) var(--space-4);
    }

    .modal-header h3 {
      color: #ffffff;
    }

    .modal-body {
      background: rgba(30, 30, 30, 0.95);
      padding: var(--space-3) var(--space-4);
    }

    .code-container {
      background: rgba(20, 20, 20, 0.8);
      border-color: rgba(255, 255, 255, 0.1);
    }

    .modal-footer {
      background: rgba(30, 30, 30, 0.95);
      border-radius: 0 0 var(--radius-lg) var(--radius-lg);
      padding: var(--space-3) var(--space-4);
    }

    .btn-close {
      background: rgba(255, 255, 255, 0.08);
      border-color: rgba(255, 255, 255, 0.1);
      color: #ffffff;
      border-radius: var(--radius-lg);
    }

    .btn-close:hover {
      background: rgba(255, 255, 255, 0.15);
      border-color: rgba(255, 255, 255, 0.2);
    }

    .secondary {
      background: rgba(255, 255, 255, 0.08);
      border-color: rgba(255, 255, 255, 0.1);
      color: #ffffff;
      border-radius: var(--radius-lg);
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
      border-radius: var(--radius-lg);
    }

    .primary:hover {
      background: linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%);
    }
  }
</style>
