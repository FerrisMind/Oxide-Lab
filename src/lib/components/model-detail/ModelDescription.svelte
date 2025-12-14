<script lang="ts">
  import type { HFModel } from '$lib/services/huggingface';
  import { renderMarkdownToSafeHtml } from '$lib/chat/markdown';
  import { enableExternalLinks } from '$lib/chat/external-links';

  interface Props {
    model: HFModel | null;
    detailedModel: HFModel | null;
    detailsLoading: boolean;
  }

  let { model, detailedModel, detailsLoading }: Props = $props();

  let descriptionEl: HTMLElement | undefined = $state();

  // Apply link enhancements when description is rendered
  $effect(() => {
    if (descriptionEl) {
      enableExternalLinks(descriptionEl);
    }
  });
</script>

{#if detailedModel?.description || model?.description}
  <div class="model-description">
    <h3>Описание</h3>
    <div class="description-content md-stream" bind:this={descriptionEl}>
      {@html renderMarkdownToSafeHtml(detailedModel?.description || model?.description || '')}
    </div>
  </div>
{:else if !detailsLoading}
  <div class="model-description">
    <h3>Описание</h3>
    <p class="no-description">Описание недоступно</p>
  </div>
{/if}

<style>
  .model-description {
    margin-bottom: var(--space-4); /* 24px */
  }

  .model-description h3 {
    font-size: 1.125rem;
    font-weight: var(--font-weight-semibold);
    color: var(--text);
    margin: 0 0 var(--space-2) 0; /* 8px → 12px closest */
  }

  .model-description p {
    color: var(--text);
    line-height: 1.6;
    margin: 0;
    opacity: 0.8;
  }

  .description-content {
    color: var(--text);
    line-height: 1.6;
    opacity: 0.9;
  }

  .no-description {
    color: var(--muted);
    font-style: italic;
  }

  /* Responsive images in markdown content */
  .description-content :global(img) {
    max-width: 100%;
    height: auto;
    border-radius: var(--radius-lg);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    margin: 1em 0;
    display: block;
    transition: all 0.2s ease;
  }

  .description-content :global(img:hover) {
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
    transform: translateY(-2px);
  }

  /* Handle images inside paragraphs or other containers */
  .description-content :global(p) :global(img) {
    margin: 0.5em 0;
  }

  /* Responsive image containers */
  .description-content :global(figure) {
    margin: 1em 0;
    text-align: center;
  }

  .description-content :global(figure) :global(img) {
    margin: 0 auto 0.5em;
  }

  .description-content :global(figcaption) {
    font-size: 0.875rem;
    color: var(--muted);
    font-style: italic;
    margin-top: 0.5em;
  }
</style>
