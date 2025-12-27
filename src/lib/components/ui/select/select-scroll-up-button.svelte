<script lang="ts">
  import CaretUp from 'phosphor-svelte/lib/CaretUp';
  import { Select as SelectPrimitive } from 'bits-ui';
  import { cn, type WithoutChildrenOrChild } from '$lib/components/ai-elements/markdown/utils/utils';

  let {
    ref = $bindable(null),
    class: className,
    ...restProps
  }: WithoutChildrenOrChild<SelectPrimitive.ScrollUpButtonProps> = $props();
</script>

<SelectPrimitive.ScrollUpButton
  bind:ref
  data-slot="select-scroll-up-button"
  class={cn('flex cursor-default items-center justify-center py-1', className)}
  {...restProps}
>
  <CaretUp class="size-4" weight="bold" />
</SelectPrimitive.ScrollUpButton>
