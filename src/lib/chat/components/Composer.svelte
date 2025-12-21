<script lang="ts">
  import ArrowUp from 'phosphor-svelte/lib/ArrowUp';
  import Stop from 'phosphor-svelte/lib/Stop';
  import Paperclip from 'phosphor-svelte/lib/Paperclip';
  import Broom from 'phosphor-svelte/lib/Broom';
  import Microphone from 'phosphor-svelte/lib/Microphone';
  import SlidersHorizontal from 'phosphor-svelte/lib/SlidersHorizontal';
  import File from 'phosphor-svelte/lib/File';
  import X from 'phosphor-svelte/lib/X';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Alert, AlertDescription, AlertTitle } from '$lib/components/ui/alert';
  import { Separator } from '$lib/components/ui/separator';
  import { cn } from '$lib/utils.js';
  import { experimentalFeatures } from '$lib/stores/experimental-features.svelte';
  import { startVoiceCapture as startVoiceCaptureSession, type VoiceCapture } from '$lib/services/voice-input';
  import { t } from '$lib/i18n';

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

  interface Props {
    prompt?: string;
    busy?: boolean;
    isLoaded?: boolean;
    canStop?: boolean;
    isRecording?: boolean;
    supports_text?: boolean;
    supports_image?: boolean;
    supports_audio?: boolean;
    supports_video?: boolean;
    isLoaderPanelVisible?: boolean;
    isChatHistoryVisible?: boolean;
    hasMessages?: boolean;
    onSend?: () => void;
    onStop?: () => void;
    onClear?: () => void;
    onAttach?: (detail: AttachDetail) => void;
    onVoiceTranscribe?: (text: string) => Promise<void>;
    onToggleLoaderPanel?: () => void;
    onToggleChatHistory?: () => void;
  }

  let {
    prompt = $bindable(''),
    busy = false,
    isLoaded = false,
    canStop = false,
    isRecording = $bindable(false),
    supports_text = true,
    supports_image = false,
    supports_audio = false,
    supports_video = false,
    isLoaderPanelVisible = false,
    isChatHistoryVisible: _isChatHistoryVisible = false,
    hasMessages = false,
    onSend,
    onStop,
    onClear,
    onAttach,
    onVoiceTranscribe,
    onToggleLoaderPanel,
    onToggleChatHistory: _onToggleChatHistory,
  }: Props = $props();

  let fileInput: HTMLInputElement | null = $state(null);
  let textareaElement: HTMLTextAreaElement | null = $state(null);
  const MAX_FILE_SIZE = 20 * 1024 * 1024;
  let attachError: string | null = $state(null);
  let voiceError: string | null = $state(null);
  let errorTimer: ReturnType<typeof setTimeout> | null = null;
  let voiceErrorTimer: ReturnType<typeof setTimeout> | null = null;
  let accept = $derived(buildAccept());
  let attachedFiles: AttachDetail[] = $state([]);
  let voiceCapture: VoiceCapture | null = $state(null);
  let isTranscribing = $state(false);

  let textareaHeight = $state(38);
  const MIN_HEIGHT = 38;
  const MAX_HEIGHT = 120;

  function triggerSend() {
    if (busy || !isLoaded || !prompt.trim()) return;
    onSend?.();
  }

  function triggerStop() {
    if (!canStop) return;
    onStop?.();
  }

  function triggerClear() {
    if (!prompt && !attachError && !voiceError) return;
    prompt = '';
    attachError = null;
    voiceError = null;
    clearErrorTimer();
    clearVoiceErrorTimer();
    onClear?.();
  }

  function triggerVoiceInput() {
    if (isTranscribing) return;
    if (isRecording) {
      void endVoiceCapture();
      return;
    }
    void beginVoiceCapture();
  }

  function triggerSettings() {
    onToggleLoaderPanel?.();
  }

  function removeAttachment(index: number) {
    attachedFiles = attachedFiles.filter((_, i) => i !== index);
  }

  function triggerAttach() {
    fileInput?.click();
  }

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

  function clearVoiceErrorTimer() {
    if (voiceErrorTimer) {
      clearTimeout(voiceErrorTimer);
      voiceErrorTimer = null;
    }
  }

  function setVoiceError(message: string) {
    voiceError = message;
    clearVoiceErrorTimer();
    voiceErrorTimer = setTimeout(() => {
      voiceError = null;
      voiceErrorTimer = null;
    }, 4000);
  }

  async function beginVoiceCapture() {
    try {
      voiceCapture = await startVoiceCaptureSession();
      isRecording = true;
      voiceError = null;
      clearVoiceErrorTimer();
    } catch (err) {
      console.error('Failed to start voice capture', err);
      setVoiceError($t('chat.composer.voice.captureFailed'));
    }
  }

  async function endVoiceCapture() {
    if (!voiceCapture) {
      isRecording = false;
      return;
    }
    isTranscribing = true;
    isRecording = false;
    try {
      const text = await voiceCapture.stop();
      voiceCapture = null;
      await onVoiceTranscribe?.(text);
    } catch (err) {
      console.error('Failed to transcribe voice input', err);
      setVoiceError($t('chat.composer.voice.transcriptionFailed'));
    } finally {
      isTranscribing = false;
    }
  }

  async function handleFileChange(event: Event) {
    const input = event.currentTarget as HTMLInputElement | null;
    const file = input?.files?.[0];
    if (!file) return;

    try {
      if (file.size > MAX_FILE_SIZE) {
        setError($t('errors.file.tooLarge'));
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
        setError($t('chat.composer.errors.imageNotSupported'));
        return;
      }
      if (isAudio && !supports_audio) {
        setError($t('chat.composer.errors.audioNotSupported'));
        return;
      }
      if (isVideo && !supports_video) {
        setError($t('chat.composer.errors.videoNotSupported'));
        return;
      }

      if (isTextLike) {
        const content = await file.text();
        const attachment = { filename: name, content };
        attachedFiles = [...attachedFiles, attachment];
        onAttach?.(attachment);
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
        onAttach?.(attachment);
        attachError = null;
        clearErrorTimer();
      } else {
        setError($t('chat.composer.errors.unsupportedFileType'));
      }
    } catch (err) {
      console.error('Failed to read attachment', err);
      setError($t('chat.composer.errors.fileReadFailed'));
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
    textareaElement.style.height = 'auto';
    const scrollHeight = textareaElement.scrollHeight;
    if (scrollHeight <= MIN_HEIGHT) {
      textareaHeight = MIN_HEIGHT;
    } else if (scrollHeight >= MAX_HEIGHT) {
      textareaHeight = MAX_HEIGHT;
    } else {
      textareaHeight = scrollHeight;
    }
    textareaElement.style.height = `${textareaHeight}px`;
  }

  function handleTextareaInput() {
    adjustTextareaHeight();
  }

  let experimentalReady = $derived(
    experimentalFeatures.initialized && experimentalFeatures.enabled,
  );
  let experimentalStatusMessage = $derived(
    experimentalFeatures.initialized
      ? experimentalFeatures.enabled
        ? null
        : $t('chat.composer.experimental.disabled')
      : $t('chat.composer.experimental.loading'),
  );

  $effect(() => {
    if (prompt !== undefined) {
      setTimeout(() => {
        adjustTextareaHeight();
      }, 0);
    }
  });
</script>

  <div
    class={cn(
      'absolute left-1/2 w-[640px] max-w-[calc(100%-2rem)] -translate-x-1/2 px-0 transition-[top,bottom,transform] duration-300',
      hasMessages ? 'bottom-6 pb-0 bg-transparent' : 'top-1/2 -translate-y-1/2',
    )}
  >
  <div
    class="rounded-2xl border bg-card/80 shadow-xl backdrop-blur supports-[backdrop-filter]:bg-card/70"
  >
    <div class="flex flex-col gap-3 p-3 sm:p-4">
      {#if attachedFiles.length > 0}
        <div class="flex flex-wrap gap-2">
          {#each attachedFiles as attachment, index}
            <Badge
              variant="secondary"
              class="flex items-center gap-2 rounded-lg bg-secondary/60 px-3 py-1 text-xs shadow-sm backdrop-blur"
            >
              <span class="flex items-center gap-1 text-muted-foreground">
                <File size={14} weight="bold" />
                <span class="max-w-[14rem] truncate font-medium">{attachment.filename}</span>
              </span>
              <Button
                variant="ghost"
                size="icon-sm"
                class="h-6 w-6 text-muted-foreground hover:text-foreground"
                onclick={() => removeAttachment(index)}
                aria-label={$t('errors.file.removeFile') + ' ' + attachment.filename}
              >
                <X size={14} weight="bold" />
                <span class="sr-only">{$t('errors.file.removeFile')} {attachment.filename}</span>
              </Button>
            </Badge>
          {/each}
        </div>
      {/if}

      <div class="flex flex-col gap-3">
        <Textarea
          bind:value={prompt}
          bind:ref={textareaElement}
          placeholder={isLoaded
            ? $t('chat.composer.placeholder')
            : $t('chat.composer.placeholderNotLoaded') || $t('chat.composer.placeholder')}
          class="composer-input min-h-[24px] md:min-h-[24px] w-full mx-auto block self-center resize-none bg-card/80 px-4 py-2 mt-3 text-base outline-none rounded-xl"
          style={`height:${textareaHeight}px; margin-top:var(--space-1); margin-bottom:calc(-1 * var(--space-2)); background-color: hsl(var(--card) / 0.8);`}
          onkeydown={handleKeydown}
          oninput={handleTextareaInput}
        />

        <Separator class="bg-transparent opacity-0" />

        <div class="flex flex-wrap items-start justify-between gap-2 px-3 -mt-2">
          <div
            class="flex min-w-0 flex-1 items-center gap-1.5 translate-x-1.5 [&>*]:-translate-y-1.5"
          >
            <Button
              variant="ghost"
              size="icon-sm"
              class="composer-icon-btn rounded-full"
              onclick={triggerAttach}
              disabled={busy || !isLoaded || !experimentalReady}
              aria-label={$t('chat.composer.attach')}
              title={experimentalStatusMessage ?? undefined}
            >
              <Paperclip size={16} weight="bold" />
            </Button>

            <Button
              variant="ghost"
              size="icon-sm"
              class={cn('composer-icon-btn rounded-full', isLoaderPanelVisible && 'text-primary')}
              onclick={triggerSettings}
              disabled={!isLoaded || busy}
              aria-label={$t('chat.composer.loaderSettings')}
            >
              <SlidersHorizontal size={16} weight="bold" />
            </Button>

            {#if prompt || attachError}
              <Button
                variant="ghost"
                size="icon-sm"
                class="composer-icon-btn rounded-full"
                onclick={triggerClear}
                disabled={busy || !isLoaded}
                aria-label={$t('chat.composer.clear')}
              >
                <Broom size={16} weight="bold" />
              </Button>
            {/if}
          </div>

          <div
            class="flex flex-shrink-0 items-center gap-1.5 -translate-x-1.5 [&>*]:-translate-y-1.5"
          >
            <Button
              variant="ghost"
              size="icon-sm"
              class={cn('composer-icon-btn rounded-full', isRecording && 'text-destructive')}
              onclick={triggerVoiceInput}
              disabled={busy || isTranscribing}
              aria-label={isRecording
                ? $t('chat.composer.voice.stopRecording')
                : $t('chat.composer.voice.startRecording')}
              aria-pressed={isRecording}
              title={isTranscribing ? $t('chat.composer.voice.transcribing') : undefined}
            >
              {#if isRecording}
                <Stop size={16} weight="bold" />
              {:else}
                <Microphone size={16} weight="bold" />
              {/if}
            </Button>

            <Button
              variant="default"
              size="icon-sm"
              class="rounded-full"
              onclick={busy ? triggerStop : triggerSend}
              disabled={!isLoaded || (!busy && !prompt.trim())}
              aria-label={busy ? $t('chat.composer.stop') : $t('chat.composer.send')}
            >
              {#if busy}
                <Stop size={16} weight="bold" />
              {:else}
                <ArrowUp size={16} weight="bold" />
              {/if}
              <span class="sr-only"
                >{busy ? $t('chat.composer.stop') : $t('chat.composer.send')}</span
              >
            </Button>
          </div>
        </div>
      </div>

      {#if attachError || voiceError}
        <Alert variant="destructive" class="text-sm">
          <AlertTitle>
            {voiceError
              ? $t('chat.composer.voice.errorTitle')
              : $t('chat.composer.errors.attachmentError')}
          </AlertTitle>
          <AlertDescription>{voiceError ?? attachError}</AlertDescription>
        </Alert>
      {/if}
    </div>
  </div>

  <input class="sr-only" type="file" {accept} bind:this={fileInput} onchange={handleFileChange} />
</div>

<style>
  :global(textarea.composer-input) {
    border: none !important;
    outline: none !important;
    box-shadow: none !important;
    background-color: hsl(var(--card) / 0.8) !important;
  }

  :global(textarea.composer-input:focus),
  :global(textarea.composer-input:focus-visible) {
    border: none !important;
    outline: none !important;
    box-shadow: none !important;
    background-color: hsl(var(--card) / 0.8) !important;
  }

  :global(button.composer-icon-btn),
  :global(button.composer-icon-btn:hover),
  :global(button.composer-icon-btn:focus-visible),
  :global(button.composer-icon-btn:active) {
    background-color: transparent !important;
    border: 1px solid transparent !important;
    outline: none !important;
    box-shadow: none !important;
    color: inherit !important;
  }

  :global(button.composer-icon-btn:hover),
  :global(button.composer-icon-btn:focus-visible) {
    border-color: hsl(var(--border)) !important;
  }
</style>
