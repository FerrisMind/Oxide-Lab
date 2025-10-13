<script lang="ts">
  import { onMount } from 'svelte';
  import { createEventDispatcher } from 'svelte';
  import { get } from 'svelte/store';
  import WarningCircle from 'phosphor-svelte/lib/WarningCircle';
  import XCircle from 'phosphor-svelte/lib/XCircle';
  import Pause from 'phosphor-svelte/lib/Pause';
  import Play from 'phosphor-svelte/lib/Play';
  import X from 'phosphor-svelte/lib/X';
  import Trash from 'phosphor-svelte/lib/Trash';

  import {
    activeDownloads,
    downloadHistory,
    downloadsLoaded,
    pauseDownload,
    resumeDownload,
    cancelDownload,
    removeDownload,
    clearHistory,
  } from '$lib/stores/download-manager';
  import type { DownloadHistoryEntry, DownloadJob } from '$lib/types/local-models';

  const STATUS_LABELS: Record<DownloadJob['status'], string> = {
    queued: 'В очереди',
    downloading: 'Загружается',
    paused: 'Приостановлена',
    completed: 'Завершена',
    error: 'Ошибка',
    cancelled: 'Отменена',
  };

  const dispatch = createEventDispatcher<{ close: void }>();

  let modalElement: HTMLDivElement | undefined;
  let isDragging = false;
  let dragOffset = { x: 0, y: 0 };
  let modalPosition = { x: 0, y: 0 };

  // Resize functionality
  let isResizing = false;
  let resizeDirection = '';
  let initialSize = { width: 720, height: 0 };
  let initialMousePos = { x: 0, y: 0 };
  let minWidth = 400;
  let minHeight = 300;

  function getResizeDirection(event: MouseEvent): string {
    if (!modalElement) return '';

    const rect = modalElement.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;
    const width = rect.width;
    const height = rect.height;
    const borderSize = 8; // Размер границы для resize

    // Определяем направление resize
    const isLeft = x < borderSize;
    const isRight = x > width - borderSize;
    const isTop = y < borderSize;
    const isBottom = y > height - borderSize;

    if (isTop && isLeft) return 'nw-resize';
    if (isTop && isRight) return 'ne-resize';
    if (isBottom && isLeft) return 'sw-resize';
    if (isBottom && isRight) return 'se-resize';
    if (isTop) return 'n-resize';
    if (isBottom) return 's-resize';
    if (isLeft) return 'w-resize';
    if (isRight) return 'e-resize';

    return '';
  }

  function handleMouseDown(event: MouseEvent) {
    if (!modalElement) return;

    const direction = getResizeDirection(event);

    if (direction) {
      // Начинаем resize
      isResizing = true;
      resizeDirection = direction;
      initialMousePos = { x: event.clientX, y: event.clientY };
      const rect = modalElement.getBoundingClientRect();
      initialSize = { width: rect.width, height: rect.height };
      modalElement.style.cursor = direction;
    } else if ((event.target as HTMLElement).closest('.modal-header')) {
      // Начинаем drag
      isDragging = true;
      dragOffset.x = event.clientX - modalPosition.x;
      dragOffset.y = event.clientY - modalPosition.y;
    }
  }

  function handleResize(event: MouseEvent) {
    if (!isResizing || !modalElement) return;

    const deltaX = event.clientX - initialMousePos.x;
    const deltaY = event.clientY - initialMousePos.y;

    let newWidth = initialSize.width;
    let newHeight = initialSize.height;
    let newLeft = modalPosition.x;
    let newTop = modalPosition.y;

    // Логика как у обычного окна - расширяемся в направлении тяги
    if (resizeDirection.includes('e')) {
      // Восток (правая сторона) - расширяемся вправо
      newWidth = Math.max(minWidth, initialSize.width + deltaX);
    }
    if (resizeDirection.includes('s')) {
      // Юг (нижняя сторона) - расширяемся вниз
      newHeight = Math.max(minHeight, initialSize.height + deltaY);
    }
    if (resizeDirection.includes('w')) {
      // Запад (левая сторона) - расширяемся влево
      newWidth = Math.max(minWidth, initialSize.width - deltaX);
      newLeft = modalPosition.x + deltaX;
    }
    if (resizeDirection.includes('n')) {
      // Север (верхняя сторона) - расширяемся вверх
      newHeight = Math.max(minHeight, initialSize.height - deltaY);
      newTop = modalPosition.y + deltaY;
    }

    // Ограничиваем размеры и позицию
    newWidth = Math.max(minWidth, Math.min(newWidth, window.innerWidth - newLeft));
    newHeight = Math.max(minHeight, Math.min(newHeight, window.innerHeight - newTop));
    newLeft = Math.max(0, Math.min(newLeft, window.innerWidth - newWidth));
    newTop = Math.max(0, Math.min(newTop, window.innerHeight - newHeight));

    modalElement.style.width = `${newWidth}px`;
    modalElement.style.height = `${newHeight}px`;
    modalElement.style.transform = `translate(${newLeft}px, ${newTop}px)`;

    modalPosition.x = newLeft;
    modalPosition.y = newTop;
  }

  function handleMouseMove(event: MouseEvent) {
    if (isResizing) {
      handleResize(event);
    } else if (isDragging && modalElement) {
      modalPosition.x = event.clientX - dragOffset.x;
      modalPosition.y = event.clientY - dragOffset.y;

      // Ограничиваем позицию в пределах окна
      const rect = modalElement.getBoundingClientRect();
      const maxX = window.innerWidth - rect.width;
      const maxY = window.innerHeight - rect.height;

      modalPosition.x = Math.max(0, Math.min(modalPosition.x, maxX));
      modalPosition.y = Math.max(0, Math.min(modalPosition.y, maxY));

      modalElement.style.transform = `translate(${modalPosition.x}px, ${modalPosition.y}px)`;
    } else if (!isDragging && !isResizing && modalElement) {
      // Показываем курсор resize при наведении на края
      const direction = getResizeDirection(event);
      modalElement.style.cursor = direction || 'default';
    }
  }

  function handleMouseUp() {
    if (isDragging && modalElement) {
      isDragging = false;
      modalElement.style.cursor = '';
    }
    if (isResizing && modalElement) {
      isResizing = false;
      resizeDirection = '';
      modalElement.classList.remove('resizing');
    }
  }

  onMount(() => {
    // Центрируем окно при открытии
    if (modalElement) {
      const rect = modalElement.getBoundingClientRect();
      modalPosition.x = (window.innerWidth - rect.width) / 2;
      modalPosition.y = (window.innerHeight - rect.height) / 2;
      modalElement.style.transform = `translate(${modalPosition.x}px, ${modalPosition.y}px)`;
    }

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  });

  function handleClose() {
    dispatch('close');
  }

  function stopPropagation(event: MouseEvent) {
    event.stopPropagation();
  }

  function toPercent(job: DownloadJob): number | null {
    const total = job.total_bytes ?? 0;
    if (!total) return null;
    if (total <= 0) return null;
    return Math.min(100, Math.round((job.downloaded_bytes / total) * 100));
  }

  function formatBytes(value?: number | null): string {
    if (!value || value <= 0) return '—';
    const units = ['Б', 'КБ', 'МБ', 'ГБ', 'ТБ'];
    let size = value;
    let index = 0;
    while (size >= 1024 && index < units.length - 1) {
      size /= 1024;
      index += 1;
    }
    const formatted = index === 0 ? size.toFixed(0) : size.toFixed(2);
    return `${formatted} ${units[index]}`;
  }

  function formatSpeed(value?: number | null): string {
    if (!value || value <= 0) return '—';
    return `${formatBytes(value)}/с`;
  }

  function formatEta(value?: number | null): string {
    if (!value || value <= 0) return '—';
    const minutes = Math.floor(value / 60);
    const seconds = Math.floor(value % 60);
    if (minutes > 0) {
      return `${minutes} мин ${seconds.toString().padStart(2, '0')} с`;
    }
    return `${seconds} с`;
  }

  function formatDate(iso?: string | null): string {
    if (!iso) return '—';
    const date = new Date(iso);
    if (Number.isNaN(date.getTime())) return '—';
    return date.toLocaleString('ru-RU', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      handleClose();
    }
    if (event.key === 'Tab' && modalElement) {
      const focusables = modalElement.querySelectorAll<HTMLElement>(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
      );
      if (focusables.length === 0) return;
      const first = focusables[0];
      const last = focusables[focusables.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    }
  }

  async function handlePause(job: DownloadJob) {
    await pauseDownload(job);
  }

  async function handleResume(job: DownloadJob) {
    await resumeDownload(job);
  }

  async function handleCancel(job: DownloadJob) {
    await cancelDownload(job);
  }

  async function handleRemove(entry: DownloadHistoryEntry, deleteFile: boolean) {
    await removeDownload(entry, deleteFile);
  }

  async function handleClearHistory() {
    const history = get(downloadHistory);
    if (!history.length) return;
    await clearHistory();
  }
</script>

<div
  class="download-modal"
  role="dialog"
  aria-modal="false"
  aria-labelledby="download-manager-title"
  tabindex="-1"
  bind:this={modalElement}
  onmousedown={handleMouseDown}
  onclick={stopPropagation}
  onkeydown={handleKeydown}
>
  <header class="modal-header">
    <h2 id="download-manager-title">Менеджер загрузок</h2>
    <button class="icon-button" aria-label="Закрыть" onclick={handleClose}>
      <X size={18} weight="bold" />
    </button>
  </header>

  <section class="modal-section">
    <h3>Активные загрузки</h3>
    {#if $downloadsLoaded && !$activeDownloads.length}
      <p class="empty">Нет активных загрузок.</p>
    {:else if !$downloadsLoaded}
      <p class="empty">Загрузка данных…</p>
    {:else}
      <ul class="download-list">
        {#each $activeDownloads as job (job.id)}
          <li class="download-item">
            <div class="item-header">
              <div>
                <strong>{job.filename}</strong>
                <div class="meta">{STATUS_LABELS[job.status]}</div>
              </div>
              <div class="actions">
                {#if job.status === 'downloading' || job.status === 'queued'}
                  <button class="icon-button" title="Пауза" onclick={() => handlePause(job)}>
                    <Pause size={16} />
                  </button>
                {:else if job.status === 'paused' || job.status === 'error'}
                  <button class="icon-button" title="Возобновить" onclick={() => handleResume(job)}>
                    <Play size={16} />
                  </button>
                {/if}
                <button class="icon-button" title="Отменить" onclick={() => handleCancel(job)}>
                  <XCircle size={16} />
                </button>
              </div>
            </div>
            <div class="progress">
              {#if toPercent(job) !== null}
                <div class="progress-bar">
                  <div class="progress-fill" style={`width: ${toPercent(job)}%`}></div>
                </div>
              {/if}
              <div class="progress-meta">
                <span
                  >{formatBytes(job.downloaded_bytes)} из {formatBytes(
                    job.total_bytes ?? null,
                  )}</span
                >
                <span>Скорость: {formatSpeed(job.speed_bytes_per_sec)}</span>
                <span>Осталось: {formatEta(job.eta_seconds)}</span>
              </div>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </section>

  <section class="modal-section">
    <div class="section-header">
      <h3>История загрузок</h3>
      <button class="clear-button" onclick={handleClearHistory} disabled={!$downloadHistory.length}>
        Очистить историю
      </button>
    </div>
    {#if !$downloadHistory.length}
      <p class="empty">История пуста.</p>
    {:else}
      <ul class="history-list">
        {#each $downloadHistory as entry (entry.id)}
          <li class="history-item">
            <div class="item-header">
              <div>
                <strong>{entry.filename}</strong>
                <div class="meta">
                  {STATUS_LABELS[entry.status]} · {formatDate(entry.finished_at)}
                </div>
              </div>
              <div class="actions">
                <button
                  class="icon-button"
                  title="Удалить запись"
                  onclick={() => handleRemove(entry, false)}
                >
                  <Trash size={16} />
                </button>
                {#if entry.status === 'completed'}
                  <button
                    class="icon-button"
                    title="Удалить запись и файл"
                    onclick={() => handleRemove(entry, true)}
                  >
                    <Trash size={16} weight="fill" />
                  </button>
                {/if}
              </div>
            </div>
            <div class="history-meta">
              <span>Размер: {formatBytes(entry.total_bytes ?? entry.downloaded_bytes)}</span>
              <span>Скачано: {formatBytes(entry.downloaded_bytes)}</span>
              {#if entry.error}
                <span class="error">
                  <WarningCircle size={14} />
                  {entry.error}
                </span>
              {/if}
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </section>
</div>

<style>
  /* Убрано: менеджер загрузок теперь перетаскиваемое окно внутри основного UI */

  .download-modal {
    position: absolute;
    width: 720px;
    min-width: 400px;
    min-height: 300px;
    max-width: calc(100vw - 32px);
    max-height: calc(100vh - 32px);
    background: var(--card);
    border-radius: 16px;
    border: 1px solid var(--border-color);
    box-shadow: var(--shadow-hover);
    display: flex;
    flex-direction: column;
    outline: none;
    overflow: hidden;
    z-index: 1000;
    cursor: default;
    user-select: none;
    resize: none;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 24px;
    border-bottom: 1px solid var(--border-color);
    background: var(--panel-bg);
    cursor: default;
    user-select: none;
    -webkit-user-select: none;
    -moz-user-select: none;
    -ms-user-select: none;
    flex-shrink: 0;
  }

  .modal-header:active {
    cursor: default;
  }

  .modal-header h2 {
    font-size: 18px;
    margin: 0;
  }

  .modal-section {
    padding: 20px 24px;
    overflow-y: auto;
    border-bottom: 1px solid var(--border-color);
    flex: 1;
  }

  .modal-section:last-of-type {
    border-bottom: none;
  }

  .modal-section h3 {
    margin: 0 0 12px;
    font-size: 16px;
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }

  .download-list,
  .history-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .download-item,
  .history-item {
    padding: 16px;
    border: 1px solid var(--border-color);
    border-radius: 12px;
    background: var(--card);
    display: flex;
    flex-direction: column;
    gap: 12px;
    box-shadow: var(--shadow);
    transition: box-shadow 0.2s ease;
  }

  .download-item:hover,
  .history-item:hover {
    box-shadow: var(--shadow-hover);
  }

  .item-header {
    display: flex;
    justify-content: space-between;
    gap: 12px;
  }

  .meta {
    color: var(--muted, #6b7280);
    font-size: 13px;
    margin-top: 4px;
  }

  .actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .icon-button {
    border: none;
    background: none;
    padding: 8px;
    border-radius: 8px;
    cursor: default;
    color: var(--text);
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .icon-button:hover {
    background: rgba(59, 130, 246, 0.12);
    transform: translateY(-1px);
  }

  .icon-button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .progress {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .progress-bar {
    height: 8px;
    border-radius: 999px;
    background: var(--panel-bg);
    overflow: hidden;
    border: 1px solid var(--border-color);
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--accent), var(--accent-2));
    transition: width 0.3s ease;
  }

  .progress-meta,
  .history-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    font-size: 13px;
    color: var(--muted, #6b7280);
  }

  .history-meta .error {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: #dc2626;
  }

  .empty {
    margin: 12px 0 0;
    color: var(--muted);
    font-size: 14px;
    text-align: center;
    padding: 20px;
    background: var(--panel-bg);
    border-radius: 8px;
    border: 1px dashed var(--border-color);
  }

  .clear-button {
    border: none;
    padding: 8px 16px;
    border-radius: 8px;
    cursor: default;
    background: var(--accent);
    color: white;
    font-size: 14px;
    font-weight: 500;
    transition: all 0.2s ease;
  }

  .clear-button:hover:not(:disabled) {
    background: var(--accent-2);
    transform: translateY(-1px);
  }

  .clear-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
    transform: none;
  }

  /* Resize handles - курсоры меняются автоматически при наведении на края */

  /* Полностью отключаем выделение текста */
  .download-modal,
  .download-modal * {
    user-select: none !important;
    -webkit-user-select: none !important;
    -moz-user-select: none !important;
    -ms-user-select: none !important;
  }

  /* Курсоры для resize и drag */
  .download-modal {
    cursor: default;
  }

  .modal-header {
    cursor: default;
  }

  .modal-header:active {
    cursor: default;
  }

  @media (max-width: 640px) {
    .download-modal {
      width: calc(100vw - 16px);
      max-height: calc(100vh - 16px);
      border-radius: 8px;
    }
  }
</style>
