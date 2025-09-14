<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import PaperPlaneRight from "phosphor-svelte/lib/PaperPlaneRight";
  import Stop from "phosphor-svelte/lib/Stop";
  import Paperclip from "phosphor-svelte/lib/Paperclip";
  import Microphone from "phosphor-svelte/lib/Microphone";
  import X from "phosphor-svelte/lib/X";

  export let prompt = "";
  export let busy = false;
  export let isLoaded = false;
  const dispatch = createEventDispatcher<{ send: void; stop: void; attach: { filename: string; content: string } ; voice: void }>();

  let fileInput: HTMLInputElement | null = null;
  const MAX_FILE_SIZE = 20 * 1024 * 1024; // 20 MB
  let attachError: string | null = null;
  let selectedFileName: string | null = null;

  // Поддерживаемые модальности (из родителя)
  export let supports_text: boolean = false;
  export let supports_image: boolean = false;
  export let supports_audio: boolean = false;
  export let supports_video: boolean = false;

  // Строка для accept у input[type=file]
  let accept = '.txt,.md,.json';
  $: accept = [
    supports_text ? '.txt,.md,.json' : '',
    supports_image ? '.png,.jpg,.jpeg,.webp,.gif' : '',
    supports_audio ? '.wav,.mp3,.ogg,.flac,.m4a' : '',
    supports_video ? '.mp4,.webm,.mov,.mkv' : '',
  ].filter(Boolean).join(',');

  function onSend() { dispatch("send"); }
  function onStop() { dispatch("stop"); }
  function onAttachClick() {
    // trigger hidden file input
    fileInput?.click();
  }
  function onVoice() { dispatch("voice"); }

  async function onFileChange(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input?.files?.[0];
    if (!file) return;

    if (file.size > MAX_FILE_SIZE) {
      attachError = `Файл слишком большой. Максимальный размер — 20 МБ.`;
      // Clear after 4s
      setTimeout(() => {
        attachError = null;
      }, 4000);
      if (fileInput) fileInput.value = '';
      return;
    }
    // remember selected file name for indicator
    selectedFileName = file.name;

    // Определяем тип на основе MIME и расширения
    const mime = file.type || '';
    const top = mime.split('/')[0];
    const ext = (file.name.split('.').pop() || '').toLowerCase();

    const isTextLike = top === 'text' || ['txt', 'md', 'json'].includes(ext) || mime === 'application/json';
    const isImage = top === 'image' || ['png','jpg','jpeg','webp','gif'].includes(ext);
    const isAudio = top === 'audio' || ['wav','mp3','ogg','flac','m4a'].includes(ext);
    const isVideo = top === 'video' || ['mp4','webm','mov','mkv'].includes(ext);

    // Проверяем поддержку модальностей
    if (isImage && !supports_image) {
      attachError = 'Модель не поддерживает изображения';
      return;
    }
    if (isAudio && !supports_audio) {
      attachError = 'Модель не поддерживает аудио';
      return;
    }
    if (isVideo && !supports_video) {
      attachError = 'Модель не поддерживает видео';
      return;
    }

    if (isTextLike) {
      const reader = new FileReader();
      reader.onload = () => {
        const content = typeof reader.result === 'string' ? reader.result : '';
        dispatch('attach', { filename: file.name, content });
        if (fileInput) fileInput.value = '';
      };
      reader.readAsText(file);
      return;
    }

    // Для media создаём object URL и вставляем маркеры в текст чата
    const url = URL.createObjectURL(file);
    let marker: string | null = null;
    if (isImage) marker = `[image: ${url}]`;
    else if (isAudio) marker = `[audio: ${url}]`;
    else if (isVideo) marker = `[video: ${url}]`;
    if (marker) {
      dispatch('attach', { filename: file.name, content: marker });
      if (fileInput) fileInput.value = '';
    } else {
      attachError = 'Неподдерживаемый тип файла';
    }
  }

  function removeSelectedFile() {
    selectedFileName = null;
    attachError = null;
    if (fileInput) fileInput.value = '';
  }
</script>

<div class="composer" style="position:relative">
  {#if selectedFileName}
    <div class="file-indicator" style="position:absolute;top:-36px;left:8px;right:8px;z-index:10;display:flex;align-items:center;gap:8px;background:var(--vscode-input-background);border:1px solid var(--vscode-input-border);padding:6px 8px;border-radius:6px;font-size:12px;">
      <div style="display:flex;align-items:center;gap:8px;overflow:hidden;white-space:nowrap;flex:1">
        <Paperclip size={24} weight="bold" />
        <div style="overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">{selectedFileName}</div>
      </div>
      <button class="composer-btn" on:click={removeSelectedFile} title="Удалить файл" style="background:transparent;border:none;color:var(--vscode-foreground);cursor:pointer;padding:0 6px">✕</button>
    </div>
  {/if}
  <textarea
    id="chat-input"
    rows="4"
    bind:value={prompt}
    placeholder="Напишите сообщение..."
    on:keydown={(e) => { if ((e as KeyboardEvent).key === 'Enter' && !(e as KeyboardEvent).shiftKey) { e.preventDefault(); onSend(); } }}
  ></textarea>
  <div class="composer-buttons">
    <input bind:this={fileInput} type="file" accept=".txt,.md,.json" on:change={onFileChange} style="display:none" />
    <button class="composer-btn attach-btn" on:click={onAttachClick} title="Прикрепить файл" disabled={busy}>
      <Paperclip size={24} weight="bold" />
    </button>
    <button class="composer-btn voice-btn" on:click={onVoice} title="Голосовое сообщение" disabled={busy}>
      <Microphone size={24} weight="bold" />
    </button>
    {#if busy}
      <button class="composer-btn stop-btn" on:click={onStop} title="Остановить генерацию">
        <Stop size={24} weight="bold" />
      </button>
    {:else}
      <button class="composer-btn send-btn" class:send-disabled={!isLoaded} on:click={onSend} disabled={!prompt.trim() || !isLoaded} title="Отправить сообщение">
        <PaperPlaneRight size={24} weight="bold" />
      </button>
    {/if}
  </div>
  {#if attachError}
    <div class="attach-error" style="color: var(--vscode-errorForeground); font-size:12px; margin-top:6px;">{attachError}</div>
  {/if}
  <slot />
</div>


