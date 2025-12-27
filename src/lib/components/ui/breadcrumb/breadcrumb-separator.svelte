<script lang="ts">
  import CaretRight from 'phosphor-svelte/lib/CaretRight';
  import { cn, type WithElementRef } from '$lib/components/ai-elements/markdown/utils/utils';
  import type { HTMLLiAttributes } from 'svelte/elements';

  let {
    ref = $bindable(null),
    class: className,
    children,
    ...restProps
  }: WithElementRef<HTMLLiAttributes> = $props();
</script>

<li
  bind:this={ref}
  data-slot="breadcrumb-separator"
  role="presentation"
  aria-hidden="true"
  class={cn('[&>svg]:size-3.5', className)}
  {...restProps}
>
  {#if children}
    {@render children?.()}
  {:else}
    <CaretRight class="size-3.5" weight="bold" />
  {/if}
</li>
