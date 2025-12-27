<script lang="ts">
  import { cn } from '../../../utils.js';
  import { Button } from '$lib/components/ui/button';
  import Copy from 'phosphor-svelte/lib/Copy';
  import Check from 'phosphor-svelte/lib/Check';
  import { useCodeCopyButton } from './code.svelte.js';

  let {
    variant = 'ghost',
    size = 'icon',
    class: className,
  }: {
    variant?: 'default' | 'ghost' | 'outline' | 'secondary' | 'destructive' | 'link';
    size?: 'default' | 'sm' | 'lg' | 'icon';
    class?: string;
  } = $props();

  const copyButton = useCodeCopyButton();
  let copied = $state(false);

  async function handleCopy() {
    await navigator.clipboard.writeText(copyButton.code);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }
</script>

<Button
  class={cn('absolute top-2 right-2 h-8 w-8', className)}
  {variant}
  {size}
  onclick={handleCopy}
>
  {#if copied}
    <Check class="h-4 w-4" />
  {:else}
    <Copy class="h-4 w-4" />
  {/if}
</Button>
