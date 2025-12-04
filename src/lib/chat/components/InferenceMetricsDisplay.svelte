<script lang="ts">
  import type { InferenceMetrics } from '$lib/types/performance';
  import { performanceService } from '$lib/services/performance-service';
  import Speedometer from 'phosphor-svelte/lib/Speedometer';
  import Eye from 'phosphor-svelte/lib/Eye';
  import EyeSlash from 'phosphor-svelte/lib/EyeSlash';
  import Copy from 'phosphor-svelte/lib/Copy';
  import Check from 'phosphor-svelte/lib/Check';

  let {
    metrics = null,
    mdStreamEl = null,
  }: {
    metrics?: InferenceMetrics | null;
    mdStreamEl?: HTMLElement | null;
  } = $props();

  let showingRaw = $state(false);
  let copied = $state(false);

  function toggleRawView() {
    // Если mdStreamEl передан, используем его
    if (mdStreamEl) {
      showingRaw = mdStreamEl.classList.toggle('show-raw');
      return;
    }

    // Иначе ищем md-stream элемент в родительском сообщении
    const metricsEl = document.querySelector('.inference-metrics');
    if (metricsEl) {
      const messageEl = metricsEl.closest('.message.assistant');
      if (messageEl) {
        const mdStream = messageEl.querySelector('.md-stream');
        if (mdStream) {
          showingRaw = mdStream.classList.toggle('show-raw');
        }
      }
    }
  }

  async function copyResponse() {
    try {
      // Ищем элемент сообщения ассистента
      const metricsEl = document.querySelector('.inference-metrics');
      if (metricsEl) {
        const messageEl = metricsEl.closest('.message.assistant');
        if (messageEl) {
          // Ищем содержимое ответа (исключая размышления)
          const bubble = messageEl.querySelector('.bubble');
          if (bubble) {
            // Получаем текст из md-content (основной ответ)
            const mdContent = bubble.querySelector('.md-content');
            let textToCopy = '';

            if (mdContent) {
              // Извлекаем текст из markdown контента
              textToCopy = (mdContent as HTMLElement).innerText || mdContent.textContent || '';
            } else {
              // Fallback: весь текст пузыря
              textToCopy = (bubble as HTMLElement).innerText || bubble.textContent || '';
            }

            // Копируем в буфер обмена
            await navigator.clipboard.writeText(textToCopy);
            copied = true;

            // Возвращаем иконку обратно через 1.5 секунды
            setTimeout(() => {
              copied = false;
            }, 1500);
          }
        }
      }
    } catch (error) {
      console.error('Failed to copy text:', error);
    }
  }
</script>

{#if metrics}
  <div class="inference-metrics">
    <Speedometer size={18} weight="regular" />
    <span class="speed">{performanceService.formatSpeed(metrics.tokens_per_second)}</span>

    <!-- Показываем кнопку всегда для сообщений ассистента -->
    <button
      class="view-button"
      onclick={toggleRawView}
      title={showingRaw ? 'Скрыть исходный код' : 'Показать исходный код'}
    >
      {#if showingRaw}
        <EyeSlash size={18} weight="regular" />
      {:else}
        <Eye size={18} weight="regular" />
      {/if}
    </button>

    <!-- Кнопка копирования -->
    <button class="copy-button" onclick={copyResponse} title="Копировать ответ">
      {#if copied}
        <Check size={18} weight="regular" />
      {:else}
        <Copy size={18} weight="regular" />
      {/if}
    </button>
  </div>
{/if}

<style>
  .inference-metrics {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.4rem;
    font-size: 0.85rem;
    color: rgba(0, 0, 0, 0.5);
  }

  .inference-metrics :global(svg) {
    opacity: 0.7;
  }

  .speed {
    font-weight: 400;
    font-family: 'Segoe UI', system-ui, sans-serif;
  }

  .view-button {
    background: none;
    border: none;
    cursor: default;
    padding: 8px;
    border-radius: 5px;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0.7;
    transition: opacity 0.2s ease;
    color: inherit;
  }

  .view-button:hover {
    opacity: 1;
    background: rgba(0, 0, 0, 0.08);
  }

  .view-button :global(svg) {
    opacity: 1;
  }

  .copy-button {
    background: none;
    border: none;
    cursor: default;
    padding: 8px;
    border-radius: 5px;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0.7;
    transition: opacity 0.2s ease;
    color: inherit;
  }

  .copy-button:hover {
    opacity: 1;
    background: rgba(0, 0, 0, 0.08);
  }

  .copy-button :global(svg) {
    opacity: 1;
  }

  /* Темная тема */
  @media (prefers-color-scheme: dark) {
    .inference-metrics {
      color: rgba(255, 255, 255, 0.6);
    }

    .inference-metrics :global(svg) {
      opacity: 0.7;
    }

    .view-button:hover,
    .copy-button:hover {
      background: rgba(255, 255, 255, 0.08);
    }
  }
</style>
