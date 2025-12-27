<script lang="ts">
  import { Button } from '$lib/components/ui/button/index.js';
  import { UseClipboard } from '$lib/hooks/use-clipboard.svelte';
  import { cn } from '$lib/components/ai-elements/markdown/utils/utils';
  import Check from 'phosphor-svelte/lib/Check';
  import Copy from 'phosphor-svelte/lib/Copy';
  import X from 'phosphor-svelte/lib/X';
  import { scale } from 'svelte/transition';
  import type { CopyButtonProps } from './types';

  let {
    ref = $bindable(null),
    text,
    icon,
    animationDuration = 500,
    variant = 'ghost',
    size = 'icon',
    onCopy,
    class: className,
    tabindex = -1,
    children,
    ...rest
  }: CopyButtonProps = $props();

  // this way if the user passes text then the button will be the default size
  if (size === 'icon' && children) {
    size = 'default';
  }

  let clipboard = new UseClipboard();
</script>

<Button
  bind:ref
  {variant}
  {size}
  {tabindex}
  class={cn('flex items-center gap-2', className)}
  type="button"
  name="copy"
  onclick={async () => {
    const status = await clipboard.copy(text);

    onCopy?.(status);
  }}
>
  {#if clipboard.status === 'success'}
    <div in:scale={{ duration: animationDuration, start: 0.85 }}>
      <Check class="size-4" weight="bold" />
      <span class="sr-only">Copied</span>
    </div>
  {:else if clipboard.status === 'failure'}
    <div in:scale={{ duration: animationDuration, start: 0.85 }}>
      <X class="size-4" weight="bold" />
      <span class="sr-only">Failed to copy</span>
    </div>
  {:else}
    <div in:scale={{ duration: animationDuration, start: 0.85 }}>
      {#if icon}
        {@render icon()}
      {:else}
        <Copy class="size-4" weight="regular" />
      {/if}
      <span class="sr-only">Copy</span>
    </div>
  {/if}
  {@render children?.()}
</Button>
