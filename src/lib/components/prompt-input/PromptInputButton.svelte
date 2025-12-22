<script lang="ts">
  import { cn } from '$lib/utils.js';
  import { Button, type ButtonProps } from '$lib/components/ui/button/index.js';

  interface Props extends Omit<ButtonProps, 'size'> {
    size?: 'default' | 'sm' | 'icon' | 'icon-sm';
    children?: import('svelte').Snippet;
  }

  let {
    variant = 'ghost',
    size = 'icon-sm',
    class: className,
    children,
    ...props
  }: Props = $props();
</script>

<Button {variant} {size} class={cn('rounded-full', className)} type="button" {...props}>
  {#if children}
    {@render children()}
  {/if}
</Button>
