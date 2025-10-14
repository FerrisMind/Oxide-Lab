<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import ArrowUp from 'phosphor-svelte/lib/ArrowUp';
  import Stop from 'phosphor-svelte/lib/Stop';
  import Paperclip from 'phosphor-svelte/lib/Paperclip';
  import Broom from 'phosphor-svelte/lib/Broom';
  import Microphone from 'phosphor-svelte/lib/Microphone';
  import SlidersHorizontal from 'phosphor-svelte/lib/SlidersHorizontal';
  import ClockCounterClockwise from 'phosphor-svelte/lib/ClockCounterClockwise';
  import File from 'phosphor-svelte/lib/File';
  import X from 'phosphor-svelte/lib/X';
  import { listen } from '@tauri-apps/api/event';
  import { experimentalFeatures } from '$lib/stores/experimental-features.svelte';
  import { loadRagDocument, queryRag, type RagResponse } from '$lib/api/rag';

  type AttachDetail = {
    filename: string;
    content: string;
  };

  const TEXT_EXTENSIONS = [
    'txt',
    'md',
    'markdown',
    'json',
    'log',
    'csv',
    'yaml',
    'yml',
    'xml',
    'html',
    'pdf',
    'docx',
    // Code files
    'js',
    'ts',
    'jsx',
    'tsx',
    'py',
    'rs',
    'go',
    'java',
    'cpp',
    'c',
    'cs',
    'php',
    'rb',
    'swift'
  ];
  const IMAGE_EXTENSIONS = ['png', 'jpg', 'jpeg', 'webp', 'gif'];
  const AUDIO_EXTENSIONS = ['wav', 'mp3', 'ogg', 'flac', 'm4a'];
  const VIDEO_EXTENSIONS = ['mp4', 'webm', 'mov', 'mkv'];
  const DEFAULT_TEXT_ACCEPT = TEXT_EXTENSIONS.map((ext) => `.${ext}`).join(',');
  const dispatch = createEventDispatcher<{
    send: void;
    stop: void;
    clear: void;
    attach: AttachDetail;
    'toggle-loader-panel': void;
    'toggle-chat-history': void;
  }>();

  export let prompt: string = '';
  export let busy: boolean = false;
  export let isLoaded: boolean = false;
  export let canStop: boolean = false;
  export let isRecording: boolean = false;
  export let supports_text: boolean = true;
  export let supports_image: boolean = false;
  export let supports_audio: boolean = false;
  export let supports_video: boolean = false;
  export let isLoaderPanelVisible: boolean = false;
  export let isChatHistoryVisible: boolean = false;

  let fileInput: HTMLInputElement | null = null;
  let textareaElement: HTMLTextAreaElement | null = null;
  const MAX_FILE_SIZE = 20 * 1024 * 1024;
  let attachError: string | null = null;
  let errorTimer: ReturnType<typeof setTimeout> | null = null;
  let accept = DEFAULT_TEXT_ACCEPT;
  let attachedFiles: AttachDetail[] = [];
  let isDragging = $state(false);
  let processingProgress = $state(0);
  let processingStage = $state('');
  let processingMessage = $state<string | null>(null);
  let processingPath = $state<string | null>(null);
  let progressClearTimer: ReturnType<typeof setTimeout> | null = null;
  const PROGRESS_RESET_DELAY = 1500;
  const DEFAULT_RAG_TOP_K = 5;

  // Переменные для автоматического изменения высоты
  let textareaHeight = 34; // Стандартная высота однострочного поля
  const MIN_HEIGHT = 34; // Минимальная высота (как у кнопок)
  const MAX_HEIGHT = 102; // Максимальная высота (3 строки)

  async function triggerSend() {
    if (busy || !isLoaded || !prompt.trim()) return;

    const hasRagDocuments = attachedFiles.some((file) =>
      file.content.startsWith('[RAG Document:'),
    );

    if (hasRagDocuments) {
      try {
        const ragResult = await queryRag(prompt, DEFAULT_RAG_TOP_K);
        if (ragResult?.hits?.length) {
          const contextualPrompt = buildContextualPrompt(prompt, ragResult);
          prompt = contextualPrompt;
        }
      } catch (error) {
        console.warn('RAG query failed', error);
      }
    }

    dispatch('send');
  }

  function triggerStop() {
    if (!canStop) return;
    dispatch('stop');
  }

  function triggerClear() {
    if (!prompt && !attachError) return;
    prompt = '';
    attachError = null;
    clearErrorTimer();
    dispatch('clear');
  }

  function triggerVoiceInput() {
    if (isRecording) {
      // Остановить запись
      isRecording = false;
      console.log('Остановка записи голоса');
    } else {
      // Начать запись
      isRecording = true;
      console.log('Начало записи голоса');
    }
  }

  function triggerSettings() {
    dispatch('toggle-loader-panel');
  }

  function triggerChatHistory() {
    dispatch('toggle-chat-history');
  }

  function removeAttachment(index: number) {
    attachedFiles = attachedFiles.filter((_, i) => i !== index);
    const hasRagDocuments = attachedFiles.some((file) =>
      file.content.startsWith('[RAG Document:'),
    );
    if (!hasRagDocuments) {
      resetProgressIndicators();
    }
  }

  function triggerAttach() {
    if (!fileInput) return;
    fileInput.click();
  }

  $: accept = buildAccept();

  function buildAccept() {
    const extensions: string[] = [];
    if (supports_text) extensions.push(...TEXT_EXTENSIONS.map((ext) => `.${ext}`));
    if (supports_image) extensions.push(...IMAGE_EXTENSIONS.map((ext) => `.${ext}`));
    if (supports_audio) extensions.push(...AUDIO_EXTENSIONS.map((ext) => `.${ext}`));
    if (supports_video) extensions.push(...VIDEO_EXTENSIONS.map((ext) => `.${ext}`));
    return extensions.join(',') || DEFAULT_TEXT_ACCEPT;
  }

  const STAGE_LABELS: Record<string, string> = {
    reading: 'Reading source',
    processing: 'Processing',
    chunking: 'Chunking',
    embedding: 'Embedding',
    indexing: 'Indexing',
    queued: 'Queued',
    complete: 'Complete',
    error: 'Error',
  };

  function describeStage(stage: string): string {
    return STAGE_LABELS[stage] ?? stage;
  }

  function resetProgressIndicators() {
    processingProgress = 0;
    processingStage = '';
    processingMessage = null;
    processingPath = null;
    if (progressClearTimer) {
      clearTimeout(progressClearTimer);
      progressClearTimer = null;
    }
  }

  function scheduleProgressReset() {
    if (progressClearTimer) {
      clearTimeout(progressClearTimer);
      progressClearTimer = null;
    }
    progressClearTimer = setTimeout(() => {
      resetProgressIndicators();
      progressClearTimer = null;
    }, PROGRESS_RESET_DELAY);
  }

  function clearErrorTimer() {
    if (errorTimer) {
      clearTimeout(errorTimer);
      errorTimer = null;
    }
  }

  function setError(message: string) {
    attachError = message;
    clearErrorTimer();
    errorTimer = setTimeout(() => {
      attachError = null;
      errorTimer = null;
    }, 4000);
  }

  async function handleFileChange(event: Event) {
    const input = event.currentTarget as HTMLInputElement | null;
    const file = input?.files?.[0];
    if (!file) return;

    try {
      const handled = await processFile(file);
      if (!handled) {
        setError('Unsupported attachment type');
      }
    } catch (error) {
      console.error('Failed to process file', error);
      setError('Failed to process file');
    } finally {
      if (input) {
        input.value = '';
      }
    }
  }

  async function processFile(file: File): Promise<boolean> {
    const ragHandled = await handleRagUpload(file);
    if (ragHandled) {
      return true;
    }

    if (file.size > MAX_FILE_SIZE) {
    if (file.size > MAX_FILE_SIZE) {
      setError('File is too large. Maximum supported size is 20 MB.');
    }

    const name = file.name;
    const mime = file.type || '';
    const topLevel = mime.split('/')[0];
    const ext = (name.split('.').pop() || '').toLowerCase();

    const isTextLike =
      topLevel === 'text' || TEXT_EXTENSIONS.includes(ext) || mime === 'application/json';
    const isImage = topLevel === 'image' || IMAGE_EXTENSIONS.includes(ext);
    const isAudio = topLevel === 'audio' || AUDIO_EXTENSIONS.includes(ext);
    const isVideo = topLevel === 'video' || VIDEO_EXTENSIONS.includes(ext);

    if (isImage && !supports_image) {
    if (isImage && !supports_image) {
      setError('Image attachments are not supported by the current model');
    }
    if (isAudio && !supports_audio) {
    if (isAudio && !supports_audio) {
      setError('Audio attachments are not supported by the current model');
    }
    if (isVideo && !supports_video) {
    if (isVideo && !supports_video) {
      setError('Video attachments are not supported by the current model');
    }

    if (isTextLike) {
      const content = await file.text();
      const attachment = { filename: name, content };
      attachedFiles = [...attachedFiles, attachment];
      dispatch('attach', attachment);
      attachError = null;
      clearErrorTimer();
      return true;
    }

    let marker: string | null = null;
    if (isImage) {
      const url = URL.createObjectURL(file);
      marker = `[image: ${url}]`;
    } else if (isAudio) {
      const url = URL.createObjectURL(file);
      marker = `[audio: ${url}]`;
    } else if (isVideo) {
      const url = URL.createObjectURL(file);
      marker = `[video: ${url}]`;
    }

    if (marker) {
      const attachment = { filename: name, content: marker };
      attachedFiles = [...attachedFiles, attachment];
      dispatch('attach', attachment);
      attachError = null;
      clearErrorTimer();
      return true;
    }

    return false;
  }

  function extractFilePath(file: File): string | null {
    const candidate =
      (file as any)?.path ?? (file as any)?.pathName ?? (file as any)?.webkitRelativePath;
    if (typeof candidate === 'string' && candidate.length > 0) {
      return candidate;
    }
    return null;
  }

  async function handleRagUpload(file: File): Promise<boolean> {
    const filePath = extractFilePath(file);
    if (!filePath) {
      return false;
    }

    try {
      processingPath = file.name || filePath;
      processingStage = 'queued';
      processingProgress = 0;
      processingMessage = null;

      const doc = await loadRagDocument(filePath);
      const placeholder = `[RAG Document: ${doc.path}]`;
      const attachment = { filename: file.name, content: placeholder };

      const exists = attachedFiles.some(
        (item) => item.filename === attachment.filename && item.content === attachment.content,
      );
      if (!exists) {
        attachedFiles = [...attachedFiles, attachment];
        dispatch('attach', attachment);
      }
      attachError = null;
      clearErrorTimer();
      return true;
    } catch (error) {
      console.error('RAG ingestion failed', error);
      const message = error instanceof Error ? error.message : String(error ?? 'Unknown error');
      processingStage = 'error';
      processingMessage = message;
      scheduleProgressReset();
      setError(`Failed to index document: ${message}`);
      return true;
    }
  }

  function handleDragEnter(event: DragEvent) {
    event.preventDefault();
    isDragging = true;
  }

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    isDragging = true;
  }

  function handleDragLeave(event: DragEvent) {
    if (event.currentTarget instanceof Node && event.relatedTarget instanceof Node) {
      if (event.currentTarget.contains(event.relatedTarget)) {
        return;
      }
    }
    isDragging = false;
  }

  async function handleDrop(event: DragEvent) {
    event.preventDefault();
    isDragging = false;

    const files = event.dataTransfer?.files;
    if (!files || files.length === 0) return;

    for (const file of Array.from(files)) {
      await processFile(file);
    }
  }

  function buildContextualPrompt(question: string, rag: RagResponse): string {
    if (!rag?.hits?.length) {
      return question;
    }

    const context = rag.hits
      .map((hit, index) => {
        const parts: string[] = [`Hit ${index + 1}`];
        if (typeof hit.score === 'number' && Number.isFinite(hit.score)) {
          parts.push(`score ${hit.score.toFixed(3)}`);
        }
        if (hit.document_path) {
          parts.push(`source ${hit.document_path}`);
        }
        const header = parts.join(' | ');
        return `### ${header}\n${hit.text.trim()}`;
      })
      .join('\n\n');

    return `Use the following retrieved context to answer the question.\n\n${context}\n\nQuestion:\n${question}`;
  }

  $effect(() => {
    const unlistenPromise = listen<{
      stage?: string;
      progress?: number;
      path?: string;
      message?: string;
    }>('rag_progress', (event) => {
      const payload = event?.payload ?? {};
      if (typeof payload.path === 'string' && payload.path.length > 0) {
        const normalized = payload.path.replace(/\\/g, '/');
        const segments = normalized.split('/');
        processingPath = segments[segments.length - 1] || payload.path;
      }
      if (typeof payload.stage === 'string') {
        processingStage = payload.stage;
      }
      if (typeof payload.progress === 'number' && Number.isFinite(payload.progress)) {
        const clamped = Math.min(1, Math.max(0, payload.progress));
        processingProgress = clamped;
      }
      if (typeof payload.message === 'string' && payload.message.length > 0) {
        processingMessage = payload.message;
      } else if (payload.stage === 'error') {
        processingMessage = 'Indexing failed';
      } else {
        processingMessage = null;
      }

      if (payload.stage === 'complete') {
        processingProgress = 1;
        scheduleProgressReset();
      } else if (payload.stage === 'error') {
        setError(payload.message ?? 'RAG indexing failed');
        scheduleProgressReset();
      } else if (progressClearTimer) {
        clearTimeout(progressClearTimer);
        progressClearTimer = null;
      }
    });

    return () => {
      unlistenPromise
        .then((fn) => fn())
        .catch(() => {});
      if (progressClearTimer) {
        clearTimeout(progressClearTimer);
        progressClearTimer = null;
      }
    };
  });



  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault();
      triggerSend();
    }
  }

  function adjustTextareaHeight() {
    if (!textareaElement) return;

    // Сбрасываем высоту для корректного расчета
    textareaElement.style.height = 'auto';

    // Получаем scrollHeight для определения необходимой высоты
    const scrollHeight = textareaElement.scrollHeight;

    // Ограничиваем высоту минимальными и максимальными значениями
    if (scrollHeight <= MIN_HEIGHT) {
      textareaHeight = MIN_HEIGHT;
    } else if (scrollHeight >= MAX_HEIGHT) {
      textareaHeight = MAX_HEIGHT;
    } else {
      textareaHeight = scrollHeight;
    }

    // Применяем новую высоту
    textareaElement.style.height = `${textareaHeight}px`;
  }

  function handleTextareaInput() {
    adjustTextareaHeight();
  }

  // Реактивное обновление высоты при изменении prompt
  $: if (prompt !== undefined) {
    // Используем setTimeout для корректного обновления после рендера
    setTimeout(() => {
      adjustTextareaHeight();
    }, 0);
  }
</script>

<div class="composer-wrapper">
  <div
    class="composer"
    class:composer--dragging={isDragging}
    on:dragenter={handleDragEnter}
    on:dragover={handleDragOver}
    on:dragleave={handleDragLeave}
    on:drop={handleDrop}
  >
    {#if attachedFiles.length > 0}
      <div class="composer__row composer__row--attachments">
        {#each attachedFiles as attachment, index}
          <div class="composer__attachment">
            <div class="composer__attachment-icon">
              <File size={16} weight="bold" />
            </div>
            <span class="composer__attachment-name">{attachment.filename}</span>
            <button
              type="button"
              class="composer__attachment-remove"
              on:click={() => removeAttachment(index)}
              aria-label="Удалить файл"
            >
              <X size={12} weight="bold" />
            </button>
          </div>
        {/each}
      </div>
    {/if}
    {#if processingStage}
      <div class="composer__rag-status">
        <div class="composer__rag-status-header">
          <span class="composer__rag-status-stage">{describeStage(processingStage)}</span>
          <span class="composer__rag-status-percent">
            {Math.round(Math.min(processingProgress, 1) * 100)}%
          </span>
        </div>
        <div class="composer__rag-status-bar">
          <div
            class="composer__rag-status-bar-fill"
            style={`width: ${Math.min(processingProgress, 1) * 100}%`}
          ></div>
        </div>
        {#if processingMessage}
          <div class="composer__rag-status-message">{processingMessage}</div>
        {/if}
        {#if processingPath}
          <div class="composer__rag-status-path">{processingPath}</div>
        {/if}
      </div>
    {/if}
    <div class="composer__row composer__row--input">
      <textarea
        class="composer__input"
        bind:value={prompt}
        bind:this={textareaElement}
        placeholder="Напишите сообщение..."
        rows="1"
        on:keydown={handleKeydown}
        on:input={handleTextareaInput}
        style="height: {textareaHeight}px; overflow-y: {textareaHeight >= MAX_HEIGHT
          ? 'auto'
          : 'hidden'};"
      ></textarea>
    </div>
    <div class="composer__row composer__row--controls">
      <div class="composer__controls composer__controls--left">
        {#if experimentalFeatures.initialized && experimentalFeatures.enabled}
          <button
            type="button"
            class="composer__button composer__button--icon"
            class:composer__button--settings-active={isChatHistoryVisible}
            on:click={triggerChatHistory}
            disabled={false}
            aria-label={isChatHistoryVisible ? 'Скрыть историю чатов' : 'Показать историю чатов'}
            draggable="false"
          >
            <ClockCounterClockwise size={16} weight="bold" />
          </button>
        {/if}
        <button
          type="button"
          class="composer__button composer__button--icon"
          class:composer__button--settings-active={isLoaderPanelVisible}
          on:click={triggerSettings}
          disabled={false}
          aria-label="Настройки лоадер панели"
          draggable="false"
        >
          <SlidersHorizontal size={16} weight="bold" />
        </button>
        {#if prompt || attachError}
          <button
            type="button"
            class="composer__button composer__button--icon"
            on:click={triggerClear}
            disabled={busy || !isLoaded}
            aria-label="Очистить"
            draggable="false"
          >
            <Broom size={16} weight="bold" />
          </button>
        {/if}
      </div>
      <div class="composer__controls composer__controls--right">
        {#if experimentalFeatures.initialized && experimentalFeatures.enabled}
          <button
            type="button"
            class="composer__button composer__button--icon"
            on:click={triggerAttach}
            disabled={busy || !isLoaded}
            aria-label="Прикрепить файл"
            draggable="false"
          >
            <Paperclip size={16} weight="bold" />
          </button>
          <button
            type="button"
            class="composer__button composer__button--icon"
            on:click={triggerVoiceInput}
            disabled={busy || !isLoaded}
            aria-label={isRecording ? 'Остановить запись' : 'Начать запись голоса'}
            draggable="false"
          >
            {#if isRecording}
              <Stop size={16} weight="bold" />
            {:else}
              <Microphone size={16} weight="bold" />
            {/if}
          </button>
        {/if}
        <button
          type="button"
          class="composer__button composer__button--primary"
          on:click={busy ? triggerStop : triggerSend}
          disabled={!isLoaded || (!busy && !prompt.trim())}
          aria-label={busy ? 'Стоп' : 'Отправить'}
          draggable="false"
        >
          {#if busy}
            <Stop size={16} weight="bold" />
          {:else}
            <ArrowUp size={16} weight="bold" />
          {/if}
        </button>
      </div>
    </div>
    <input
      class="composer__file-input"
      type="file"
      {accept}
      bind:this={fileInput}
      on:change={handleFileChange}
    />
  </div>

  {#if attachError}
    <div class="composer__error">{attachError}</div>
  {/if}
</div>

<style>
  .composer-wrapper {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
    flex-shrink: 0;
  }

  .composer {
    --composer-row-height: 34px;
    --composer-border: rgba(0, 0, 0, 0.08);
    --composer-bg: var(--card, #fcfbfa);
    --composer-text: var(--text, #2b2a29);
    --composer-muted: var(--muted, #6d6a6a);
    --composer-control-bg: rgba(255, 255, 255, 0.7);
    --composer-control-hover-bg: rgba(255, 255, 255, 0.95);
    --composer-control-border: rgba(0, 0, 0, 0.12);
    --composer-control-radius: 12px;
    --composer-primary: var(--accent-2, #e2c6ff);
    --composer-primary-strong: rgba(43, 42, 41, 0.92);
    --composer-primary-strong-hover: rgba(43, 42, 41, 0.82);
    --composer-primary-text: #fdfbf8;

    display: flex;
    flex-direction: column;
    gap: 12px;
    background: #1a1a1a;
    border: 2px solid transparent;
    border-radius: 16px;
    padding: 12px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.04);
    flex-shrink: 0;
    width: 100%;
    transition:
      border-color 0.2s ease,
      box-shadow 0.2s ease;
  }

  .composer--dragging {
    border-color: rgba(102, 126, 234, 0.4);
    background: rgba(102, 126, 234, 0.08);
  }

  .composer:focus-within {
    border-color: rgba(226, 198, 255, 0.35);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.04);
  }

  .composer__row {
    display: flex;
    align-items: stretch;
    width: 100%;
  }

  .composer__row--input {
    overflow: hidden;
    flex: 1 1 auto;
    min-width: 0;
  }

  /* Make input field wider */

  .composer__row--controls {
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .composer__controls {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .composer__controls--left {
    flex-shrink: 0;
  }

  .composer__controls--right {
    margin-left: auto;
    gap: 8px;
    flex-shrink: 0;
  }

  .composer__row--attachments {
    flex-wrap: wrap;
    gap: 8px;
    align-items: flex-start;
  }

  .composer__rag-status {
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(102, 126, 234, 0.3);
    border-radius: var(--composer-control-radius);
    padding: 8px 12px;
    color: var(--composer-text);
  }

  .composer__rag-status-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .composer__rag-status-stage {
    color: rgba(226, 198, 255, 0.9);
  }

  .composer__rag-status-percent {
    color: rgba(255, 255, 255, 0.7);
  }

  .composer__rag-status-bar {
    position: relative;
    width: 100%;
    height: 4px;
    background: rgba(255, 255, 255, 0.12);
    border-radius: 999px;
    overflow: hidden;
  }

  .composer__rag-status-bar-fill {
    position: absolute;
    top: 0;
    left: 0;
    bottom: 0;
    background: linear-gradient(135deg, rgba(102, 126, 234, 0.6), rgba(118, 75, 162, 0.8));
    border-radius: inherit;
    transition: width 0.2s ease;
  }

  .composer__rag-status-message {
    font-size: 11px;
    color: rgba(255, 205, 178, 0.95);
  }

  .composer__rag-status-path {
    font-size: 11px;
    color: rgba(255, 255, 255, 0.6);
    word-break: break-all;
  }

  .composer__attachment {
    display: flex;
    flex-direction: row;
    align-items: center;
    justify-content: flex-start;
    gap: 6px;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: var(--composer-control-radius);
    padding: 0 8px;
    width: auto;
    height: var(--composer-row-height);
    font-size: 11px;
    color: var(--composer-text);
    position: relative;
    transition: background 0.2s ease;
    box-sizing: border-box;
    min-width: 120px;
    max-width: 200px;
  }

  .composer__attachment:hover {
    background: rgba(255, 255, 255, 0.15);
  }

  .composer__attachment-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    color: rgba(255, 255, 255, 0.7);
    flex-shrink: 0;
  }

  .composer__attachment-name {
    width: 100%;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    text-align: left;
    line-height: 1.2;
    flex-grow: 1;
  }

  .composer__attachment-remove {
    display: flex;
    background: none;
    border: none;
    color: var(--composer-muted);
    cursor: default;
    padding: 4px;
    border-radius: 50%;
    transition:
      color 0.2s ease,
      background 0.2s ease;
    width: 20px;
    height: 20px;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .composer__attachment-remove:hover {
    color: var(--composer-text);
    background: rgba(255, 255, 255, 0.2);
  }

  .composer__input {
    width: 100%;
    min-width: 0;
    min-height: var(--composer-row-height);
    max-height: calc(var(--composer-row-height) * 3);
    resize: none;
    border: none; /* Remove border */
    border-radius: var(--composer-control-radius);
    padding: 8px 16px 8px 8px;
    font-size: 14px;
    line-height: 1.2;
    color: #ffffff;
    background: #1a1a1a;
    outline: none;
    box-shadow: none; /* Remove box shadow */
    transition:
      height 0.2s ease,
      background-color 0.2s ease; /* Remove border-color and box-shadow transitions */
    overflow-y: hidden;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    box-sizing: border-box;
    display: block;
    text-align: left;
  }

  .composer__input:focus {
    border-color: transparent; /* Remove focus border */
    background: #1a1a1a;
    box-shadow: none; /* Remove focus shadow */
  }

  .composer__input::placeholder {
    color: rgba(255, 255, 255, 0.4);
    font-style: italic;
  }

  .composer__button {
    border: 2px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.08);
    color: #ffffff;
    border-radius: var(--composer-control-radius);
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
      color 0.2s ease;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    box-sizing: border-box;
    min-width: 0;
    position: relative;
    overflow: hidden;
  }

  .composer__button::before {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.1), transparent);
    transition: left 0.5s ease;
  }

  .composer__button:not(:disabled) {
    cursor: default;
    color: #ffffff;
  }

  .composer__button:not(:disabled):hover {
    transform: none;
    background: rgba(255, 255, 255, 0.15);
    border-color: rgba(255, 255, 255, 0.2);
    box-shadow:
      0 8px 25px rgba(0, 0, 0, 0.15),
      0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .composer__button:not(:disabled):hover::before {
    left: 100%;
  }

  .composer__button:not(:disabled):active {
    transform: translateY(-1px);
    box-shadow:
      0 4px 15px rgba(0, 0, 0, 0.2),
      0 2px 8px rgba(0, 0, 0, 0.15);
  }

  .composer__button:disabled {
    opacity: 0.4;
    color: rgba(255, 255, 255, 0.3);
    cursor: not-allowed;
    transform: none;
  }

  .composer__button--primary {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    border: 2px solid transparent;
    color: #ffffff;
    font-weight: 600;
    box-shadow:
      0 4px 15px rgba(102, 126, 234, 0.3),
      0 2px 8px rgba(0, 0, 0, 0.1);
    width: var(--composer-row-height);
    height: var(--composer-row-height);
    padding: 0;
    border-radius: var(--composer-control-radius);
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: hidden;
  }

  .composer__button--primary::before {
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

  .composer__button--primary:not(:disabled):hover {
    background: linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%);
    box-shadow:
      0 8px 25px rgba(102, 126, 234, 0.4),
      0 4px 12px rgba(0, 0, 0, 0.15);
    transform: none;
    scale: 1.05;
  }

  .composer__button--primary:not(:disabled):hover::before {
    opacity: 1;
  }

  .composer__button--primary:not(:disabled):active {
    transform: none;
    scale: 1.02;
    box-shadow:
      0 4px 15px rgba(102, 126, 234, 0.5),
      0 2px 8px rgba(0, 0, 0, 0.2);
  }

  .composer__button--primary:disabled {
    background: linear-gradient(135deg, #4a5568 0%, #2d3748 100%);
    border-color: transparent;
    color: rgba(255, 255, 255, 0.4);
    box-shadow: none;
    transform: none;
  }

  .composer__button--icon {
    width: var(--composer-row-height);
    height: var(--composer-row-height);
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--composer-control-radius);
    position: relative;
  }

  .composer__button--icon:not(:disabled):hover {
    transform: none;
    scale: 1.1;
  }

  .composer__button--icon:not(:disabled):active {
    transform: none;
    scale: 1.05;
  }

  /* Стили для кнопки настроек в зависимости от состояния панели */
  .composer__button--icon.composer__button--settings-active {
    background: rgba(102, 126, 234, 0.15);
    border-color: rgba(102, 126, 234, 0.3);
  }

  .composer__button--icon.composer__button--settings-active:not(:disabled):hover {
    transform: none;
    background: rgba(102, 126, 234, 0.25);
    border-color: rgba(102, 126, 234, 0.5);
    box-shadow:
      0 8px 25px rgba(102, 126, 234, 0.3),
      0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .composer__button--icon.composer__button--settings-active:not(:disabled):active {
    transform: none;
    background: rgba(102, 126, 234, 0.35);
    border-color: rgba(102, 126, 234, 0.6);
  }

  /* Отключаем ховер для кнопки настроек когда панель неактивна */
  .composer__button--icon:not(.composer__button--settings-active):not(:disabled):hover {
    transform: none;
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(255, 255, 255, 0.1);
    box-shadow: none;
  }

  .composer__file-input {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    border: 0;
  }

  @media (max-width: 720px) {
    .composer {
      gap: 16px;
    }

    .composer__row--controls {
      flex-wrap: wrap;
      justify-content: center;
    }

    .composer__controls--left {
      flex-shrink: 1;
    }

    .composer__controls--right {
      margin-left: 0;
      width: 100%;
      justify-content: center;
      flex-shrink: 1;
    }

    .composer__attachment {
      min-width: 100px;
      max-width: 150px;
      padding: 0 6px;
    }

    .composer__attachment-icon {
      margin-bottom: 0;
    }

    .composer__attachment-name {
      font-size: 10px;
    }

    .composer__row--input {
      flex: 1;
    }
  }

  .composer__error {
    margin-top: 8px;
    font-size: 13px;
    color: var(--danger, #c45555);
    width: 100%;
  }
</style>
