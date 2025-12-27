<script lang="ts">
  import { cn } from '$lib/components/ai-elements/markdown/utils/utils';
  import type { HTMLAttributes } from 'svelte/elements';

  type MessageRole = 'user' | 'assistant' | 'system' | 'data';

  type MessageProps = HTMLAttributes<HTMLDivElement> & {
    from: MessageRole;
  };

  let { class: className = '', from, children, ...restProps }: MessageProps = $props();

  let id = $props.id();

  const messageClasses = $derived.by(() =>
    cn(
      'group flex w-full items-end justify-end gap-2 py-4',
      from === 'user' ? 'is-user' : 'is-assistant flex-row-reverse justify-end',
      className,
    ),
  );
</script>

<div class={messageClasses} data-message-id={id} {...restProps}>
  {@render children?.()}
</div>
