<script lang="ts">
  /**
   * Message List Component
   *
   * Displays chat messages using ai-elements conversation components.
   * Based on ai-elements conversation-with-actions pattern.
   * Includes support for reasoning/chain-of-thought.
   */
  import type { ChatMessage } from '$lib/chat/types';
  import {
    Conversation,
    ConversationContent,
    ConversationEmptyState,
  } from '$lib/components/ai-elements/conversation';
  import { Message, MessageContent } from '$lib/components/ai-elements/message';
  import {
    Reasoning,
    ReasoningTrigger,
    ReasoningContent,
  } from '$lib/components/ai-elements/reasoning';
  import { Button } from '$lib/components/ui/button';
  import * as Tooltip from '$lib/components/ui/tooltip';
  import { Markdown } from '$lib/components/ai-elements/markdown';
  import Sparkle from 'phosphor-svelte/lib/Sparkle';
  import Copy from 'phosphor-svelte/lib/Copy';
  import ArrowsClockwise from 'phosphor-svelte/lib/ArrowsClockwise';
  import Code from 'phosphor-svelte/lib/Code';
  import PencilSimple from 'phosphor-svelte/lib/PencilSimple';
  import { t } from '$lib/i18n';
  import { cn } from '../../utils';
  import { chatState } from '$lib/stores/chat';

  // Props using Svelte 5 runes
  let {
    messages = $bindable([]),
    showModelNotice = false,
    onRegenerate,
    onEdit,
  }: {
    messages?: ChatMessage[];
    showModelNotice?: boolean;
    onRegenerate?: (index: number) => void;
    onEdit?: (index: number, newContent: string) => void;
  } = $props();

  // State for showing raw response
  let showRawIndex = $state<number | null>(null);
  // State for editing user messages
  let editingIndex = $state<number | null>(null);
  let editingContent = $state('');

  // Derived value for placeholder only state
  let placeholderOnly = $derived(showModelNotice && messages.length === 0);

  // Action handlers
  function handleCopy(content: string) {
    navigator.clipboard.writeText(content);
  }

  function handleRegenerate(index: number) {
    if (onRegenerate) {
      onRegenerate(index);
    }
  }

  function startEdit(index: number, content: string) {
    editingIndex = index;
    editingContent = content;
  }

  function cancelEdit() {
    editingIndex = null;
    editingContent = '';
  }

  function submitEdit() {
    if (editingIndex !== null && onEdit) {
      onEdit(editingIndex, editingContent);
    }
    cancelEdit();
  }

  function toggleRaw(index: number) {
    showRawIndex = showRawIndex === index ? null : index;
  }

  function getRawContent(m: ChatMessage): string {
    let raw = '';
    if (m.thinking) {
      raw += '<think>\n' + m.thinking + '\n</think>\n';
    }
    raw += m.content;
    return raw;
  }
</script>

