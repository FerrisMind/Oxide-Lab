<script lang="ts">
  import { createEventDispatcher } from "svelte";

  type AttachDetail = {
    filename: string;
    content: string;
  };

  const TEXT_EXTENSIONS = ['txt', 'md', 'markdown', 'json', 'log', 'csv', 'yaml', 'yml', 'xml', 'html'];
  const IMAGE_EXTENSIONS = ['png', 'jpg', 'jpeg', 'webp', 'gif'];
  const AUDIO_EXTENSIONS = ['wav', 'mp3', 'ogg', 'flac', 'm4a'];
  const VIDEO_EXTENSIONS = ['mp4', 'webm', 'mov', 'mkv'];
  const DEFAULT_TEXT_ACCEPT = TEXT_EXTENSIONS.map((ext) => `.${ext}`).join(',');
  const dispatch = createEventDispatcher<{
    send: void;
    stop: void;
    regenerate: void;
    clear: void;
    attach: AttachDetail;
  }>();

  export let prompt: string = "";
  export let busy: boolean = false;
  export let isLoaded: boolean = false;
  export let canStop: boolean = false;
  export let canRegenerate: boolean = false;
  export let supports_text: boolean = true;
  export let supports_image: boolean = false;
  export let supports_audio: boolean = false;
  export let supports_video: boolean = false;

  let fileInput: HTMLInputElement | null = null;
  const MAX_FILE_SIZE = 20 * 1024 * 1024;
  let attachError: string | null = null;
  let errorTimer: ReturnType<typeof setTimeout> | null = null;
  let accept = DEFAULT_TEXT_ACCEPT;

  function triggerSend() {
    if (busy || !isLoaded || !prompt.trim()) return;
    dispatch("send");
  }

  function triggerStop() {
    if (!canStop) return;
    dispatch("stop");
  }

  function triggerRegenerate() {
    if (!canRegenerate || busy || !isLoaded) return;
    dispatch("regenerate");
  }

  function triggerClear() {
    if (!prompt && !attachError) return;
    prompt = "";
    attachError = null;
    clearErrorTimer();
    dispatch("clear");
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
        topLevel === 'text' ||
        TEXT_EXTENSIONS.includes(ext) ||
        mime === 'application/json';
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
        dispatch("attach", { filename: name, content });
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
        dispatch("attach", { filename: name, content: marker });
        attachError = null;
        clearErrorTimer();
      } else {
        setError('Неподдерживаемый тип файла');
      }
    } catch (err) {
      console.error("Failed to read attachment", err);
      setError('Не удалось прочитать файл');
    } finally {
      if (input) input.value = "";
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      triggerSend();
    }
  }
</script>

