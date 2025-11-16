<script lang="ts">
  import { onMount } from 'svelte';
  import { createEventDispatcher } from 'svelte';
  import XCircle from 'phosphor-svelte/lib/XCircle';
  import Pause from 'phosphor-svelte/lib/Pause';
  import Play from 'phosphor-svelte/lib/Play';
  import X from 'phosphor-svelte/lib/X';

  import {
    activeDownloads,
    downloadsLoaded,
    pauseDownload,
    resumeDownload,
    cancelDownload,
    ensureDownloadManager,
  } from '$lib/stores/download-manager';
  import type { DownloadJob } from '$lib/types/local-models';

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

    if (isTop && isRight) return 'ne-resize';
    if (isBottom && isLeft) return 'sw-resize';
    if (isBottom && isRight) return 'se-resize';
    if (isBottom) return 's-resize';
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
      // Запад (левая сторона) - расширяемся влево, НО не двигаем окно
      newWidth = Math.max(minWidth, initialSize.width - deltaX);
      // Позиция остаётся той же - окно не двигается при ресайзе слева
    }
    if (resizeDirection.includes('n')) {
      // Север (верхняя сторона) - расширяемся вверх, НО не двигаем окно
      newHeight = Math.max(minHeight, initialSize.height - deltaY);
      // Позиция остаётся той же - окно не двигается при ресайзе сверху
    }

    // Ограничиваем размеры в пределах окна браузера
    const maxWidth = window.innerWidth - newLeft;
    const maxHeight = window.innerHeight - newTop;
    newWidth = Math.max(minWidth, Math.min(newWidth, maxWidth));
    newHeight = Math.max(minHeight, Math.min(newHeight, maxHeight));

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
    void ensureDownloadManager();
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

  type DownloadStatus = DownloadJob['status'];

  type DownloadGroup = {
    id: string;
    title: string;
    jobs: DownloadJob[];
    status: DownloadStatus;
    downloadedBytes: number;
    totalBytes: number | null;
    updatedAt?: string;
  };

  const STATUS_PRIORITY: Record<DownloadStatus, number> = {
    error: 6,
    cancelled: 5,
    downloading: 4,
    queued: 3,
    paused: 2,
    completed: 1,
  };

  function mergeStatus(current: DownloadStatus, incoming: DownloadStatus): DownloadStatus {
    return STATUS_PRIORITY[incoming] > STATUS_PRIORITY[current] ? incoming : current;
  }

  function groupActiveDownloadsList(jobs: DownloadJob[]): DownloadGroup[] {
    const map = new Map<string, DownloadGroup>();
    for (const job of jobs) {
      const key = job.group_id ?? job.id;
      const title = job.display_name ?? job.filename;
      if (!map.has(key)) {
        map.set(key, {
          id: key,
          title,
          jobs: [],
          status: job.status,
          downloadedBytes: 0,
          totalBytes: job.total_bytes ?? null,
          updatedAt: job.updated_at ?? job.started_at,
        });
      }
      const group = map.get(key)!;
      group.jobs = [...group.jobs, job];
      group.status = mergeStatus(group.status, job.status);
      group.downloadedBytes += job.downloaded_bytes;
      if (group.totalBytes !== null && typeof job.total_bytes === 'number') {
        group.totalBytes = (group.totalBytes ?? 0) + job.total_bytes;
      } else if (job.total_bytes === undefined || job.total_bytes === null) {
        group.totalBytes = null;
      }
      group.updatedAt = job.updated_at ?? group.updatedAt;
    }
    return Array.from(map.values()).sort((a, b) => {
      const aTime = a.updatedAt ?? '';
      const bTime = b.updatedAt ?? '';
      return bTime.localeCompare(aTime);
    });
  }

  let groupedActiveDownloads = $derived(groupActiveDownloadsList($activeDownloads));

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

  async function handleGroupPause(group: DownloadGroup) {
    await Promise.all(
      group.jobs
        .filter((job) => job.status === 'downloading' || job.status === 'queued')
        .map((job) => pauseDownload(job)),
    );
  }

  async function handleGroupResume(group: DownloadGroup) {
    await Promise.all(
      group.jobs
        .filter((job) => job.status === 'paused' || job.status === 'error')
        .map((job) => resumeDownload(job)),
    );
  }

  async function handleGroupCancel(group: DownloadGroup) {
    await Promise.all(group.jobs.map((job) => cancelDownload(job)));
  }

  function groupProgressPercent(group: DownloadGroup): number | null {
    if (group.totalBytes === null || group.totalBytes <= 0) {
      return null;
    }
    return Math.min(100, Math.round((group.downloadedBytes / group.totalBytes) * 100));
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
    <h2 id="download-manager-title">Загрузки</h2>
    <button class="icon-button" aria-label="Закрыть" onclick={handleClose}>
      <X size={18} weight="bold" />
    </button>
  </header>

  <section class="modal-section">
    <div class="downloads-content">
      <div class="active-section">
        {#if $downloadsLoaded && !groupedActiveDownloads.length}
          <p class="empty">Нет активных загрузок</p>
        {:else if !$downloadsLoaded}
          <p class="empty">Загрузка данных…</p>
        {:else}
          <ul class="download-list">
            {#each groupedActiveDownloads as group (group.id)}
              <li class="download-item">
                <div class="item-title">
                  <strong>{group.title}</strong>
                </div>
                <div class="item-progress-row">
                  <div class="progress">
                    {#if groupProgressPercent(group) !== null}
                      <div class="progress-bar">
                        <div
                          class="progress-fill"
                          style={`width: ${groupProgressPercent(group)}%`}
                        ></div>
                      </div>
                    {:else}
                      <div class="progress-bar indeterminate">
                        <div class="progress-fill"></div>
                      </div>
                    {/if}
                  </div>
                  <div class="actions">
                    {#if group.jobs.some((job) => job.status === 'downloading' || job.status === 'queued')}
                      <button class="icon-button" title="Пауза" onclick={() => handleGroupPause(group)}>
                        <Pause size={16} />
                      </button>
                    {/if}
                    {#if group.jobs.some((job) => job.status === 'paused' || job.status === 'error')}
                      <button class="icon-button" title="Возобновить" onclick={() => handleGroupResume(group)}>
                        <Play size={16} />
                      </button>
                    {/if}
                    <button class="icon-button" title="Отменить" onclick={() => handleGroupCancel(group)}>
                      <XCircle size={16} />
                    </button>
                  </div>
                </div>
                <div class="progress-meta">
                  <span>
                    {formatBytes(group.downloadedBytes)}
                    {group.totalBytes !== null ? ` из ${formatBytes(group.totalBytes)}` : ''}
                    · 0 MB/s · 00:00
                  </span>
                </div>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    </div>
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
    padding: 14px 24px;
    height: 48px;
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
    display: flex;
    flex-direction: column;
  }

  .modal-section:last-of-type {
    border-bottom: none;
  }

  .downloads-content {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
  }

  .active-section {
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    justify-content: center;
    flex: 1;
  }

  .download-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .download-item {
    padding: 16px;
    border: 1px solid var(--border-color);
    border-radius: 12px;
    background: var(--card);
    display: flex;
    flex-direction: column;
    gap: 8px;
    box-shadow: var(--shadow);
    transition: box-shadow 0.2s ease;
  }

  .download-item:hover {
    box-shadow: var(--shadow-hover);
  }

  .item-title {
    font-size: 14px;
  }

  .item-progress-row {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .item-progress-row .progress {
    flex: 1;
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

  .progress-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    font-size: 13px;
    color: var(--muted, #6b7280);
  }

  .progress-bar.indeterminate .progress-fill {
    width: 40%;
    animation: progress-indeterminate 1.2s linear infinite;
  }

  @keyframes progress-indeterminate {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(250%);
    }
  }

  .empty {
    margin: 12px 0 0;
    color: var(--muted);
    font-size: 14px;
    text-align: center;
    padding: 20px;
    background: var(--card);
    border-radius: 8px;
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
