<script lang="ts">
  import { cn } from '$lib/components/ai-elements/markdown/utils/utils.js';
  import { CollapsibleTrigger } from '$lib/components/ui/collapsible/index.js';
  import { getReasoningContext } from './reasoning-context.svelte.js';

  interface Props {
    class?: string;
    children?: import('svelte').Snippet;
  }

  let { class: className = '', children, ...props }: Props = $props();

  let reasoningContext = getReasoningContext();

  let thinkingMessage = $derived.by(() => {
    let { isStreaming, duration } = reasoningContext;

    if (isStreaming) {
      return 'Thinking...';
    }
    if (duration < 2) {
      return 'Thought for a moment';
    }
    return `Thought for ${duration} seconds`;
  });
</script>

<CollapsibleTrigger
  class={cn(
    'flex items-center cursor-pointer group/thinking self-start relative select-text',
    'transition-colors',
    reasoningContext.isStreaming || reasoningContext.isOpen 
      ? 'text-foreground' 
      : 'text-muted-foreground hover:text-foreground',
    className,
  )}
  {...props}
>
  {#if children}
    {@render children()}
  {:else}
    <!-- Light bulb icon (visible when collapsed, hidden on hover) -->
    <svg
      class={cn(
        'w-3 absolute left-0 top-1/2 -translate-y-1/2 transition-opacity fill-current',
        reasoningContext.isOpen ? 'opacity-0' : 'opacity-100 group-hover/thinking:opacity-0'
      )}
      viewBox="0 0 14 24"
      fill="none"
    >
      <path d="M0 6.01562C0 9.76562 2.24609 10.6934 2.87109 17.207C2.91016 17.5586 3.10547 17.7832 3.47656 17.7832H9.58984C9.9707 17.7832 10.166 17.5586 10.2051 17.207C10.8301 10.6934 13.0664 9.76562 13.0664 6.01562C13.0664 2.64648 10.1855 0 6.5332 0C2.88086 0 0 2.64648 0 6.01562ZM1.47461 6.01562C1.47461 3.37891 3.78906 1.47461 6.5332 1.47461C9.27734 1.47461 11.5918 3.37891 11.5918 6.01562C11.5918 8.81836 9.73633 9.48242 8.85742 16.3086H4.21875C3.33008 9.48242 1.47461 8.81836 1.47461 6.01562ZM3.44727 19.8926H9.62891C9.95117 19.8926 10.1953 19.6387 10.1953 19.3164C10.1953 19.0039 9.95117 18.75 9.62891 18.75H3.44727C3.125 18.75 2.87109 19.0039 2.87109 19.3164C2.87109 19.6387 3.125 19.8926 3.44727 19.8926ZM6.5332 22.7246C8.04688 22.7246 9.30664 21.9824 9.4043 20.8594H3.67188C3.74023 21.9824 5.00977 22.7246 6.5332 22.7246Z" />
    </svg>
    
    <!-- Arrow icon (visible on hover when collapsed, always visible when open) -->
    <svg
      class={cn(
        'h-4 w-4 absolute transition-all stroke-current',
        reasoningContext.isOpen
          ? 'rotate-0 opacity-100'
          : '-rotate-90 opacity-0 group-hover/thinking:opacity-100'
      )}
      viewBox="0 0 24 24"
      fill="none"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <polyline points="6 9 12 15 18 9"></polyline>
    </svg>

    <span class="ml-6 select-text text-sm">{thinkingMessage}</span>
  {/if}
</CollapsibleTrigger>
