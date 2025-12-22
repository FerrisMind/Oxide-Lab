<script lang="ts">
  import { cn } from '$lib/utils';
  import type { Snippet } from 'svelte';
  import type { HTMLAttributes } from 'svelte/elements';

  let {
    class: className,
    children,
    ...restProps
  }: {
    class?: string;
    children: Snippet;
  } & HTMLAttributes<HTMLDivElement> = $props();
</script>

<div class={cn('message-actions', className)} {...restProps}>
  {@render children()}
</div>

<style>
  .message-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--muted);
  }
</style>
