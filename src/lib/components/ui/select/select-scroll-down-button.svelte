<script lang="ts">
  import CaretDown from 'phosphor-svelte/lib/CaretDown';
  import { Select as SelectPrimitive } from 'bits-ui';
  import { cn, type WithoutChildrenOrChild } from '$lib/components/ai-elements/markdown/utils/utils';

  let {
    ref = $bindable(null),
    class: className,
    ...restProps
  }: WithoutChildrenOrChild<SelectPrimitive.ScrollDownButtonProps> = $props();
</script>

<SelectPrimitive.ScrollDownButton
  bind:ref
  data-slot="select-scroll-down-button"
  class={cn('flex cursor-default items-center justify-center py-1', className)}
  {...restProps}
>
  <CaretDown class="size-4" weight="bold" />
</SelectPrimitive.ScrollDownButton>
