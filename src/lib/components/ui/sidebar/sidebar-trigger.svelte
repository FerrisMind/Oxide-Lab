<script lang="ts">
  import { Button } from '$lib/components/ui/button/index.js';
  import { cn } from '$lib/components/ai-elements/markdown/utils/utils.js';
  import SidebarSimple from 'phosphor-svelte/lib/SidebarSimple';
  import type { ComponentProps, Snippet } from 'svelte';
  import { useSidebar } from './context.svelte.js';

  let {
    ref = $bindable(null),
    class: className,
    onclick,
    children,
    ...restProps
  }: ComponentProps<typeof Button> & {
    onclick?: (e: MouseEvent) => void;
    children?: Snippet;
  } = $props();

  const sidebar = useSidebar();
  
  function handleClick(e: MouseEvent) {
    console.log('[Trigger] clicked, isMobile:', sidebar.isMobile, 'openMobile:', sidebar.openMobile);
    onclick?.(e);
    sidebar.toggle();
    console.log('[Trigger] after toggle, openMobile:', sidebar.openMobile);
  }
</script>

<Button
  data-sidebar="trigger"
  data-slot="sidebar-trigger"
  variant="ghost"
  size="icon"
  class={cn('size-7', className)}
  type="button"
  onclick={handleClick}
  {...restProps}
>
  {#if children}
    {@render children()}
  {:else}
    <SidebarSimple class="size-4" weight="regular" />
  {/if}
  <span class="sr-only">Toggle Sidebar</span>
</Button>
