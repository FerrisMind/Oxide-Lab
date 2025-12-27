<script lang="ts">
  /**
   * Message Actions Container
   *
   * Groups action buttons for a message.
   */
  import { cn } from '../../../utils';
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

<div class={cn('text-muted-foreground flex items-center gap-2', className)} {...restProps}>
  {@render children()}
</div>
