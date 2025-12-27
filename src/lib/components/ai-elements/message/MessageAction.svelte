<script lang="ts">
  /**
   * Message Action with Tooltip
   *
   * Wraps an action button with a tooltip.
   */
  import {
    Tooltip,
    TooltipContent,
    TooltipProvider,
    TooltipTrigger,
  } from '$lib/components/ui/tooltip';
  import type { Snippet } from 'svelte';

  interface Props {
    tooltip?: Snippet;
    side?: 'top' | 'bottom' | 'left' | 'right';
    class?: string;
    children: Snippet;
    delayDuration?: number;
  }

  let { tooltip, side = 'top', class: className, children, delayDuration = 60 }: Props = $props();
</script>

<TooltipProvider>
  <Tooltip {delayDuration}>
    <TooltipTrigger>
      {@render children()}
    </TooltipTrigger>
    {#if tooltip}
      <TooltipContent {side} class={className}>
        {@render tooltip()}
      </TooltipContent>
    {/if}
  </Tooltip>
</TooltipProvider>
