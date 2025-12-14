<script lang="ts">
  import type { ChatMessage } from '$lib/chat/types';
  import {
    appendSegments,
    finalizeStreaming,
    getAssistantBubbleEl,
    registerAssistantBubble,
  } from '$lib/chat/stream_render';
  import ChatPlaceholder from './ChatPlaceholder.svelte';
  import InferenceMetricsDisplay from './InferenceMetricsDisplay.svelte';
  import UserMessageActions from './UserMessageActions.svelte';
  import { inferenceMetricsStore } from '$lib/stores/inference-metrics';

  // Используем Svelte 5 руны для props
  let {
    messages = $bindable([]),
    messagesEl = $bindable(null),
    showModelNotice = false,
  }: {
    messages?: ChatMessage[];
    messagesEl?: HTMLDivElement | null;
    showModelNotice?: boolean;
  } = $props();
  
  // Производное значение для placeholderOnly
  let placeholderOnly = $derived(showModelNotice && messages.length === 0);
  
  // Получаем store с метриками
  let metricsMap = $state(new Map());
  
  $effect(() => {
    const unsubscribe = inferenceMetricsStore.subscribe((value) => {
      metricsMap = value;
    });
    return unsubscribe;
  });

  // Ре-гидратация ассистентских сообщений при загрузке истории (plain text -> markdown bubble)
  $effect(() => {
    messages.forEach((m, i) => {
      if (m.role !== 'assistant') return;
      if (!m.content || m.content.trim() === '') return;
      const bubble = getAssistantBubbleEl(i);
      if (!bubble) return;
      if (bubble.textContent && bubble.textContent.trim().length > 0) return;

      // #region agent log
      void fetch('http://127.0.0.1:7243/ingest/772f9f1b-e203-482c-aa15-3d8d8eb57ac6', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          sessionId: 'debug-session',
          runId: 'run2',
          hypothesisId: 'H6',
          location: 'MessageList.svelte:rehydrate',
          message: 'rehydrate assistant bubble',
          data: {
            index: i,
            contentLength: m.content.length,
          },
          timestamp: Date.now(),
        }),
      }).catch(() => {});
      // #endregion

      appendSegments(i, bubble as HTMLDivElement, [{ kind: 'text', data: m.content }], false);
      finalizeStreaming(i);

      // Проверим наличие метрик после ре-гидратации (H7)
      const metrics = metricsMap.get(i);
      if (!metrics) {
        // #region agent log
        void fetch('http://127.0.0.1:7243/ingest/772f9f1b-e203-482c-aa15-3d8d8eb57ac6', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            sessionId: 'debug-session',
            runId: 'run2',
            hypothesisId: 'H7',
            location: 'MessageList.svelte:metricsMissing',
            message: 'no metrics for assistant after rehydrate',
            data: { index: i, contentLength: m.content.length },
            timestamp: Date.now(),
          }),
        }).catch(() => {});
        // #endregion
      }
    });
  });

  // Функция для получения mdStreamEl для конкретного сообщения
  function getMdStreamEl(messageIndex: number): HTMLElement | null {
    const bubbleEl = getAssistantBubbleEl(messageIndex);
    if (bubbleEl) {
      return bubbleEl.querySelector('.md-stream');
    }
    return null;
  }
</script>

<style>
  .metrics-container {
    display: flex;
    justify-content: flex-start;
    width: 100%;
    max-width: 80%;
    opacity: 0;
    transition: opacity 0.2s ease;
  }

  .message.assistant:hover .metrics-container {
    opacity: 1;
  }

</style>

<div
  class="messages"
  bind:this={messagesEl}
  class:placeholder-only={placeholderOnly}
  class:with-placeholder={showModelNotice}
>
  {#if showModelNotice}
    <ChatPlaceholder variant="inline" />
  {/if}
  {#each messages as m, i}
    <div class="message {m.role}">
      <div class="bubble">
        {#if m.role === 'assistant'}
          <div class="rich" use:registerAssistantBubble={{ index: i }}></div>
        {:else}
          {m.content}
        {/if}
      </div>
      
      <!-- Отображаем метрики производительности под ответом ассистента -->
      {#if m.role === 'assistant'}
        {@const metrics = metricsMap.get(i)}
        {@const mdStreamEl = getMdStreamEl(i)}
        {#if metrics}
          <div class="metrics-container">
            <InferenceMetricsDisplay {metrics} {mdStreamEl} />
          </div>
        {/if}
      {/if}

      <!-- Отображаем кнопку копирования для сообщений пользователя и ассистента -->
      <UserMessageActions messageContent={m.content} />
      
      <!-- Кнопка будет добавлена здесь через JavaScript -->
    </div>
  {/each}
</div>