{#if placeholderOnly}
  <div class="absolute inset-0 flex items-center justify-center">
    <ConversationEmptyState
      title={$t('chat.placeholder.title') || 'Model not loaded'}
      description={$t('chat.placeholder.description') || 'Load a model to start chatting'}
    >
      {#snippet icon()}
        <Sparkle size={48} weight="duotone" class="text-muted-foreground" />
      {/snippet}
    </ConversationEmptyState>
  </div>
{:else}
  <div class="flex flex-col gap-4 sm:gap-6 lg:gap-8 py-6 sm:py-8">
    {#each messages as m, i (i)}
      {@const isAssistant = m.role === 'assistant'}
      {@const isLastMessage = i === messages.length - 1}
      {@const thinkingContent = m.thinking?.replace(/<think>/g, '').trim()}
      {@const hasThinking = isAssistant && thinkingContent}
      {@const isStreaming = isLastMessage && isAssistant && $chatState.busy}
      {@const showActions = isAssistant && !isStreaming && m.content}

      <div
        class={cn(
          'w-full mx-auto px-3 sm:px-4 md:px-6 max-w-2xl lg:max-w-3xl xl:max-w-4xl',
          isAssistant ? 'items-start' : 'items-end flex flex-col',
        )}
      >
        {#if isAssistant}
          <!-- Assistant Message -->
          <div class="flex flex-col group w-full">
            {#if showRawIndex === i}
              <!-- Raw view -->
              <pre class="raw-response text-sm bg-muted/50 p-4 rounded-lg overflow-x-auto whitespace-pre-wrap font-mono">{getRawContent(m)}</pre>
            {:else}
              {#if hasThinking}
                <Reasoning isStreaming={m.isThinking} class="mb-3">
                  <ReasoningTrigger>
                    {$t('chat.thinking.ready') || 'Thoughts'}
                  </ReasoningTrigger>
                  <ReasoningContent>
                    {thinkingContent}
                  </ReasoningContent>
                </Reasoning>
              {/if}

              <Markdown
                content={m.content}
                class="prose prose-sm dark:prose-invert max-w-none"
              />
            {/if}

            <!-- Message Actions (only show when not streaming and has content) -->
            {#if showActions}
              <div
                class={cn(
                  'message-actions mt-2 flex gap-0.5 opacity-0 transition-opacity duration-150 group-hover:opacity-100',
                  isLastMessage && 'opacity-100',
                )}
              >
                <Tooltip.Provider>
                  <Tooltip.Root delayDuration={60}>
                    <Tooltip.Trigger>
                      <Button
                        variant="ghost"
                        size="icon"
                        class="h-8 w-8 text-muted-foreground hover:text-foreground"
                        onclick={() => handleCopy(m.content)}
                      >
                        <Copy class="h-4 w-4" />
                      </Button>
                    </Tooltip.Trigger>
                    <Tooltip.Content>{$t('chat.actions.copy') || 'Copy'}</Tooltip.Content>
                  </Tooltip.Root>
                </Tooltip.Provider>

                <Tooltip.Provider>
                  <Tooltip.Root delayDuration={60}>
                    <Tooltip.Trigger>
                      <Button
                        variant="ghost"
                        size="icon"
                        class="h-8 w-8 text-muted-foreground hover:text-foreground"
                        onclick={() => handleRegenerate(i)}
                      >
                        <ArrowsClockwise class="h-4 w-4" />
                      </Button>
                    </Tooltip.Trigger>
                    <Tooltip.Content
                      >{$t('chat.actions.regenerate') || 'Regenerate'}</Tooltip.Content
                    >
                  </Tooltip.Root>
                </Tooltip.Provider>

                <Tooltip.Provider>
                  <Tooltip.Root delayDuration={60}>
                    <Tooltip.Trigger>
                      <Button
                        variant={showRawIndex === i ? 'secondary' : 'ghost'}
                        size="icon"
                        class="h-8 w-8 text-muted-foreground hover:text-foreground"
                        onclick={() => toggleRaw(i)}
                      >
                        <Code class="h-4 w-4" />
                      </Button>
                    </Tooltip.Trigger>
                    <Tooltip.Content>{showRawIndex === i ? 'Rendered' : 'Raw'}</Tooltip.Content>
                  </Tooltip.Root>
                </Tooltip.Provider>
              </div>
            {/if}
          </div>
        {:else}
          <!-- User Message -->
          <div class="flex flex-col items-end group">
            {#if editingIndex === i}
              <!-- Edit mode -->
              <div class="w-full max-w-[85%] sm:max-w-[75%]">
                <textarea
                  bind:value={editingContent}
                  class="w-full min-h-[80px] p-3 rounded-lg bg-muted border border-border text-foreground resize-y"
                  onkeydown={(e) => {
                    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
                      e.preventDefault();
                      submitEdit();
                    }
                    if (e.key === 'Escape') {
                      cancelEdit();
                    }
                  }}
                ></textarea>
                <div class="flex justify-end gap-2 mt-2">
                  <Button variant="ghost" size="sm" onclick={cancelEdit}>
                    {$t('common.cancel') || 'Cancel'}
                  </Button>
                  <Button variant="default" size="sm" onclick={submitEdit}>
                    {$t('common.save') || 'Save & Submit'}
                  </Button>
                </div>
              </div>
            {:else}
              <!-- Normal view -->
              <div
                class="bg-muted text-foreground max-w-[85%] rounded-3xl rounded-tr-none px-5 py-2.5 sm:max-w-[75%] break-words"
              >
                {m.content}
              </div>

              <!-- User Message Actions -->
              <div
                class="message-actions mt-1 flex justify-end gap-0.5 opacity-0 transition-opacity duration-150 group-hover:opacity-100"
              >
                <Tooltip.Provider>
                  <Tooltip.Root delayDuration={60}>
                    <Tooltip.Trigger>
                      <Button
                        variant="ghost"
                        size="icon"
                        class="h-7 w-7 text-muted-foreground hover:text-foreground"
                        onclick={() => handleCopy(m.content)}
                      >
                        <Copy class="h-3.5 w-3.5" />
                      </Button>
                    </Tooltip.Trigger>
                    <Tooltip.Content>{$t('chat.actions.copy') || 'Copy'}</Tooltip.Content>
                  </Tooltip.Root>
                </Tooltip.Provider>

                <Tooltip.Provider>
                  <Tooltip.Root delayDuration={60}>
                    <Tooltip.Trigger>
                      <Button
                        variant="ghost"
                        size="icon"
                        class="h-7 w-7 text-muted-foreground hover:text-foreground"
                        onclick={() => startEdit(i, m.content)}
                      >
                        <PencilSimple class="h-3.5 w-3.5" />
                      </Button>
                    </Tooltip.Trigger>
                    <Tooltip.Content>{$t('chat.actions.edit') || 'Edit'}</Tooltip.Content>
                  </Tooltip.Root>
                </Tooltip.Provider>
              </div>
            {/if}
          </div>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<style>
  /* ===== Assistant Content (CSS for line-height) ===== */
  .assistant-content {
    color: var(--foreground);
    line-height: 1.6;
  }
</style>
