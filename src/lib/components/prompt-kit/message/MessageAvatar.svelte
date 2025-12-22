<script lang="ts">
  import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar/index.js';
  import { cn } from '$lib/utils';

  let {
    src,
    alt,
    fallback,
    class: className,
  }: {
    src: string;
    alt: string;
    fallback?: string;
    class?: string;
  } = $props();
</script>

<Avatar class={cn('message-avatar', className)}>
  <AvatarImage {src} {alt} />
  {#if fallback}
    <AvatarFallback>{fallback}</AvatarFallback>
  {/if}
</Avatar>

<style>
  :global(.message-avatar) {
    height: var(--space-5);
    width: var(--space-5);
    flex-shrink: 0;
  }
</style>
