<script lang="ts">
  import {
    Reasoning,
    ReasoningTrigger,
    ReasoningContent,
  } from '$lib/components/ai-elements/reasoning/index.js';

  interface Props {
    value?: string;
    labelLoading?: string;
    labelReady?: string;
    streaming?: boolean;
    autoCollapse?: boolean;
    onToggle?: (open: boolean) => void;
  }

  let {
    value = 'thinking',
    labelLoading = 'Thinking...',
    labelReady = 'Thoughts',
    streaming = $bindable(false),
    autoCollapse = true,
    onToggle,
  }: Props = $props();

  // State for HTML content
  let bodyHtml = $state('');
  let isOpen = $state(streaming);

  // Handle open change from Reasoning component
  function handleOpenChange(open: boolean) {
    isOpen = open;
    onToggle?.(open);
  }

  // Exported methods for imperative API (used by markdown_block.ts)
  export function setOpen(next: boolean) {
    isOpen = next;
  }

  export function setStreaming(next: boolean) {
    streaming = next;
    if (streaming) {
      isOpen = true;
    }
  }

  export function appendHtml(html: string) {
    bodyHtml += html;
  }

  export function resetContent() {
    bodyHtml = '';
  }
</script>

<div class="thinking-block" data-think-id={value}>
  <Reasoning
    class="thinking-reasoning"
    isStreaming={streaming}
    bind:open={isOpen}
    defaultOpen={streaming}
    onOpenChange={handleOpenChange}
  >
    <ReasoningTrigger class="thinking-trigger" />
    <ReasoningContent class="thinking-content">
      {@html bodyHtml}
    </ReasoningContent>
  </Reasoning>
</div>

<style>
  .thinking-block {
    width: 100%;
    margin: var(--space-2) 0;
  }

  /* Стили для Reasoning - адаптация Tailwind к CSS */
  :global(.thinking-reasoning) {
    width: 100%;
    margin-bottom: var(--space-3);
  }

  /* Trigger стили - минимальный дизайн как в оригинале */
  :global(.thinking-trigger) {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: auto !important;
    color: var(--muted);
    font-size: var(--font-size-sm);
    transition: color var(--duration-normal) var(--ease-default);
    background: transparent;
    border: none;
    padding: 0;
    cursor: pointer;
  }

  :global(.thinking-trigger:hover) {
    color: var(--text);
  }

  :global(.thinking-trigger p) {
    margin: 0;
  }

  /* Content стили */
  :global(.thinking-content) {
    margin-top: var(--space-3);
    font-size: var(--font-size-sm);
    line-height: var(--line-height-relaxed);
    color: var(--muted);
  }

  :global(.thinking-content p) {
    margin-bottom: var(--space-2);
  }

  :global(.thinking-content p:last-child) {
    margin-bottom: 0;
  }
</style>
