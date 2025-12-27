<script lang="ts">
  /**
   * Model Icon Component
   *
   * Displays an icon for a model based on its family or name.
   * Uses Lobe Icons SVG library from npm package.
   */
  import { getModelIconName } from '$lib/components/ai-elements/markdown/utils/model-icons';
  import Cube from 'phosphor-svelte/lib/Cube';

  interface Props {
    family?: string | null;
    size?: number;
    class?: string;
  }

  let { family = null, size = 24, class: className = '' }: Props = $props();

  // Pre-load all icons using import.meta.glob
  const iconModules = import.meta.glob('/node_modules/@lobehub/icons-static-svg/icons/*.svg', {
    query: '?url',
    import: 'default',
    eager: true,
  }) as Record<string, string>;

  let iconName = $derived(getModelIconName(family));
  let hasError = $state(false);

  // Get icon src from preloaded modules
  let iconSrc = $derived.by(() => {
    // Try color version first
    const colorKey = `/node_modules/@lobehub/icons-static-svg/icons/${iconName}-color.svg`;
    if (iconModules[colorKey]) {
      return iconModules[colorKey];
    }
    // Fallback to non-color
    const defaultKey = `/node_modules/@lobehub/icons-static-svg/icons/${iconName}.svg`;
    if (iconModules[defaultKey]) {
      return iconModules[defaultKey];
    }
    return null;
  });

  function handleError() {
    hasError = true;
  }

  // Reset error when icon changes
  $effect(() => {
    iconName;
    hasError = false;
  });
</script>

{#if hasError || !iconSrc}
  <Cube {size} weight="duotone" class={className} />
{:else}
  <img
    src={iconSrc}
    alt={family || 'Model'}
    width={size}
    height={size}
    class={className}
    onerror={handleError}
  />
{/if}

<style>
  img {
    object-fit: contain;
  }
</style>
