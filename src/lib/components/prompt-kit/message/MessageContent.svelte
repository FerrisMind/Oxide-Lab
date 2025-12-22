<script lang="ts">
  import { cn } from '$lib/utils';
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  let {
    markdown = false,
    class: className,
    content,
    children,
    ...restProps
  }: {
    content?: string;
    markdown?: boolean;
    class?: string;
    children?: Snippet;
  } & HTMLAttributes<HTMLDivElement> = $props();
</script>

<div class={cn('message-content', className)} {...restProps}>
  {#if children}
    {@render children()}
  {:else if content}
    {content}
  {/if}
</div>

<style>
  .message-content {
    color: var(--text);
    background: var(--panel-bg);
    border-radius: var(--radius-md);
    padding: var(--space-2);
    overflow-wrap: break-word;
    white-space: normal;
  }
</style>