<div class="composer-wrapper">
  <div class="composer">
    <div class="composer__row composer__row--input">
      <textarea
        class="composer__input"
        bind:value={prompt}
        placeholder="Напишите сообщение..."
        rows="1"
        on:keydown={handleKeydown}
      ></textarea>
    </div>
    <div class="composer__row composer__row--controls">
      <div class="composer__controls composer__controls--left">
        <button
          type="button"
          class="composer__button"
          on:click={triggerAttach}
          disabled={busy}
          aria-label="Прикрепить файл"
        >
          Прикрепить
        </button>
        <button
          type="button"
          class="composer__button"
          on:click={triggerClear}
          disabled={busy || (!prompt && !attachError)}
        >
          Очистить
        </button>
      </div>
      <div class="composer__controls composer__controls--right">
        <button
          type="button"
          class="composer__button"
          on:click={triggerStop}
          disabled={!canStop}
        >
          Стоп
        </button>
        <button
          type="button"
          class="composer__button"
          on:click={triggerRegenerate}
          disabled={!canRegenerate || busy || !isLoaded}
        >
          Перегенерировать
        </button>
        <button
          type="button"
          class="composer__button composer__button--primary"
          on:click={triggerSend}
          disabled={busy || !isLoaded || !prompt.trim()}
        >
          Отправить
        </button>
      </div>
    </div>
    <input
      class="composer__file-input"
      type="file"
      accept={accept}
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
    --composer-primary: var(--accent-2, #e2c6ff);
    --composer-primary-strong: rgba(43, 42, 41, 0.92);
    --composer-primary-strong-hover: rgba(43, 42, 41, 0.82);
    --composer-primary-text: #fdfbf8;

    display: grid;
    grid-template-rows: repeat(2, var(--composer-row-height));
    gap: 12px;
    background: var(--composer-bg);
    border: 2px solid var(--composer-primary);
    border-radius: 16px;
    padding: 12px;
    box-shadow:
      0 0 0 2px rgba(226, 198, 255, 0.25),
      0 8px 24px rgba(0, 0, 0, 0.04);
    flex-shrink: 0;
    width: 100%;
    transition:
      border-color 0.2s ease,
      box-shadow 0.2s ease;
  }

  .composer:hover,
  .composer:focus-within {
    border-color: var(--composer-primary);
    box-shadow:
      0 0 0 3px rgba(226, 198, 255, 0.35),
      0 8px 24px rgba(0, 0, 0, 0.04);
  }

  .composer__row {
    display: flex;
    align-items: stretch;
  }

  .composer__row--input {
    overflow: hidden;
  }

  .composer__row--controls {
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .composer__input {
    width: 100%;
    height: 100%;
    resize: none;
    border: 1px solid var(--composer-control-border);
    border-radius: 12px;
    padding: 0 8px;
    font-size: 12px;
    line-height: calc(var(--composer-row-height) - 4px);
    color: var(--composer-text);
    background: var(--composer-control-bg);
    outline: none;
    box-shadow: inset 0 2px 6px rgba(0, 0, 0, 0.03);
  }

  .composer__input::placeholder {
    color: rgba(0, 0, 0, 0.35);
  }

  .composer__controls {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .composer__controls--right {
    margin-left: auto;
    gap: 8px;
  }

  .composer__button {
    border: 1px solid var(--composer-control-border);
    background: var(--composer-control-bg);
    color: var(--composer-text);
    border-radius: 999px;
    padding: 0 14px;
    height: var(--composer-row-height);
    font-size: 12px;
    cursor: default;
    transition: transform 0.15s ease, box-shadow 0.15s ease, background 0.15s ease;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    box-sizing: border-box;
    min-width: 0;
  }

  .composer__button:not(:disabled) {
    cursor: pointer;
    color: var(--composer-text);
  }

  .composer__button:not(:disabled):hover {
    transform: translateY(-1px);
    background: var(--composer-control-hover-bg);
    border-color: rgba(0, 0, 0, 0.18);
    box-shadow: 0 6px 16px rgba(0, 0, 0, 0.08);
  }

  .composer__button:not(:disabled):active {
    transform: translateY(0);
    box-shadow: none;
  }

  .composer__button:disabled {
    opacity: 0.6;
    color: var(--composer-muted);
  }

  .composer__button--primary {
    background: var(--composer-primary-strong);
    border-color: transparent;
    color: var(--composer-primary-text);
    font-weight: 600;
  }

  .composer__button--primary:not(:disabled):hover {
    background: var(--composer-primary-strong-hover);
  }

  .composer__button--primary:disabled {
    background: rgba(0, 0, 0, 0.1);
    color: rgba(0, 0, 0, 0.35);
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
      grid-template-rows: repeat(2, minmax(52px, auto));
    }

    .composer__row--controls {
      flex-wrap: wrap;
      justify-content: center;
    }

    .composer__controls--right {
      margin-left: 0;
      width: 100%;
      justify-content: center;
    }
  }

  .composer__error {
    margin-top: 8px;
    font-size: 13px;
    color: var(--danger, #c45555);
    width: 100%;
  }
</style>
