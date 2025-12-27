<script lang="ts">
  /**
   * Download Manager Modal
   * 
   * Draggable, resizable modal for managing active downloads.
   * Features: progress bars, pause/resume/cancel, speed & ETA display.
   */
  import { onMount } from 'svelte';
  import * as Card from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import X from 'phosphor-svelte/lib/X';
  import Pause from 'phosphor-svelte/lib/Pause';
  import Play from 'phosphor-svelte/lib/Play';
  import XCircle from 'phosphor-svelte/lib/XCircle';
  import DownloadSimple from 'phosphor-svelte/lib/DownloadSimple';
  import Speedometer from 'phosphor-svelte/lib/Speedometer';
  import Timer from 'phosphor-svelte/lib/Timer';
  import {
    activeDownloads,
    downloadsLoaded,
    pauseDownload,
    resumeDownload,
    cancelDownload,
    ensureDownloadManager,
  } from '$lib/stores/download-manager';
  import type { DownloadJob } from '$lib/types/local-models';
  import { t } from '$lib/i18n';

  interface Props {
    onClose?: () => void;
  }

  let { onClose }: Props = $props();

  let modalElement: HTMLDivElement | undefined;
  let isDragging = $state(false);
  let dragOffset = $state({ x: 0, y: 0 });
  let modalPosition = $state({ x: 0, y: 0 });

  // Resize state
  let isResizing = $state(false);
  let resizeDirection = $state('');
  let initialSize = $state({ width: 600, height: 400 });
  let initialMousePos = $state({ x: 0, y: 0 });
  const minWidth = 400;
  const minHeight = 300;

  // ─────────────────────────────────────────────────────────────
  // Download Group Types
  // ─────────────────────────────────────────────────────────────

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

  // ─────────────────────────────────────────────────────────────
  // Formatting Utils
  // ─────────────────────────────────────────────────────────────

  function formatBytes(value?: number | null): string {
    if (!value || value <= 0) return '—';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let size = value;
    let index = 0;
    while (size >= 1024 && index < units.length - 1) {
      size /= 1024;
      index += 1;
    }
    return index === 0 ? `${size.toFixed(0)} ${units[index]}` : `${size.toFixed(2)} ${units[index]}`;
  }

  function formatSpeed(bytesPerSec?: number | null): string {
    if (!bytesPerSec || bytesPerSec <= 0) return '0 B/s';
    const units = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
    let speed = bytesPerSec;
    let index = 0;
    while (speed >= 1024 && index < units.length - 1) {
      speed /= 1024;
      index += 1;
    }
    return index === 0 ? `${speed.toFixed(0)} ${units[index]}` : `${speed.toFixed(1)} ${units[index]}`;
  }

  function formatTime(seconds?: number | null): string {
    if (!seconds || seconds <= 0) return '—';
    if (seconds < 60) return `${seconds}s`;
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    if (minutes < 60) return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
    const hours = Math.floor(minutes / 60);
    const remainingMinutes = minutes % 60;
    return `${hours}:${remainingMinutes.toString().padStart(2, '0')}:${remainingSeconds.toString().padStart(2, '0')}`;
  }

  function calculateGroupSpeed(group: DownloadGroup): number | null {
    const activeJobs = group.jobs.filter((job) => job.status === 'downloading');
    if (activeJobs.length === 0) return null;
    const totalSpeed = activeJobs.reduce((sum, job) => sum + (job.speed_bytes_per_sec ?? 0), 0);
    return totalSpeed > 0 ? totalSpeed : null;
  }

  function calculateGroupEta(group: DownloadGroup): number | null {
    const speed = calculateGroupSpeed(group);
    if (!speed || speed <= 0 || group.totalBytes === null) return null;
    const remainingBytes = group.totalBytes - group.downloadedBytes;
    if (remainingBytes <= 0) return 0;
    return Math.ceil(remainingBytes / speed);
  }

  function groupProgressPercent(group: DownloadGroup): number | null {
    if (group.totalBytes === null || group.totalBytes <= 0) return null;
    return Math.min(100, Math.round((group.downloadedBytes / group.totalBytes) * 100));
  }

  // ─────────────────────────────────────────────────────────────
  // Drag & Resize
  // ─────────────────────────────────────────────────────────────

  function getResizeDirection(event: MouseEvent): string {
    if (!modalElement) return '';
    const rect = modalElement.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;
    const borderSize = 8;

    const isRight = x > rect.width - borderSize;
    const isBottom = y > rect.height - borderSize;

    if (isBottom && isRight) return 'se-resize';
    if (isBottom) return 's-resize';
    if (isRight) return 'e-resize';
    return '';
  }

  function handleMouseDown(event: MouseEvent) {
    if (!modalElement) return;
    const direction = getResizeDirection(event);

    if (direction) {
      isResizing = true;
      resizeDirection = direction;
      initialMousePos = { x: event.clientX, y: event.clientY };
      const rect = modalElement.getBoundingClientRect();
      initialSize = { width: rect.width, height: rect.height };
    } else if ((event.target as HTMLElement).closest('.modal-header')) {
      isDragging = true;
      dragOffset = { x: event.clientX - modalPosition.x, y: event.clientY - modalPosition.y };
    }
  }

  function handleMouseMove(event: MouseEvent) {
    if (isResizing && modalElement) {
      const deltaX = event.clientX - initialMousePos.x;
      const deltaY = event.clientY - initialMousePos.y;

      let newWidth = initialSize.width;
      let newHeight = initialSize.height;

      if (resizeDirection.includes('e')) {
        newWidth = Math.max(minWidth, initialSize.width + deltaX);
      }
      if (resizeDirection.includes('s')) {
        newHeight = Math.max(minHeight, initialSize.height + deltaY);
      }

      modalElement.style.width = `${newWidth}px`;
      modalElement.style.height = `${newHeight}px`;
    } else if (isDragging && modalElement) {
      modalPosition = {
        x: Math.max(0, Math.min(event.clientX - dragOffset.x, window.innerWidth - modalElement.offsetWidth)),
        y: Math.max(0, Math.min(event.clientY - dragOffset.y, window.innerHeight - modalElement.offsetHeight)),
      };
      modalElement.style.transform = `translate(${modalPosition.x}px, ${modalPosition.y}px)`;
    } else if (modalElement) {
      modalElement.style.cursor = getResizeDirection(event) || 'default';
    }
  }

  function handleMouseUp() {
    isDragging = false;
    isResizing = false;
    resizeDirection = '';
  }

  // ─────────────────────────────────────────────────────────────
  // Actions
  // ─────────────────────────────────────────────────────────────

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

  function handleClose() {
    onClose?.();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      handleClose();
    }
  }

  // ─────────────────────────────────────────────────────────────
  // Lifecycle
  // ─────────────────────────────────────────────────────────────

  onMount(() => {
    void ensureDownloadManager();

    // Center modal on open
    if (modalElement) {
      const rect = modalElement.getBoundingClientRect();
      modalPosition = {
        x: (window.innerWidth - rect.width) / 2,
        y: (window.innerHeight - rect.height) / 2,
      };
      modalElement.style.transform = `translate(${modalPosition.x}px, ${modalPosition.y}px)`;
    }

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  });
</script>

