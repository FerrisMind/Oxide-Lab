<script lang="ts">
  import { cn } from '$lib/utils.js';
  import { Button, type ButtonProps } from '$lib/components/ui/button/index.js';
  import ArrowUp from 'phosphor-svelte/lib/ArrowUp';
  import Stop from 'phosphor-svelte/lib/Stop';
  import X from 'phosphor-svelte/lib/X';
  import { Spinner } from '$lib/components/ui/spinner';
  import type { ChatStatus } from './prompt-input-context.svelte.js';

  interface Props extends Omit<ButtonProps, 'size'> {
    status?: ChatStatus;
    size?: 'default' | 'sm' | 'icon' | 'icon-sm';
    children?: import('svelte').Snippet;
  }

  let {
    class: className,
    variant = 'default',
    size = 'icon',
    status,
    children,
    ...props
  }: Props = $props();

  const Icon = $derived.by(() => {
    if (status === 'submitted') {
      return null; // Will render Spinner instead
    } else if (status === 'streaming') {
      return Stop;
    } else if (status === 'error') {
      return X;
    }
    return ArrowUp;
  });
</script>

<Button class={cn('gap-1.5 rounded-lg', className)} {size} type="submit" {variant} {...props}>
  {#if children}
    {@render children()}
  {:else if status === 'submitted'}
    <Spinner size={16} class="animate-spin" />
  {:else if Icon}
    <Icon size={16} weight="bold" />
  {/if}
</Button>
