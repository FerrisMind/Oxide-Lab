<script lang="ts">
  import { cn } from '$lib/utils';

  interface Props {
    size?: number;
    label?: string;
    testId?: string;
    class?: string;
  }

  let { size = 16, label, testId, class: className = '' }: Props = $props();
</script>

<span
  class={cn('ui-spinner', className)}
  style={`--spinner-size: ${size}px`}
  role={label ? 'status' : undefined}
  aria-label={label}
  aria-hidden={label ? undefined : 'true'}
  data-testid={testId}
></span>

<style>
  .ui-spinner {
    width: var(--spinner-size);
    height: var(--spinner-size);
    display: inline-block;
    vertical-align: middle;
    border-radius: 9999px;
    box-sizing: border-box;
    border: 2px solid color-mix(in srgb, currentColor 25%, transparent 75%);
    border-top-color: currentColor;
    animation: ui-spinner-rotate 0.8s linear infinite;
  }

  @keyframes ui-spinner-rotate {
    to {
      transform: rotate(360deg);
    }
  }
</style>
