<script lang="ts">
  import { cn } from '$lib/utils';
  import { MessageClass, setMessageContext, type MessageSchema } from './message-context.svelte.js';
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  let {
    class: className,
    children,
    ...restProps
  }: MessageSchema & {
    class?: string;
    children: Snippet;
  } & HTMLAttributes<HTMLDivElement> = $props();

  const contextInstance = new MessageClass();
  setMessageContext(contextInstance);
</script>

<div class={cn('message-row', className)} {...restProps}>
  {@render children()}
</div>

<style>
  .message-row {
    display: flex;
    gap: var(--space-3);
  }
</style>
