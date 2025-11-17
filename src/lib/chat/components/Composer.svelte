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
  import { experimentalFeatures } from '$lib/stores/experimental-features.svelte';

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

  // Переменные для автоматического изменения высоты
  let textareaHeight = 34; // Стандартная высота однострочного поля
  const MIN_HEIGHT = 34; // Минимальная высота (как у кнопок)
  const MAX_HEIGHT = 102; // Максимальная высота (3 строки)

  function triggerSend() {
    if (busy || !isLoaded || !prompt.trim()) return;
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
      if (file.size > MAX_FILE_SIZE) {
        setError('Файл слишком большой. Максимальный размер — 20 МБ.');
        return;
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
        setError('Модель не поддерживает изображения');
        return;
      }
      if (isAudio && !supports_audio) {
        setError('Модель не поддерживает аудио');
        return;
      }
      if (isVideo && !supports_video) {
        setError('Модель не поддерживает видео');
        return;
      }

      if (isTextLike) {
        const content = await file.text();
        const attachment = { filename: name, content };
        attachedFiles = [...attachedFiles, attachment];
        dispatch('attach', attachment);
        attachError = null;
        clearErrorTimer();
        return;
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
      } else {
        setError('Неподдерживаемый тип файла');
      }
    } catch (err) {
      console.error('Failed to read attachment', err);
      setError('Не удалось прочитать файл');
    } finally {
      if (input) input.value = '';
    }
  }

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
  <div class="composer">
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
        {:else}
          <!-- Debug info for history button -->
          <div
            style="position: absolute; top: -30px; left: 0; font-size: 10px; color: red; background: rgba(255,255,255,0.9); padding: 2px;"
          >
            Hist: {experimentalFeatures.enabled ? 'ON' : 'OFF'} | Init: {experimentalFeatures.initialized
              ? 'YES'
              : 'NO'}
          </div>
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
        {:else}
          <!-- Debug info for experimental buttons -->
          <div
            style="position: absolute; top: -50px; right: 0; font-size: 10px; color: red; background: rgba(255,255,255,0.9); padding: 2px;"
          >
            Exp: {experimentalFeatures.enabled ? 'ON' : 'OFF'} | Init: {experimentalFeatures.initialized
              ? 'YES'
              : 'NO'}
          </div>
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
    background: var(--card);
    border: 2px solid transparent;
    border-radius: 16px;
    padding: 12px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.04);
    flex-shrink: 0;
    width: 100%;
    max-width: 768px;
    margin: 0 auto;
    transition:
      border-color 0.2s ease,
      box-shadow 0.2s ease;
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
    color: var(--text);
    background: var(--card);
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
    background: var(--card);
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
    box-shadow: none;
  }

  .composer__button:not(:disabled):hover::before {
    left: 100%;
  }

  .composer__button:not(:disabled):active {
    transform: translateY(0);
    box-shadow: none;
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
    box-shadow: none;
    transform: none;
  }

  .composer__button--primary:not(:disabled):hover::before {
    opacity: 1;
  }

  .composer__button--primary:not(:disabled):active {
    transform: none;
    scale: none;
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
  }

  .composer__button--icon:not(:disabled):active {
    transform: none;
    scale: none;
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
    box-shadow: none;
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