<div
  class="fixed z-[9999] w-[90vw] sm:w-[600px] min-w-[320px] sm:min-w-[400px] min-h-[300px] max-w-[calc(100vw-32px)] max-h-[calc(100vh-32px)] bg-card border rounded-xl shadow-xl flex flex-col overflow-hidden select-none"
  role="dialog"
  aria-modal="false"
  aria-labelledby="download-manager-title"
  tabindex="-1"
  bind:this={modalElement}
  onmousedown={handleMouseDown}
  onkeydown={handleKeydown}
>
  <!-- Header -->
  <header class="modal-header flex items-center justify-between px-3 sm:px-4 py-2 sm:py-3 border-b bg-muted/30 cursor-move shrink-0">
    <h2 id="download-manager-title" class="font-semibold text-sm sm:text-base flex items-center gap-2">
      <DownloadSimple class="size-4 sm:size-5" />
      {$t('common.downloads.title') || 'Downloads'}
    </h2>
    <Button variant="ghost" size="icon" class="size-7 sm:size-8" onclick={handleClose}>
      <X class="size-4" weight="bold" />
    </Button>
  </header>

  <!-- Content -->
  <section class="flex-1 overflow-auto p-3 sm:p-4 custom-scrollbar">
    {#if !$downloadsLoaded}
      <div class="flex items-center justify-center h-full text-muted-foreground">
        {$t('common.downloads.loading') || 'Loading...'}
      </div>
    {:else if groupedActiveDownloads.length === 0}
      <div class="flex flex-col items-center justify-center h-full gap-3 text-muted-foreground">
        <DownloadSimple class="size-12 opacity-30" />
        <p>{$t('common.downloads.noActiveDownloads') || 'No active downloads'}</p>
      </div>
    {:else}
      <div class="space-y-3">
        {#each groupedActiveDownloads as group (group.id)}
          <Card.Root class="p-4">
            <!-- Title -->
            <div class="flex items-center justify-between mb-3">
              <strong class="text-sm truncate max-w-[70%]">{group.title}</strong>
              <Badge variant={group.status === 'downloading' ? 'default' : 'secondary'} class="text-xs">
                {group.status}
              </Badge>
            </div>

            <!-- Progress Bar -->
            <div class="flex items-center gap-3 mb-3">
              <div class="flex-1">
                {#if groupProgressPercent(group) !== null}
                  <div class="h-2 rounded-full bg-muted overflow-hidden">
                    <div
                      class="h-full bg-gradient-to-r from-primary to-blue-400 transition-all duration-300"
                      style="width: {groupProgressPercent(group)}%"
                    ></div>
                  </div>
                {:else}
                  <div class="h-2 rounded-full bg-muted overflow-hidden">
                    <div class="h-full w-[40%] bg-gradient-to-r from-primary to-blue-400 animate-pulse"></div>
                  </div>
                {/if}
              </div>

              <!-- Actions -->
              <div class="flex gap-1">
                {#if group.jobs.some((job) => job.status === 'downloading' || job.status === 'queued')}
                  <Button variant="ghost" size="icon" class="size-7" title={$t('common.downloads.pause') || 'Pause'} onclick={() => handleGroupPause(group)}>
                    <Pause class="size-4" />
                  </Button>
                {/if}
                {#if group.jobs.some((job) => job.status === 'paused' || job.status === 'error')}
                  <Button variant="ghost" size="icon" class="size-7" title={$t('common.downloads.resume') || 'Resume'} onclick={() => handleGroupResume(group)}>
                    <Play class="size-4" />
                  </Button>
                {/if}
                <Button variant="ghost" size="icon" class="size-7 text-destructive" title={$t('common.downloads.cancel') || 'Cancel'} onclick={() => handleGroupCancel(group)}>
                  <XCircle class="size-4" />
                </Button>
              </div>
            </div>

            <!-- Meta -->
            <div class="flex flex-wrap gap-2 text-xs text-muted-foreground">
              <span class="flex items-center gap-1 px-2 py-0.5 rounded border bg-muted/30">
                <DownloadSimple class="size-3" weight="bold" />
                {formatBytes(group.downloadedBytes)}
                {group.totalBytes !== null ? ` / ${formatBytes(group.totalBytes)}` : ''}
              </span>
              <span class="flex items-center gap-1 px-2 py-0.5 rounded border bg-muted/30">
                <Speedometer class="size-3" weight="bold" />
                {formatSpeed(calculateGroupSpeed(group))}
              </span>
              <span class="flex items-center gap-1 px-2 py-0.5 rounded border bg-muted/30">
                <Timer class="size-3" weight="bold" />
                {formatTime(calculateGroupEta(group))}
              </span>
            </div>
          </Card.Root>
        {/each}
      </div>
    {/if}
  </section>

  <!-- Resize Handle (visual indicator) -->
  <div class="absolute bottom-0 right-0 w-4 h-4 cursor-se-resize opacity-50 hover:opacity-100 transition-opacity">
    <svg class="size-4 text-muted-foreground" viewBox="0 0 16 16" fill="currentColor">
      <path d="M14 14H10L14 10V14Z" />
      <path d="M14 8H6L14 0V8Z" opacity="0.5" />
    </svg>
  </div>
</div>
